//! The IO side of `TrainerSim` (TCK-159): maps `GenomeRecord` / `TrainerRunState` onto the rows of `TrainerRepo`
//! and logs failures (the trainer continues without the DB). Used by the engine and the headless runner.

use crate::persistence::{LogDbError, TrainerGenome, TrainerRepo, TrainerRunStateRow};
use crate::trainer::mode::Scenario;
use crate::trainer::sim::{GenomeRecord, TrainerRunState, TrainerSim};

fn to_row(record: &GenomeRecord, generation: u32) -> TrainerGenome {
    TrainerGenome {
        name: record.name.clone(),
        genome: record.genome.clone(),
        fitness: record.fitness as f64,
        generation: generation as i64,
        methylations: record.methylations.clone(),
        synapse_weights: record.synapse_weights.clone(),
    }
}

fn to_record(row: TrainerGenome) -> GenomeRecord {
    GenomeRecord {
        genome: row.genome,
        name: row.name,
        fitness: row.fitness as f32,
        generation: row.generation as u32,
        methylations: row.methylations,
        synapse_weights: row.synapse_weights,
    }
}

/// Persists a completed generation: previous elites become inactive, the new elites are inserted as active parents,
/// then the run is pruned to `retention` rows (TCK-183).
pub fn save_elites(
    repo: &(impl TrainerRepo + ?Sized),
    run_id: &str,
    generation: u32,
    elites: &[GenomeRecord],
    retention: usize,
) {
    let rows: Vec<TrainerGenome> = elites.iter().map(|e| to_row(e, generation)).collect();
    repo.save_trainer_generation(run_id, &rows, Some(retention))
        .log_db_err("saving trainer elites");
}

/// Best genomes of a run by fitness: the active parents (`active_only`) or the hall of fame; empty on a read error
/// (logged).
pub fn load_genomes(
    repo: &(impl TrainerRepo + ?Sized),
    run_id: &str,
    active_only: bool,
    limit: usize,
) -> Vec<GenomeRecord> {
    match repo.trainer_genomes(run_id, active_only, limit as i64) {
        Ok(rows) => rows.into_iter().map(to_record).collect(),
        Err(e) => {
            log::error!("loading trainer genomes failed: {}", e);
            Vec::new()
        }
    }
}

/// A sandbox assignment from the UI, stored as an active parent of the current generation (fitness 0).
pub fn save_assigned(
    repo: &(impl TrainerRepo + ?Sized),
    run_id: &str,
    generation: u32,
    slot: u32,
    record: &GenomeRecord,
) {
    repo.assign_trainer_genome(run_id, slot, &to_row(record, generation))
        .log_db_err("saving the assigned trainer genome");
}

/// Stores the run state of `sim` (TCK-191, `TrainerRunState::of`); called right after `save_elites`, so it is the
/// state the next generation runs with.
pub fn save_run_state(repo: &(impl TrainerRepo + ?Sized), run_id: &str, sim: &TrainerSim) {
    let state = TrainerRunState::of(sim);
    let row = TrainerRunStateRow {
        scenario: state.scenario.mode().to_string(),
        level: state.level,
        level_auto: state.level_auto,
        layout_id: state.layout_id,
        mutation_ops_max: state.mutation_ops_max,
        population: state.population,
        seed: state.seed,
    };
    repo.save_trainer_run_state(run_id, &row)
        .log_db_err("saving the trainer run state");
}

/// Saved run state of `run_id`; `None` without a row or on a read error (logged), i.e. today's defaults.
pub fn load_run_state(repo: &(impl TrainerRepo + ?Sized), run_id: &str) -> Option<TrainerRunState> {
    match repo.trainer_run_state(run_id) {
        Ok(row) => row.map(|r| TrainerRunState {
            scenario: Scenario::from_mode(&r.scenario),
            level: r.level,
            level_auto: r.level_auto,
            layout_id: r.layout_id,
            mutation_ops_max: r.mutation_ops_max,
            population: r.population,
            seed: r.seed,
        }),
        Err(e) => {
            log::error!("loading the trainer run state failed: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{MemoryDb, init_db};
    use crate::trainer::sim::TrainerParams;

    fn elite(fitness: f32) -> GenomeRecord {
        GenomeRecord {
            genome: format!("GENOME{}", fitness),
            name: "elite".to_string(),
            fitness,
            generation: 1,
            methylations: vec![0.5],
            synapse_weights: vec![0.25, 0.75],
        }
    }

    /// A trainer at generation 2 with non-default run state (curriculum, manual level, given layout, 5 ops).
    fn trainer_with_state(layout_id: u64) -> TrainerSim {
        let params = TrainerParams {
            population: 2,
            scenario: Scenario::Curriculum,
            level: 0.4,
            level_auto: false,
            mutation_ops_max: 5,
            ..TrainerParams::from_config(crate::config::test_config())
        };
        TrainerSim::restore_at_layout(
            params,
            3,
            2,
            layout_id,
            Vec::new(),
            Vec::new(),
            10,
            crate::config::test_config(),
        )
    }

    fn retention_keeps_best_rows_and_active_parents(repo: &(impl TrainerRepo + ?Sized)) {
        save_elites(repo, "run", 1, &[elite(90.0), elite(50.0)], 3);
        save_elites(repo, "run", 2, &[elite(80.0), elite(70.0)], 3);
        save_elites(repo, "other", 1, &[elite(1.0)], 3);
        // Gen 3 elites are worse than the top 3 but stay as active parents
        save_elites(repo, "run", 3, &[elite(10.0), elite(5.0)], 3);

        let hof: Vec<f32> = load_genomes(repo, "run", false, 100)
            .iter()
            .map(|r| r.fitness)
            .collect();
        assert_eq!(hof, vec![90.0, 80.0, 70.0, 10.0, 5.0]);
        let parents = load_genomes(repo, "run", true, 100);
        assert_eq!(parents.iter().map(|r| r.fitness).collect::<Vec<_>>(), vec![10.0, 5.0]);
        assert_eq!(parents[0].generation, 3);
        assert_eq!(parents[0].methylations, vec![0.5]);
        assert_eq!(parents[0].synapse_weights, vec![0.25, 0.75]);
        assert_eq!(
            load_genomes(repo, "other", false, 100).len(),
            1,
            "other runs are untouched"
        );
    }

    #[test]
    fn test_trainer_retention_keeps_best_rows_and_active_parents() {
        retention_keeps_best_rows_and_active_parents(&init_db(":memory:").unwrap());
        retention_keeps_best_rows_and_active_parents(&MemoryDb::new());
    }

    fn run_state_round_trip_and_missing_row(repo: &(impl TrainerRepo + ?Sized)) {
        assert_eq!(load_run_state(repo, "run"), None, "no row: today's defaults");

        // u64 beyond i64 survives the decimal text column
        let sim = trainer_with_state(u64::MAX - 1);
        save_run_state(repo, "run", &sim);
        let loaded = load_run_state(repo, "run").unwrap();
        assert_eq!(loaded, TrainerRunState::of(&sim));
        assert_eq!(
            (
                loaded.scenario,
                loaded.level,
                loaded.level_auto,
                loaded.layout_id,
                loaded.mutation_ops_max
            ),
            (Scenario::Curriculum, 0.4, false, u64::MAX - 1, 5)
        );
        // The run seed (TCK-194) survives the text column beyond i64 too
        let reseeded = TrainerSim::restore_at_layout(
            sim.params.clone(),
            u64::MAX - 2,
            2,
            0,
            Vec::new(),
            Vec::new(),
            10,
            crate::config::test_config(),
        );
        save_run_state(repo, "run", &reseeded);
        assert_eq!(load_run_state(repo, "run").unwrap().seed, Some(u64::MAX - 2));

        // Saving again replaces the run's row; other runs stay separate
        save_run_state(repo, "run", &trainer_with_state(7));
        assert_eq!(load_run_state(repo, "run").unwrap().layout_id, 7);
        assert_eq!(load_run_state(repo, "other"), None);
    }

    #[test]
    fn test_trainer_run_state_round_trip_and_missing_row() {
        run_state_round_trip_and_missing_row(&init_db(":memory:").unwrap());
        run_state_round_trip_and_missing_row(&MemoryDb::new());
    }

    fn delete_run_clears_that_run_only(repo: &(impl TrainerRepo + ?Sized)) {
        for run in ["run", "other"] {
            save_elites(repo, run, 1, &[elite(1.0)], 10);
            save_run_state(repo, run, &trainer_with_state(7));
        }
        repo.delete_trainer_run("run").unwrap();
        assert!(load_genomes(repo, "run", false, 100).is_empty());
        assert_eq!(load_run_state(repo, "run"), None);
        assert_eq!(load_genomes(repo, "other", false, 100).len(), 1);
        assert!(load_run_state(repo, "other").is_some());
    }

    #[test]
    fn test_delete_trainer_run_clears_genomes_and_run_state_of_that_run_only() {
        delete_run_clears_that_run_only(&init_db(":memory:").unwrap());
        delete_run_clears_that_run_only(&MemoryDb::new());
    }
}
