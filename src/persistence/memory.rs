//! `MemoryDb`: in-memory implementation of every repository trait (tests, headless use without a file). Same
//! observable behaviour as the SQLite store; `tests::contract` runs one suite against both.

use std::cell::RefCell;
use std::collections::HashMap;

use super::dto::*;
use super::repos::*;
use super::unique_suffix;
use crate::shared::time;

struct GenomeRow {
    id: String,
    run_id: String,
    active: bool,
    genome: TrainerGenome,
}

struct CatalogueRow {
    entry: CatalogueEntry,
    /// Insertion order (SQLite rowid) for equal `created_at`
    seq: u64,
}

#[derive(Default)]
struct Tables {
    species: Vec<SpeciesRecord>,
    catalogue: Vec<CatalogueRow>,
    catalogue_seq: u64,
    genomes: Vec<GenomeRow>,
    run_states: HashMap<String, TrainerRunStateRow>,
    snapshot: Option<String>,
    progenitor: Option<String>,
}

#[derive(Default)]
pub struct MemoryDb {
    tables: RefCell<Tables>,
}

impl MemoryDb {
    pub fn new() -> Self {
        Self::default()
    }
}

/// `CURRENT_TIMESTAMP` text (`YYYY-MM-DD HH:MM:SS`, UTC) of a Unix time in seconds.
fn sqlite_timestamp(secs: i64) -> String {
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Rows of `rows` best first by fitness (stable: insertion order on ties).
fn best_first<'a>(rows: impl Iterator<Item = &'a GenomeRow>) -> Vec<&'a GenomeRow> {
    let mut sorted: Vec<&GenomeRow> = rows.collect();
    sorted.sort_by(|a, b| b.genome.fitness.total_cmp(&a.genome.fitness));
    sorted
}

fn limited<T>(items: Vec<T>, limit: i64) -> Vec<T> {
    if limit < 0 { items } else { items.into_iter().take(limit as usize).collect() }
}

impl SpeciesRepo for MemoryDb {
    fn insert_species(&self, s: &NewSpecies) -> RepoResult<()> {
        let mut t = self.tables.borrow_mut();
        if t.species.iter().any(|r| r.id == s.genome || r.latin_name == s.latin_name) {
            return Ok(());
        }
        t.species.push(SpeciesRecord {
            id: s.genome.clone(),
            latin_name: s.latin_name.clone(),
            genome: s.genome.clone(),
            parent_name: s.parent_name.clone(),
            status: SpeciesStatus::Alive,
            peak_population: 1,
            birth_time: time::now_secs(),
            extinction_time: None,
            generation: s.generation as i32,
            carnivory: s.carnivory,
        });
        Ok(())
    }

    fn list_species(&self) -> RepoResult<Vec<SpeciesRecord>> {
        Ok(self.tables.borrow().species.clone())
    }

    fn species_states(&self) -> RepoResult<Vec<SpeciesState>> {
        Ok(self.tables.borrow().species.iter().map(|r| SpeciesState { id: r.id.clone(), status: r.status, peak_population: r.peak_population }).collect())
    }

    fn species_name(&self, id: &str) -> RepoResult<Option<String>> {
        Ok(self.tables.borrow().species.iter().find(|r| r.id == id).map(|r| r.latin_name.clone()))
    }

    fn mark_species_alive(&self, id: &str) -> RepoResult<()> {
        for r in self.tables.borrow_mut().species.iter_mut().filter(|r| r.id == id) {
            r.status = SpeciesStatus::Alive;
            r.extinction_time = None;
        }
        Ok(())
    }

    fn mark_species_extinct(&self, id: &str) -> RepoResult<()> {
        for r in self.tables.borrow_mut().species.iter_mut().filter(|r| r.id == id) {
            r.status = SpeciesStatus::Extinct;
            r.extinction_time = Some(time::now_secs());
        }
        Ok(())
    }

    fn set_species_peak(&self, id: &str, peak: i32) -> RepoResult<()> {
        for r in self.tables.borrow_mut().species.iter_mut().filter(|r| r.id == id) {
            r.peak_population = peak;
        }
        Ok(())
    }

    fn clear_species(&self) -> RepoResult<()> {
        self.tables.borrow_mut().species.clear();
        Ok(())
    }

    fn prune_extinct_species(&self, keep: usize) -> RepoResult<usize> {
        let mut t = self.tables.borrow_mut();
        // Newest extinction first, later insertion first on ties (SQLite: extinction_time DESC, rowid DESC)
        let mut extinct: Vec<(usize, i64)> =
            t.species.iter().enumerate().filter(|(_, r)| r.status == SpeciesStatus::Extinct).map(|(i, r)| (i, r.extinction_time.unwrap_or(i64::MIN))).collect();
        extinct.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));
        let drop: Vec<usize> = extinct.iter().skip(keep).map(|(i, _)| *i).collect();
        let before = t.species.len();
        let mut idx = 0;
        t.species.retain(|_| {
            let keep_row = !drop.contains(&idx);
            idx += 1;
            keep_row
        });
        Ok(before - t.species.len())
    }
}

impl CatalogueRepo for MemoryDb {
    fn list_catalogue(&self) -> RepoResult<Vec<CatalogueEntry>> {
        let t = self.tables.borrow();
        let mut rows: Vec<&CatalogueRow> = t.catalogue.iter().collect();
        rows.sort_by(|a, b| b.entry.created_at.cmp(&a.entry.created_at).then(a.seq.cmp(&b.seq)));
        Ok(rows.into_iter().map(|r| r.entry.clone()).collect())
    }

    fn catalogue_entry(&self, id: &str) -> RepoResult<Option<CatalogueEntry>> {
        Ok(self.tables.borrow().catalogue.iter().find(|r| r.entry.id == id).map(|r| r.entry.clone()))
    }

    fn save_catalogue_entry(&self, e: &NewCatalogueEntry) -> RepoResult<()> {
        let mut t = self.tables.borrow_mut();
        t.catalogue.retain(|r| r.entry.id != e.id);
        t.catalogue_seq += 1;
        let seq = t.catalogue_seq;
        t.catalogue.push(CatalogueRow {
            entry: CatalogueEntry {
                id: e.id.clone(),
                name: e.name.clone(),
                genome: e.genome.clone(),
                source: e.source.clone(),
                fitness: e.fitness,
                carnivory: e.carnivory,
                methylations: e.methylations.clone(),
                synapse_weights: e.synapse_weights.clone(),
                created_at: sqlite_timestamp(time::now_secs()),
            },
            seq,
        });
        Ok(())
    }

    fn rename_catalogue_entry(&self, id: &str, name: &str) -> RepoResult<()> {
        for r in self.tables.borrow_mut().catalogue.iter_mut().filter(|r| r.entry.id == id) {
            r.entry.name = name.to_string();
        }
        Ok(())
    }

    fn delete_catalogue_entry(&self, id: &str) -> RepoResult<()> {
        self.tables.borrow_mut().catalogue.retain(|r| r.entry.id != id);
        Ok(())
    }
}

impl MemoryDb {
    fn put_genome(&self, id: String, run_id: &str, genome: &TrainerGenome) {
        let mut t = self.tables.borrow_mut();
        let row = GenomeRow { id, run_id: run_id.to_string(), active: true, genome: genome.clone() };
        match t.genomes.iter_mut().find(|r| r.id == row.id) {
            Some(existing) => *existing = row,
            None => t.genomes.push(row),
        }
    }
}

impl TrainerRepo for MemoryDb {
    fn save_trainer_generation(&self, run_id: &str, genomes: &[TrainerGenome], retention: Option<usize>) -> RepoResult<()> {
        for r in self.tables.borrow_mut().genomes.iter_mut().filter(|r| r.run_id == run_id) {
            r.active = false;
        }
        for (idx, g) in genomes.iter().enumerate() {
            self.put_genome(format!("{}-{}-{}-{}", run_id, g.generation, idx, unique_suffix()), run_id, g);
        }
        if let Some(retention) = retention {
            self.prune_trainer_genomes(run_id, retention)?;
        }
        Ok(())
    }

    fn assign_trainer_genome(&self, run_id: &str, slot: u32, g: &TrainerGenome) -> RepoResult<()> {
        self.put_genome(format!("{}-{}-assigned-{}", run_id, g.generation, slot), run_id, g);
        Ok(())
    }

    fn add_trainer_genome(&self, run_id: &str, g: &TrainerGenome) -> RepoResult<()> {
        self.put_genome(format!("{}-{}-added-{}", run_id, g.generation, unique_suffix()), run_id, g);
        Ok(())
    }

    fn prune_trainer_genomes(&self, run_id: &str, retention: usize) -> RepoResult<()> {
        let mut t = self.tables.borrow_mut();
        let kept: Vec<String> = best_first(t.genomes.iter().filter(|r| r.run_id == run_id)).into_iter().take(retention).map(|r| r.id.clone()).collect();
        t.genomes.retain(|r| r.run_id != run_id || r.active || kept.contains(&r.id));
        Ok(())
    }

    fn trainer_genomes(&self, run_id: &str, active_only: bool, limit: i64) -> RepoResult<Vec<TrainerGenome>> {
        let t = self.tables.borrow();
        let rows = best_first(t.genomes.iter().filter(|r| r.run_id == run_id && (r.active || !active_only)));
        Ok(limited(rows.into_iter().map(|r| r.genome.clone()).collect(), limit))
    }

    fn ranked_trainer_genomes(&self) -> RepoResult<Vec<RankedGenome>> {
        let t = self.tables.borrow();
        let mut rows: Vec<&GenomeRow> = t.genomes.iter().collect();
        rows.sort_by(|a, b| b.genome.fitness.total_cmp(&a.genome.fitness).then(a.id.cmp(&b.id)));
        Ok(rows
            .into_iter()
            .map(|r| RankedGenome { genome: r.genome.genome.clone(), fitness: r.genome.fitness, run_id: r.run_id.clone(), generation: r.genome.generation })
            .collect())
    }

    fn trainer_runs(&self) -> RepoResult<Vec<TrainerRunSummary>> {
        let t = self.tables.borrow();
        let mut runs: Vec<TrainerRunSummary> = Vec::new();
        for r in &t.genomes {
            match runs.iter_mut().find(|s| s.run_id == r.run_id) {
                Some(s) => {
                    s.max_generation = s.max_generation.max(r.genome.generation);
                    s.max_fitness = s.max_fitness.max(r.genome.fitness);
                }
                None => runs.push(TrainerRunSummary { run_id: r.run_id.clone(), max_generation: r.genome.generation, max_fitness: r.genome.fitness }),
            }
        }
        runs.sort_by(|a, b| a.run_id.cmp(&b.run_id));
        Ok(runs)
    }

    fn save_trainer_run_state(&self, run_id: &str, state: &TrainerRunStateRow) -> RepoResult<()> {
        self.tables.borrow_mut().run_states.insert(run_id.to_string(), state.clone());
        Ok(())
    }

    fn trainer_run_state(&self, run_id: &str) -> RepoResult<Option<TrainerRunStateRow>> {
        Ok(self.tables.borrow().run_states.get(run_id).cloned())
    }

    fn delete_trainer_run(&self, run_id: &str) -> RepoResult<()> {
        let mut t = self.tables.borrow_mut();
        t.genomes.retain(|r| r.run_id != run_id);
        t.run_states.remove(run_id);
        Ok(())
    }
}

impl OceanSnapshotRepo for MemoryDb {
    fn save_ocean_snapshot(&self, state_json: &str) -> RepoResult<()> {
        self.tables.borrow_mut().snapshot = Some(state_json.to_string());
        Ok(())
    }

    fn ocean_snapshot(&self) -> RepoResult<Option<String>> {
        Ok(self.tables.borrow().snapshot.clone())
    }

    fn save_progenitor_genome(&self, genome: &str) -> RepoResult<()> {
        self.tables.borrow_mut().progenitor = Some(genome.to_string());
        Ok(())
    }

    fn clear_ocean_state(&self) -> RepoResult<()> {
        let mut t = self.tables.borrow_mut();
        t.snapshot = None;
        t.progenitor = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sqlite_timestamp_format() {
        assert_eq!(super::sqlite_timestamp(0), "1970-01-01 00:00:00");
        assert_eq!(super::sqlite_timestamp(1_709_251_199), "2024-02-29 23:59:59");
    }
}
