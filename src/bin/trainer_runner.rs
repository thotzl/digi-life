//! Headless runner (TCK-173): runs `TrainerSim` without UI and warp cap and writes a JSON report.
//! In memory by default; `--persist --db <path>` writes the elites and the run state (TCK-191) of every generation to
//! that DB.
//! `probe` runs the mechanics diagnosis probes of TCK-172 (`diagnostics::probes`); its `--db` is opened read-only.
//! `ocean` runs the IO-free `OceanSim` from a fresh start and writes a time-series report (TCK-158); it opens no DB.

use std::collections::HashMap;
use std::process::ExitCode;

use src_tauri::diagnostics::ocean_report::{run_ocean, OceanRunConfig, DEFAULT_SAMPLE_EVERY};
use src_tauri::diagnostics::probes::{load_hof_genomes, run_probe, ProbeOptions, PROBE_NAMES};
use src_tauri::diagnostics::report::{run_trainer, RunConfig, ScheduleSpan};
use src_tauri::trainer::mode::TrainingMode;
use src_tauri::trainer::room::registry;
use src_tauri::trainer::sim::{CurriculumParams, LayoutPolicy, MutationChannelMode, TrainerEvent, TrainerParams, TrialAggregation};
use src_tauri::biology::trainer_store;
use src_tauri::persistence::init_db;
use src_tauri::config::AppConfig;

const USAGE: &str = "Usage:
  trainer_runner run --seed <str> --generations <N> --scenario <room id>|curriculum
                     (room ids: the room registry, today standard|exploration)
                     [--population <N; default config trainerPopulation>]
                     [--schedule <room>:<gens>,... (room ids or curriculum; replaces --scenario,
                      sets --generations)] [--eval-final <room id>,... e.g. standard,exploration]
                     [--eval-trials <N; default --trials>]
                     [--curriculum-upper <x>] [--curriculum-lower <x>] [--curriculum-step <x>] [--start-level <0..1>]
                     [--rehearsal <share 0..1>] [--ramp-start <x>] (defaults: config trainerCurriculum)
                     [--trials <N; default config trainerTrials>] [--aggregation mean|median]
                     [--one-fifth true|false] [--elite-ratio <0..1; default 0.25>]
                     [--minimal-criterion true|false; default config trainerMinimalCriterion.enabled]
                     [--layout-policy hold|rotate; default config trainerLayoutPolicy]
                     [--mutation-channels mixed|separate; default config mutationChannels]
                     [--multi-trial true|false] [--parallel true|false]
                     [--persist --db <path>] [--run-id <id>] [--out <file>]
  trainer_runner probe heredity|speed|equipment|steering|wall|hebbian|topology|metabolism|all --seed <str>
                     [--out <file; directory for 'all'>] [--db <path: HoF genomes, opened read-only>]
                     [--hof-limit 64] [--samples <N random genomes>]
                     (default out: .agents/artifacts/probes/<name>.json)
  trainer_runner ocean --seed <str> --ticks <N> [--sample-every <K; default 100>] [--out <file>]
                     (fresh Ocean, restock with structured random founders, no DB)";

/// Default output directory of the probes.
const PROBE_DIR: &str = ".agents/artifacts/probes";

/// Flags without a value.
const SWITCHES: &[&str] = &["persist"];

fn main() -> ExitCode {
    src_tauri::logging::init_console();
    // Entry point: the one place the runner reads the process config; everything below receives it
    let app = AppConfig::global();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("run") => parse_flags(&args[1..]).and_then(|flags| run(&flags, app)),
        Some("ocean") => parse_flags(&args[1..]).and_then(|flags| ocean(&flags, app)),
        Some("probe") => match args.get(1) {
            Some(name) if !name.starts_with("--") => parse_flags(&args[2..]).and_then(|flags| probe(name, &flags, app)),
            _ => Err("probe needs a name".to_string()),
        },
        _ => Err("missing or unknown subcommand".to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}\n\n{}", e, USAGE);
            ExitCode::from(2)
        }
    }
}

/// `--key value` pairs and value-less switches.
fn parse_flags(args: &[String]) -> Result<HashMap<String, String>, String> {
    let mut flags = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let key = arg.strip_prefix("--").ok_or_else(|| format!("unexpected argument '{}'", arg))?;
        let value = if SWITCHES.contains(&key) {
            "true".to_string()
        } else {
            iter.next().ok_or_else(|| format!("--{} needs a value", key))?.clone()
        };
        flags.insert(key.to_string(), value);
    }
    Ok(flags)
}

fn required<'a>(flags: &'a HashMap<String, String>, key: &str) -> Result<&'a str, String> {
    flags.get(key).map(String::as_str).ok_or_else(|| format!("--{} is required", key))
}

fn parse<T: std::str::FromStr>(flags: &HashMap<String, String>, key: &str, default: Option<T>) -> Result<T, String> {
    match flags.get(key) {
        Some(v) => v.parse().map_err(|_| format!("invalid value '{}' for --{}", v, key)),
        None => default.ok_or_else(|| format!("--{} is required", key)),
    }
}

fn run(flags: &HashMap<String, String>, app: &AppConfig) -> Result<(), String> {
    const KNOWN: &[&str] = &[
        "seed", "generations", "scenario", "population", "trials", "aggregation", "one-fifth", "elite-ratio", "minimal-criterion", "mutation-channels", "layout-policy", "multi-trial", "parallel", "persist", "db",
        "run-id", "out", "schedule", "eval-final", "curriculum-upper", "curriculum-lower", "curriculum-step", "start-level", "rehearsal", "ramp-start", "eval-trials",
    ];
    if let Some(unknown) = flags.keys().find(|k| !KNOWN.contains(&k.as_str())) {
        return Err(format!("unknown flag --{}", unknown));
    }

    let schedule = match flags.get("schedule") {
        Some(spec) => parse_schedule(spec)?,
        None => Vec::new(),
    };
    let scenario = match (flags.get("scenario"), schedule.first()) {
        (Some(name), first) => {
            let scenario = parse_room(name)?;
            if first.is_some_and(|span| span.scenario != scenario) {
                return Err("--scenario differs from the first --schedule room".to_string());
            }
            scenario
        }
        (None, Some(span)) => span.scenario,
        (None, None) => return Err("--scenario or --schedule is required".to_string()),
    };
    let scheduled: Option<u32> = (!schedule.is_empty()).then(|| schedule.iter().map(|span| span.generations).sum());
    let generations: u32 = parse(flags, "generations", scheduled)?;
    if let Some(total) = scheduled.filter(|&total| total != generations) {
        return Err(format!("--generations {} differs from the --schedule total {}", generations, total));
    }
    let eval_final = match flags.get("eval-final") {
        Some(list) => list.split(',').map(|name| parse_room(name.trim())).collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let eval_final = eval_final
        .into_iter()
        .map(|mode| match mode {
            TrainingMode::Room(id) => Ok(id),
            TrainingMode::Curriculum => Err(format!("--eval-final takes registry rooms ({})", registry::ids().collect::<Vec<_>>().join(", "))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let defaults = RunConfig::with_defaults(required(flags, "seed")?, scenario, parse(flags, "population", Some(app.trainer.trainer_population))?, generations, app);
    let d = &defaults.params;
    let c = d.curriculum;
    let curriculum = CurriculumParams {
        upper: parse(flags, "curriculum-upper", Some(c.upper))?,
        lower: parse(flags, "curriculum-lower", Some(c.lower))?,
        step: parse(flags, "curriculum-step", Some(c.step))?,
        start_level: parse(flags, "start-level", Some(c.start_level))?,
        rehearsal_share: parse(flags, "rehearsal", Some(c.rehearsal_share))?,
        metabolic_ramp_start: parse(flags, "ramp-start", Some(c.metabolic_ramp_start))?,
        ..c
    };
    if !(0.0..=1.0).contains(&curriculum.start_level) || !(0.0..=1.0).contains(&curriculum.rehearsal_share) {
        return Err("--start-level and --rehearsal must be in [0, 1]".to_string());
    }
    if !(curriculum.step > 0.0 && curriculum.step <= 1.0) || curriculum.lower > curriculum.upper {
        return Err("--curriculum-step must be in (0, 1] and --curriculum-lower <= --curriculum-upper".to_string());
    }
    let trial_aggregation = match flags.get("aggregation").map(String::as_str) {
        None => d.trial_aggregation,
        Some("mean") => TrialAggregation::Mean,
        Some("median") => TrialAggregation::Median,
        Some(other) => return Err(format!("unknown aggregation '{}'", other)),
    };
    let mutation_channels = match flags.get("mutation-channels").map(String::as_str) {
        None => d.mutation_channels,
        Some("mixed") => MutationChannelMode::Mixed,
        Some("separate") => MutationChannelMode::Separate,
        Some(other) => return Err(format!("unknown mutation channel mode '{}'", other)),
    };
    let layout_policy = match flags.get("layout-policy").map(String::as_str) {
        None => d.layout_policy,
        Some("hold") => LayoutPolicy::Hold,
        Some("rotate") => LayoutPolicy::Rotate,
        Some(other) => return Err(format!("unknown layout policy '{}'", other)),
    };
    let elite_ratio: f32 = parse(flags, "elite-ratio", Some(d.elite_ratio))?;
    if !(elite_ratio > 0.0 && elite_ratio <= 1.0) {
        return Err(format!("--elite-ratio must be in (0, 1], got {}", elite_ratio));
    }
    let params = TrainerParams {
        trials: parse(flags, "trials", Some(d.trials))?,
        trial_aggregation,
        multi_trial: parse(flags, "multi-trial", Some(d.multi_trial))?,
        one_fifth_rule: parse(flags, "one-fifth", Some(d.one_fifth_rule))?,
        elite_ratio,
        minimal_criterion: parse(flags, "minimal-criterion", Some(d.minimal_criterion))?,
        mutation_channels,
        layout_policy,
        curriculum,
        ..d.clone()
    };
    let config = RunConfig {
        params,
        schedule,
        eval_final,
        eval_trials: if flags.contains_key("eval-trials") { Some(parse(flags, "eval-trials", None)?) } else { None },
        parallel: parse(flags, "parallel", Some(false))?,
        ..defaults
    };
    let persist = flags.contains_key("persist");
    let run_id = flags.get("run-id").cloned().unwrap_or_else(|| format!("headless-{}", config.seed));

    // The DB is opened only with --persist, and only at an explicit path (never the app DB by default)
    let conn = match (persist, flags.get("db")) {
        (true, Some(path)) => {
            log::info!("persisting elites to {} (run id '{}')", path, run_id);
            Some(init_db(path).map_err(|e| format!("cannot open DB {}: {}", path, e))?)
        }
        (true, None) => return Err("--persist requires --db <path>".to_string()),
        (false, Some(_)) => return Err("--db is only used with --persist".to_string()),
        (false, None) => None,
    };
    let retention = app.trainer.trainer_genome_retention_per_run;

    let p = &config.params;
    log::info!(
        "seed '{}', scenario {}, population {}, generations {}, trials {} ({:?}), multi-trial {}, elite ratio {}, 1/5 rule {}, minimal criterion {}, mutation channels {:?}, parallel {}",
        config.seed,
        config.first_mode().id(),
        p.population,
        config.generations,
        p.trials,
        p.trial_aggregation,
        p.multi_trial,
        p.elite_ratio,
        p.one_fifth_rule,
        p.minimal_criterion,
        p.mutation_channels,
        config.parallel
    );
    if p.mode() == TrainingMode::Curriculum || config.schedule.iter().any(|span| span.scenario == TrainingMode::Curriculum) {
        log::info!("curriculum {:?}", p.curriculum);
    }
    if !config.schedule.is_empty() {
        log::info!("schedule {:?}", config.schedule);
    }
    let report = run_trainer(&config, app, |event, sim| {
        let TrainerEvent::GenerationCompleted { generation, elites, stats, .. } = event;
        log::info!(
            "generation {}/{}: {} level {:.2}, best {:.2}, median {:.2}, success {:.2}{}",
            generation,
            config.generations,
            stats.scenario.id(),
            stats.level,
            stats.best_fitness,
            stats.median_fitness,
            stats.success_rate,
            stats.level_move.map(|m| format!(", gate {:?}", m)).unwrap_or_default()
        );
        if let Some(conn) = &conn {
            trainer_store::save_elites(conn, &run_id, *generation, elites, retention);
            trainer_store::save_run_state(conn, &run_id, sim);
        }
    });
    for eval in &report.final_eval {
        log::info!("final eval {}: success {:.3}, mean fitness {:.3}", eval.scenario, eval.eval.success_rate, eval.eval.mean_fitness);
    }
    log::info!(
        "done in {:.2} s ({:.0} ticks/s total, {:.0} per sandbox; generation ends {:.2} s)",
        report.perf.wall_seconds, report.perf.ticks_per_second_total, report.perf.ticks_per_second_per_sandbox, report.perf.generation_end_seconds
    );

    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    match flags.get("out") {
        Some(path) => write_file(path, json),
        None => {
            println!("{}", json);
            Ok(())
        }
    }
}

/// Room of `--scenario`, `--schedule` and `--eval-final`: a registry room id or "curriculum".
fn parse_room(name: &str) -> Result<TrainingMode, String> {
    TrainingMode::parse(name)
}

/// `room:gens,room:gens,...` (e.g. `standard:30,exploration:30,standard:10`).
fn parse_schedule(spec: &str) -> Result<Vec<ScheduleSpan>, String> {
    spec.split(',')
        .map(|part| {
            let (room, gens) = part.trim().split_once(':').ok_or_else(|| format!("schedule span '{}' is not room:generations", part))?;
            let generations: u32 = gens.parse().map_err(|_| format!("invalid generation count in schedule span '{}'", part))?;
            if generations == 0 {
                return Err(format!("schedule span '{}' has no generations", part));
            }
            Ok(ScheduleSpan { scenario: parse_room(room)?, generations })
        })
        .collect()
}

fn write_file(path: &str, json: String) -> Result<(), String> {
    if let Some(dir) = std::path::Path::new(path).parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {}", dir.display(), e))?;
    }
    std::fs::write(path, json + "\n").map_err(|e| format!("cannot write {}: {}", path, e))
}

fn ocean(flags: &HashMap<String, String>, app: &AppConfig) -> Result<(), String> {
    const KNOWN: &[&str] = &["seed", "ticks", "sample-every", "out"];
    if let Some(unknown) = flags.keys().find(|k| !KNOWN.contains(&k.as_str())) {
        return Err(format!("unknown flag --{}", unknown));
    }
    let config = OceanRunConfig {
        seed: required(flags, "seed")?.to_string(),
        ticks: parse(flags, "ticks", None)?,
        sample_every: parse(flags, "sample-every", Some(DEFAULT_SAMPLE_EVERY))?,
    };
    if config.sample_every == 0 {
        return Err("--sample-every must be at least 1".to_string());
    }
    log::info!("ocean seed '{}', {} ticks, sample every {}", config.seed, config.ticks, config.sample_every);
    let report = run_ocean(&config, app, |sample| {
        log::info!(
            "tick {}: population {}, births {}, deaths {}, species alive {}, mean energy {}",
            sample.tick,
            sample.state.population,
            sample.counts.births,
            sample.counts.deaths.total(),
            sample.state.species_alive,
            sample.state.mean_energy.map_or("-".to_string(), |e| format!("{:.2}", e))
        );
    });
    let s = &report.summary;
    log::info!(
        "done in {:.2} s ({:.0} ticks/s): population {} (min {}, max {}), births {}, deaths {} (starved {}, aged {}, hunted {}), restocked {}, species {} alive / {} extinct",
        report.perf.wall_seconds, report.perf.ticks_per_second, s.state.population, s.min_population, s.max_population, s.counts.births, s.deaths_total,
        s.counts.deaths.starved, s.counts.deaths.aged, s.counts.deaths.hunted, s.counts.restocked, s.state.species_alive, s.state.species_extinct
    );
    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    match flags.get("out") {
        Some(path) => write_file(path, json),
        None => {
            println!("{}", json);
            Ok(())
        }
    }
}

fn probe(name: &str, flags: &HashMap<String, String>, app: &AppConfig) -> Result<(), String> {
    const KNOWN: &[&str] = &["seed", "out", "db", "hof-limit", "samples"];
    if let Some(unknown) = flags.keys().find(|k| !KNOWN.contains(&k.as_str())) {
        return Err(format!("unknown flag --{}", unknown));
    }
    let all = name == "all";
    let names: Vec<&str> = if all { PROBE_NAMES.to_vec() } else { vec![name] };
    if !all && !PROBE_NAMES.contains(&name) {
        return Err(format!("unknown probe '{}'", name));
    }

    let hof = match flags.get("db") {
        Some(path) => {
            let hof = load_hof_genomes(path, parse(flags, "hof-limit", Some(64))?)?;
            log::info!("{} distinct HoF genomes from {} (read-only)", hof.len(), path);
            hof
        }
        None => Vec::new(),
    };
    let opts = ProbeOptions {
        seed: required(flags, "seed")?.to_string(),
        samples: if flags.contains_key("samples") { Some(parse(flags, "samples", None)?) } else { None },
        hof_source: flags.get("db").cloned(),
        hof,
        config: app.clone(),
    };

    for name in names {
        let path = match flags.get("out") {
            Some(dir) if all => format!("{}/{}.json", dir.trim_end_matches(['/', '\\']), name),
            Some(file) => file.clone(),
            None => format!("{}/{}.json", PROBE_DIR, name),
        };
        let started = std::time::Instant::now();
        let report = run_probe(name, &opts)?;
        write_file(&path, serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?)?;
        log::info!("probe {} -> {} ({:.1} s)", name, path, started.elapsed().as_secs_f32());
    }
    Ok(())
}
