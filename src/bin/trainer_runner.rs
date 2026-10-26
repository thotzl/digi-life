//! Headless trainer runner (TCK-173): runs `TrainerSim` without UI and warp cap and writes a JSON report.
//! In memory by default; `--persist --db <path>` writes the elites of every generation to that DB.
//! `probe` runs the mechanics diagnosis probes of TCK-172 (`biology::probes`); its `--db` is opened read-only.

use std::collections::HashMap;
use std::process::ExitCode;

use src_tauri::biology::probes::{load_hof_genomes, run_probe, ProbeOptions, PROBE_NAMES};
use src_tauri::biology::scenarios::Scenario;
use src_tauri::biology::trainer_report::{run_trainer, RunConfig};
use src_tauri::biology::trainer_sim::TrainerEvent;
use src_tauri::database::{init_db, save_trainer_elites};
use src_tauri::shared::types::AppConfig;

const USAGE: &str = "Usage:
  trainer_runner run --seed <str> --generations <N> --scenario standard|exploration --population <N>
                     [--trials 3] [--multi-trial true|false] [--parallel true|false]
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
    const KNOWN: &[&str] = &["seed", "generations", "scenario", "population", "trials", "multi-trial", "parallel", "persist", "db", "run-id", "out"];
    if let Some(unknown) = flags.keys().find(|k| !KNOWN.contains(&k.as_str())) {
        return Err(format!("unknown flag --{}", unknown));
    }

    let scenario = match required(flags, "scenario")? {
        "standard" => Scenario::Standard,
        "exploration" => Scenario::Exploration,
        other => return Err(format!("unknown scenario '{}'", other)),
    };
    let config = RunConfig {
        seed: required(flags, "seed")?.to_string(),
        scenario,
        population: parse(flags, "population", None)?,
        generations: parse(flags, "generations", None)?,
        trials: parse(flags, "trials", Some(3))?,
        multi_trial: parse(flags, "multi-trial", Some(true))?,
        parallel: parse(flags, "parallel", Some(false))?,
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
        "[runner] seed '{}', scenario {:?}, population {}, generations {}, trials {}, multi-trial {}, parallel {}",
        config.seed, config.scenario, config.population, config.generations, config.trials, config.multi_trial, config.parallel
    );
    let report = run_trainer(&config, |event| {
        let TrainerEvent::GenerationCompleted { generation, elites, stats, .. } = event;
        eprintln!(
            "[runner] generation {}/{}: best {:.1}, median {:.1}, success {:.2}",
            generation, config.generations, stats.best_fitness, stats.median_fitness, stats.success_rate
        );
        if let Some(conn) = &conn {
            save_trainer_elites(conn, &run_id, *generation, elites, retention);
        }
    });
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
