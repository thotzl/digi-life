//! Headless trainer runner (TCK-173): runs `TrainerSim` without UI and warp cap and writes a JSON report.
//! In memory by default; `--persist --db <path>` writes the elites of every generation to that DB.
//! `probe` runs the mechanics diagnosis probes of TCK-172 (`biology::probes`); its `--db` is opened read-only.

use std::collections::HashMap;
use std::process::ExitCode;

use src_tauri::biology::probes::{load_hof_genomes, run_probe, ProbeOptions, PROBE_NAMES};
use src_tauri::biology::scenarios::Scenario;
use src_tauri::biology::trainer_report::{run_trainer, RunConfig, ScheduleSpan};
use src_tauri::biology::trainer_sim::{CurriculumParams, LayoutPolicy, MutationChannelMode, TrainerEvent, TrialAggregation};
use src_tauri::database::{init_db, save_trainer_elites};
use src_tauri::shared::types::AppConfig;

const USAGE: &str = "Usage:
  trainer_runner run --seed <str> --generations <N> --scenario standard|exploration|curriculum --population <N>
                     [--schedule <room>:<gens>,... (rooms standard|exploration|curriculum; replaces --scenario,
                      sets --generations)] [--eval-final standard,exploration] [--eval-trials <N; default --trials>]
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
                     (default out: .agents/artifacts/probes/<name>.json)";

/// Default output directory of the probes.
const PROBE_DIR: &str = ".agents/artifacts/probes";

/// Flags without a value.
const SWITCHES: &[&str] = &["persist"];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("run") => parse_flags(&args[1..]).and_then(|flags| run(&flags)),
        Some("probe") => match args.get(1) {
            Some(name) if !name.starts_with("--") => parse_flags(&args[2..]).and_then(|flags| probe(name, &flags)),
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

fn run(flags: &HashMap<String, String>) -> Result<(), String> {
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
    if eval_final.contains(&Scenario::Curriculum) {
        return Err("--eval-final takes fixed rooms (standard, exploration)".to_string());
    }
    let defaults = RunConfig::with_defaults(required(flags, "seed")?, scenario, parse(flags, "population", None)?, generations);
    let c = defaults.curriculum;
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
        None => defaults.trial_aggregation,
        Some("mean") => TrialAggregation::Mean,
        Some("median") => TrialAggregation::Median,
        Some(other) => return Err(format!("unknown aggregation '{}'", other)),
    };
    let mutation_channels = match flags.get("mutation-channels").map(String::as_str) {
        None => defaults.mutation_channels,
        Some("mixed") => MutationChannelMode::Mixed,
        Some("separate") => MutationChannelMode::Separate,
        Some(other) => return Err(format!("unknown mutation channel mode '{}'", other)),
    };
    let layout_policy = match flags.get("layout-policy").map(String::as_str) {
        None => defaults.layout_policy,
        Some("hold") => LayoutPolicy::Hold,
        Some("rotate") => LayoutPolicy::Rotate,
        Some(other) => return Err(format!("unknown layout policy '{}'", other)),
    };
    let elite_ratio: f32 = parse(flags, "elite-ratio", Some(defaults.elite_ratio))?;
    if !(elite_ratio > 0.0 && elite_ratio <= 1.0) {
        return Err(format!("--elite-ratio must be in (0, 1], got {}", elite_ratio));
    }
    let config = RunConfig {
        trials: parse(flags, "trials", Some(defaults.trials))?,
        trial_aggregation,
        multi_trial: parse(flags, "multi-trial", Some(defaults.multi_trial))?,
        one_fifth_rule: parse(flags, "one-fifth", Some(defaults.one_fifth_rule))?,
        elite_ratio,
        minimal_criterion: parse(flags, "minimal-criterion", Some(defaults.minimal_criterion))?,
        mutation_channels,
        layout_policy,
        curriculum,
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
            eprintln!("[runner] persisting elites to {} (run id '{}')", path, run_id);
            Some(init_db(path).map_err(|e| format!("cannot open DB {}: {}", path, e))?)
        }
        (true, None) => return Err("--persist requires --db <path>".to_string()),
        (false, Some(_)) => return Err("--db is only used with --persist".to_string()),
        (false, None) => None,
    };
    let retention = AppConfig::global().trainer_genome_retention_per_run;

    eprintln!(
        "[runner] seed '{}', scenario {:?}, population {}, generations {}, trials {} ({:?}), multi-trial {}, elite ratio {}, 1/5 rule {}, minimal criterion {}, mutation channels {:?}, parallel {}",
        config.seed,
        config.scenario,
        config.population,
        config.generations,
        config.trials,
        config.trial_aggregation,
        config.multi_trial,
        config.elite_ratio,
        config.one_fifth_rule,
        config.minimal_criterion,
        config.mutation_channels,
        config.parallel
    );
    if config.scenario == Scenario::Curriculum || config.schedule.iter().any(|span| span.scenario == Scenario::Curriculum) {
        eprintln!("[runner] curriculum {:?}", config.curriculum);
    }
    if !config.schedule.is_empty() {
        eprintln!("[runner] schedule {:?}", config.schedule);
    }
    let report = run_trainer(&config, |event| {
        let TrainerEvent::GenerationCompleted { generation, elites, stats, .. } = event;
        eprintln!(
            "[runner] generation {}/{}: {:?} level {:.2}, best {:.2}, median {:.2}, success {:.2}{}",
            generation,
            config.generations,
            stats.scenario,
            stats.level,
            stats.best_fitness,
            stats.median_fitness,
            stats.success_rate,
            stats.level_move.map(|m| format!(", gate {:?}", m)).unwrap_or_default()
        );
        if let Some(conn) = &conn {
            save_trainer_elites(conn, &run_id, *generation, elites, retention);
        }
    });
    for eval in &report.final_eval {
        eprintln!("[runner] final eval {:?}: success {:.3}, mean fitness {:.3}", eval.scenario, eval.eval.success_rate, eval.eval.mean_fitness);
    }
    eprintln!(
        "[runner] done in {:.2} s ({:.0} ticks/s total, {:.0} per sandbox; generation ends {:.2} s)",
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

/// Room name of `--scenario`, `--schedule` and `--eval-final`.
fn parse_room(name: &str) -> Result<Scenario, String> {
    match name {
        "standard" => Ok(Scenario::Standard),
        "exploration" => Ok(Scenario::Exploration),
        "curriculum" => Ok(Scenario::Curriculum),
        other => Err(format!("unknown scenario '{}'", other)),
    }
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

fn probe(name: &str, flags: &HashMap<String, String>) -> Result<(), String> {
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
            eprintln!("[probe] {} distinct HoF genomes from {} (read-only)", hof.len(), path);
            hof
        }
        None => Vec::new(),
    };
    let opts = ProbeOptions {
        seed: required(flags, "seed")?.to_string(),
        samples: if flags.contains_key("samples") { Some(parse(flags, "samples", None)?) } else { None },
        hof_source: flags.get("db").cloned(),
        hof,
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
        eprintln!("[probe] {} -> {} ({:.1} s)", name, path, started.elapsed().as_secs_f32());
    }
    Ok(())
}
