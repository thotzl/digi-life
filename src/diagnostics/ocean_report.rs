//! Headless Ocean run and its JSON report (TCK-158, used by `trainer_runner ocean`).
//! Runs the IO-free `OceanSim` (TCK-156) from a fresh start without UI, warp cap or database. Everything except `perf`
//! is a pure function of the seed text, the tick count, the sample period and the config (same inputs → identical
//! report on every platform build that the trainer reference also holds on). The Ocean step is sequential (no rayon),
//! so there is no parallel variant.

use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::shared::stats::Distribution;
use crate::sim::ocean::{DeathCause, FRESH_FOUNDERS, OceanEvent, OceanSim, OceanStep, ocean_seed};

/// Default sample period (ticks) of the time series.
pub const DEFAULT_SAMPLE_EVERY: u64 = 100;

/// Run settings of a headless Ocean run.
#[derive(Debug, Clone, PartialEq)]
pub struct OceanRunConfig {
    /// Seed text; the Ocean seed is `ocean_seed(seed, None)`, the one the app uses for a fresh Ocean with `core.seed`
    pub seed: String,
    pub ticks: u64,
    /// A sample every `sample_every` ticks (plus tick 0 and the last tick)
    pub sample_every: u64,
}

/// Report header.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OceanReportMeta {
    pub seed: String,
    /// `ocean_seed(seed, None)` as 16 hex digits (a u64 does not survive a JSON number)
    pub ocean_seed: String,
    /// `core.seed`: the procedural world (reefs, vents) comes from it, not from `seed`
    pub world_seed: String,
    pub ticks: u64,
    pub sample_every: u64,
    pub founders: usize,
    pub target_population: usize,
    pub max_creatures: usize,
    pub food_spore_count: usize,
    pub world_width: f32,
    pub world_height: f32,
    /// Restock source: always structured random founders (the app's empty-catalogue branch; no DB is read)
    pub restock: String,
    pub version: String,
}

/// Deaths by cause.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deaths {
    pub starved: u32,
    pub aged: u32,
    pub hunted: u32,
}

impl Deaths {
    pub fn total(&self) -> u32 {
        self.starved + self.aged + self.hunted
    }

    fn add(&mut self, cause: DeathCause) {
        match cause {
            DeathCause::Starved => self.starved += 1,
            DeathCause::Aged => self.aged += 1,
            DeathCause::Hunted => self.hunted += 1,
        }
    }

    fn merge(&mut self, other: &Deaths) {
        self.starved += other.starved;
        self.aged += other.aged;
        self.hunted += other.hunted;
    }
}

/// Event counts over an interval (since the previous sample, or the whole run in the summary).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OceanCounts {
    /// Mitosis splits (each child joins the population, TCK-143)
    pub births: u32,
    /// Splits whose child genome mutated
    pub mutated_births: u32,
    /// Founders added by the restock
    pub restocked: u32,
    pub deaths: Deaths,
    pub bites: u32,
    pub spores_eaten: u32,
    /// Species seen for the first time
    pub species_discovered: u32,
    /// Species whose last member died
    pub extinctions: u32,
}

impl OceanCounts {
    /// Counts the events and species changes of one step.
    fn add(&mut self, step: &OceanStep) {
        for event in &step.events {
            match event {
                OceanEvent::Ate { .. } => self.spores_eaten += 1,
                OceanEvent::Bite { .. } => self.bites += 1,
                OceanEvent::Died { cause, .. } => self.deaths.add(*cause),
                OceanEvent::Mitosis { mutated, .. } => {
                    self.births += 1;
                    self.mutated_births += u32::from(*mutated);
                }
                OceanEvent::Extinct { .. } => self.extinctions += 1,
                OceanEvent::Restocked { .. } => self.restocked += 1,
            }
        }
        self.species_discovered += step
            .species
            .iter()
            .filter(|c| matches!(c, crate::sim::ocean::SpeciesChange::Discovered { .. }))
            .count() as u32;
    }

    fn merge(&mut self, other: &OceanCounts) {
        self.births += other.births;
        self.mutated_births += other.mutated_births;
        self.restocked += other.restocked;
        self.deaths.merge(&other.deaths);
        self.bites += other.bites;
        self.spores_eaten += other.spores_eaten;
        self.species_discovered += other.species_discovered;
        self.extinctions += other.extinctions;
    }
}

/// State of the Ocean after a tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OceanState {
    pub population: usize,
    /// Ledger species with living members / without
    pub species_alive: usize,
    pub species_extinct: usize,
    /// Over the living population; `null` when it is empty
    pub mean_energy: Option<f32>,
    pub median_energy: Option<f32>,
    pub mean_age: Option<f32>,
    /// Plant (`type_id` 1) and meat (`type_id` 2) spores; the field keeps its size
    pub spores_plant: usize,
    pub spores_meat: usize,
    pub highest_generation: u32,
}

impl OceanState {
    fn of(sim: &OceanSim, energies: &mut Vec<f32>) -> Self {
        let creatures = sim.creatures();
        energies.clear();
        energies.extend(creatures.iter().map(|c| c.energy));
        let energy = Distribution::of(energies);
        let mean_age = (!creatures.is_empty())
            .then(|| (creatures.iter().map(|c| c.age as f64).sum::<f64>() / creatures.len() as f64) as f32);
        let entries = sim.ledger().entries();
        let species_alive = entries.iter().filter(|e| e.alive).count();
        let spores = &sim.spores().spores;
        let spores_meat = spores.iter().filter(|s| s.type_id == 2).count();
        Self {
            population: creatures.len(),
            species_alive,
            species_extinct: entries.len() - species_alive,
            mean_energy: energy.as_ref().map(|d| d.mean),
            median_energy: energy.as_ref().map(|d| d.p50),
            mean_age,
            spores_plant: spores.len() - spores_meat,
            spores_meat,
            highest_generation: sim.highest_generation(),
        }
    }
}

/// One point of the time series: the state after `tick` and the events since the previous sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OceanSample {
    pub tick: u64,
    #[serde(flatten)]
    pub state: OceanState,
    #[serde(flatten)]
    pub counts: OceanCounts,
}

/// Whole-run summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OceanSummary {
    /// State after the last tick
    #[serde(flatten)]
    pub state: OceanState,
    /// Events over the whole run (founders of the fresh start not counted)
    #[serde(flatten)]
    pub counts: OceanCounts,
    pub deaths_total: u32,
    /// Population after each tick (before the first tick: the founders)
    pub min_population: usize,
    pub max_population: usize,
    pub mean_population: f32,
    /// Ledger size: founder species plus every species discovered during the run
    pub species_total: usize,
}

/// Throughput; excluded from reproducibility comparisons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OceanPerf {
    pub wall_seconds: f64,
    pub ticks_per_second: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OceanReport {
    pub meta: OceanReportMeta,
    pub samples: Vec<OceanSample>,
    pub summary: OceanSummary,
    pub perf: OceanPerf,
}

/// Runs a fresh Ocean for `config.ticks` ticks (restock with structured random founders) and builds the report.
/// `on_sample` receives every sample (progress output).
pub fn run_ocean(config: &OceanRunConfig, app: &AppConfig, mut on_sample: impl FnMut(&OceanSample)) -> OceanReport {
    let sample_every = config.sample_every.max(1);
    let seed = ocean_seed(&config.seed, None);
    let (mut sim, _founders) = OceanSim::fresh(app, seed);
    let ocean = &app.ocean;
    let meta = OceanReportMeta {
        seed: config.seed.clone(),
        ocean_seed: format!("{:016x}", seed),
        world_seed: app.core.seed.clone(),
        ticks: config.ticks,
        sample_every,
        founders: FRESH_FOUNDERS,
        target_population: ocean.target_population,
        max_creatures: ocean.max_creatures,
        food_spore_count: ocean.food_spore_count,
        world_width: ocean.world_width,
        world_height: ocean.world_height,
        restock: "structured_random".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let mut energies = Vec::new();
    let mut samples = Vec::with_capacity((config.ticks / sample_every) as usize + 2);
    let first = OceanSample {
        tick: 0,
        state: OceanState::of(&sim, &mut energies),
        counts: OceanCounts::default(),
    };
    on_sample(&first);
    samples.push(first);

    let mut interval = OceanCounts::default();
    let mut total = OceanCounts::default();
    let founders = sim.creatures().len();
    let (mut min_population, mut max_population, mut population_sum) = (founders, founders, 0u64);
    let start = Instant::now();
    for tick in 1..=config.ticks {
        let step = sim.tick(Vec::new);
        interval.add(&step);
        let population = sim.creatures().len();
        min_population = min_population.min(population);
        max_population = max_population.max(population);
        population_sum += population as u64;
        if tick % sample_every == 0 || tick == config.ticks {
            let sample = OceanSample {
                tick,
                state: OceanState::of(&sim, &mut energies),
                counts: interval,
            };
            on_sample(&sample);
            samples.push(sample);
            total.merge(&interval);
            interval = OceanCounts::default();
        }
    }
    let wall_seconds = start.elapsed().as_secs_f64();

    let summary = OceanSummary {
        state: OceanState::of(&sim, &mut energies),
        counts: total,
        deaths_total: total.deaths.total(),
        min_population,
        max_population,
        mean_population: if config.ticks > 0 {
            (population_sum as f64 / config.ticks as f64) as f32
        } else {
            founders as f32
        },
        species_total: sim.ledger().entries().len(),
    };
    OceanReport {
        meta,
        samples,
        summary,
        perf: OceanPerf {
            wall_seconds,
            ticks_per_second: if wall_seconds > 0.0 {
                config.ticks as f64 / wall_seconds
            } else {
                0.0
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(seed: &str, ticks: u64, sample_every: u64) -> OceanReport {
        run_ocean(
            &OceanRunConfig {
                seed: seed.to_string(),
                ticks,
                sample_every,
            },
            crate::config::test_config(),
            |_| {},
        )
    }

    /// Report without the run-dependent `perf` (the comparator also drops `meta.version`).
    fn reproducible(report: &OceanReport) -> String {
        let mut value = serde_json::to_value(report).unwrap();
        value.as_object_mut().unwrap().remove("perf");
        value.to_string()
    }

    #[test]
    fn test_ocean_report_is_reproducible_per_seed_and_samples_every_k_ticks() {
        let a = run("ocean-test", 250, 100);
        let b = run("ocean-test", 250, 100);
        assert_eq!(reproducible(&a), reproducible(&b), "same seed, same report");
        assert_ne!(
            reproducible(&a),
            reproducible(&run("ocean-other", 250, 100)),
            "the seed matters"
        );

        assert_eq!(
            a.samples.iter().map(|s| s.tick).collect::<Vec<_>>(),
            vec![0, 100, 200, 250],
            "tick 0, every K, the last tick"
        );
        assert_eq!(a.samples[0].state.population, FRESH_FOUNDERS);
        // The first tick restocks the fresh founders up to the target population
        let target = crate::config::test_config().ocean.target_population;
        assert!(
            a.samples[1].counts.restocked as usize >= target - FRESH_FOUNDERS,
            "{:?}",
            a.samples[1].counts
        );
        // Interval counts add up to the summary
        let births: u32 = a.samples.iter().map(|s| s.counts.births).sum();
        let deaths: u32 = a.samples.iter().map(|s| s.counts.deaths.total()).sum();
        assert_eq!((births, deaths), (a.summary.counts.births, a.summary.deaths_total));
        // Population balance: founders + restocked + mitosis children - deaths (TCK-143)
        assert_eq!(
            a.summary.state.population as i64,
            FRESH_FOUNDERS as i64 + a.summary.counts.restocked as i64 + births as i64 - deaths as i64
        );
        assert_eq!(
            a.summary.species_total,
            a.summary.state.species_alive + a.summary.state.species_extinct
        );
        assert_eq!(a.summary.state, a.samples.last().unwrap().state);
    }
}
