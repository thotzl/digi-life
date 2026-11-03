//! `trainer_genomes` and `trainer_run_state`: tables, migration steps and the SQLite `TrainerRepo`.

use rusqlite::{Connection, OptionalExtension, Result, params};

use super::dto::*;
use super::repos::TrainerRepo;
use super::schema::TableDef;
use super::{Db, collect_rows, unique_suffix};
use crate::shared::{json_col, time};

pub(super) const TABLES: &[TableDef] = &[
    TableDef {
        name: "trainer_genomes",
        model_scoped: true,
        create: &[
            "CREATE TABLE IF NOT EXISTS trainer_genomes (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            generation INTEGER NOT NULL,
            name TEXT NOT NULL,
            genome TEXT NOT NULL,
            fitness REAL NOT NULL,
            active INTEGER DEFAULT 1,
            created_at INTEGER NOT NULL,
            methylations TEXT NOT NULL DEFAULT '[]',
            synapse_weights TEXT NOT NULL DEFAULT '[]'
        );",
            "CREATE INDEX IF NOT EXISTS idx_trainer_run ON trainer_genomes(run_id);",
        ],
    },
    // Per-run trainer state a restore continues with (TCK-191); layout_id and seed are u64 as decimal text
    TableDef {
        name: "trainer_run_state",
        model_scoped: true,
        create: &["CREATE TABLE IF NOT EXISTS trainer_run_state (
            run_id TEXT PRIMARY KEY,
            scenario TEXT NOT NULL,
            level REAL NOT NULL,
            level_auto INTEGER NOT NULL,
            layout_id TEXT NOT NULL,
            mutation_ops_max INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            population INTEGER,
            seed TEXT
        );"],
    },
];

fn columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({});", table))?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;
    Ok(names)
}

fn add_column_if_missing(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
    if !columns(conn, table)?.iter().any(|c| c == column) {
        conn.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {};", table, column, definition),
            [],
        )?;
    }
    Ok(())
}

/// Migration 1 (frozen): both tables as of v1.12.30, plus the columns older databases lack (formerly the hand-written
/// checks of `init_db`; their errors are no longer discarded): `methylations`/`synapse_weights` of `trainer_genomes`,
/// `population` (TCK-192) and `seed` (TCK-194) of `trainer_run_state`.
pub(super) fn v1_baseline(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trainer_genomes (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            generation INTEGER NOT NULL,
            name TEXT NOT NULL,
            genome TEXT NOT NULL,
            fitness REAL NOT NULL,
            active INTEGER DEFAULT 1,
            created_at INTEGER NOT NULL,
            methylations TEXT NOT NULL DEFAULT '[]',
            synapse_weights TEXT NOT NULL DEFAULT '[]'
        );",
        [],
    )?;
    add_column_if_missing(conn, "trainer_genomes", "methylations", "TEXT NOT NULL DEFAULT '[]'")?;
    add_column_if_missing(conn, "trainer_genomes", "synapse_weights", "TEXT NOT NULL DEFAULT '[]'")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_trainer_run ON trainer_genomes(run_id);",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trainer_run_state (
            run_id TEXT PRIMARY KEY,
            scenario TEXT NOT NULL,
            level REAL NOT NULL,
            level_auto INTEGER NOT NULL,
            layout_id TEXT NOT NULL,
            mutation_ops_max INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            population INTEGER,
            seed TEXT
        );",
        [],
    )?;
    add_column_if_missing(conn, "trainer_run_state", "population", "INTEGER")?;
    add_column_if_missing(conn, "trainer_run_state", "seed", "TEXT")?;
    Ok(())
}

const INSERT_GENOME: &str = "INTO trainer_genomes (id, run_id, generation, name, genome, fitness, active, created_at, methylations, synapse_weights)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9)";

/// `INSERT` (or `INSERT OR REPLACE` with `replace`) of one active genome row.
fn insert_genome(conn: &Connection, replace: bool, id: &str, run_id: &str, g: &TrainerGenome) -> Result<()> {
    conn.execute(
        &format!(
            "{} {}",
            if replace { "INSERT OR REPLACE" } else { "INSERT" },
            INSERT_GENOME
        ),
        params![
            id,
            run_id,
            g.generation,
            g.name,
            g.genome,
            g.fitness,
            time::now_secs(),
            json_col::encode(&g.methylations, "trainer_genomes.methylations"),
            json_col::encode(&g.synapse_weights, "trainer_genomes.synapse_weights")
        ],
    )?;
    Ok(())
}

fn prune(conn: &Connection, run_id: &str, retention: usize) -> Result<()> {
    conn.execute(
        "DELETE FROM trainer_genomes WHERE run_id = ?1 AND active = 0 AND id NOT IN
         (SELECT id FROM trainer_genomes WHERE run_id = ?1 ORDER BY fitness DESC LIMIT ?2)",
        params![run_id, retention as i64],
    )?;
    Ok(())
}

impl TrainerRepo for Db {
    fn save_trainer_generation(
        &self,
        run_id: &str,
        genomes: &[TrainerGenome],
        retention: Option<usize>,
    ) -> RepoResult<()> {
        let tx = self.conn().unchecked_transaction()?;
        tx.execute(
            "UPDATE trainer_genomes SET active = 0 WHERE run_id = ?1",
            params![run_id],
        )?;
        for (idx, g) in genomes.iter().enumerate() {
            let id = format!("{}-{}-{}-{}", run_id, g.generation, idx, unique_suffix());
            insert_genome(&tx, false, &id, run_id, g)?;
        }
        if let Some(retention) = retention {
            prune(&tx, run_id, retention)?;
        }
        tx.commit()?;
        Ok(())
    }

    fn assign_trainer_genome(&self, run_id: &str, slot: u32, g: &TrainerGenome) -> RepoResult<()> {
        let id = format!("{}-{}-assigned-{}", run_id, g.generation, slot);
        Ok(insert_genome(self.conn(), true, &id, run_id, g)?)
    }

    fn add_trainer_genome(&self, run_id: &str, g: &TrainerGenome) -> RepoResult<()> {
        let id = format!("{}-{}-added-{}", run_id, g.generation, unique_suffix());
        Ok(insert_genome(self.conn(), false, &id, run_id, g)?)
    }

    fn prune_trainer_genomes(&self, run_id: &str, retention: usize) -> RepoResult<()> {
        Ok(prune(self.conn(), run_id, retention)?)
    }

    fn trainer_genomes(&self, run_id: &str, active_only: bool, limit: i64) -> RepoResult<Vec<TrainerGenome>> {
        let filter = if active_only { " AND active = 1" } else { "" };
        let mut stmt = self.conn().prepare(&format!(
            "SELECT genome, name, fitness, generation, methylations, synapse_weights FROM trainer_genomes WHERE run_id = ?1{} ORDER BY fitness DESC LIMIT ?2",
            filter
        ))?;
        let rows = stmt.query_map(params![run_id, limit], |row| {
            Ok(TrainerGenome {
                genome: row.get(0)?,
                name: row.get(1)?,
                fitness: row.get(2)?,
                generation: row.get(3)?,
                methylations: json_col::decode(&row.get::<_, String>(4)?, "trainer_genomes.methylations")
                    .unwrap_or_default(),
                synapse_weights: json_col::decode(&row.get::<_, String>(5)?, "trainer_genomes.synapse_weights")
                    .unwrap_or_default(),
            })
        })?;
        Ok(collect_rows(rows, "trainer_genomes"))
    }

    fn ranked_trainer_genomes(&self) -> RepoResult<Vec<RankedGenome>> {
        let mut stmt = self
            .conn()
            .prepare("SELECT genome, fitness, run_id, generation FROM trainer_genomes ORDER BY fitness DESC, id ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(RankedGenome {
                genome: row.get(0)?,
                fitness: row.get(1)?,
                run_id: row.get(2)?,
                generation: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>>>()?)
    }

    fn trainer_runs(&self) -> RepoResult<Vec<TrainerRunSummary>> {
        let mut stmt = self.conn().prepare(
            "SELECT run_id, MAX(generation), MAX(fitness) FROM trainer_genomes GROUP BY run_id ORDER BY run_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TrainerRunSummary {
                run_id: row.get(0)?,
                max_generation: row.get(1)?,
                max_fitness: row.get(2)?,
            })
        })?;
        Ok(collect_rows(rows, "trainer_genomes"))
    }

    fn save_trainer_run_state(&self, run_id: &str, s: &TrainerRunStateRow) -> RepoResult<()> {
        self.conn().execute(
            "INSERT OR REPLACE INTO trainer_run_state (run_id, scenario, level, level_auto, layout_id, mutation_ops_max, updated_at, population, seed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                run_id,
                s.scenario,
                s.level as f64,
                s.level_auto,
                s.layout_id.to_string(),
                s.mutation_ops_max as i64,
                time::now_secs(),
                s.population.map(|n| n as i64),
                s.seed.map(|s| s.to_string())
            ],
        )?;
        Ok(())
    }

    fn trainer_run_state(&self, run_id: &str) -> RepoResult<Option<TrainerRunStateRow>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT scenario, level, level_auto, layout_id, mutation_ops_max, population, seed FROM trainer_run_state WHERE run_id = ?1",
                params![run_id],
                |row| {
                    Ok(TrainerRunStateRow {
                        scenario: row.get(0)?,
                        level: row.get::<_, f64>(1)? as f32,
                        level_auto: row.get(2)?,
                        layout_id: row.get::<_, String>(3)?.parse().unwrap_or(0),
                        mutation_ops_max: row.get::<_, i64>(4)?.clamp(0, u32::MAX as i64) as u32,
                        population: row.get::<_, Option<i64>>(5)?.filter(|&n| n > 0).map(|n| n as usize),
                        seed: row.get::<_, Option<String>>(6)?.and_then(|s| s.parse().ok()),
                    })
                },
            )
            .optional()?)
    }

    fn delete_trainer_run(&self, run_id: &str) -> RepoResult<()> {
        let tx = self.conn().unchecked_transaction()?;
        tx.execute("DELETE FROM trainer_genomes WHERE run_id = ?1", params![run_id])?;
        tx.execute("DELETE FROM trainer_run_state WHERE run_id = ?1", params![run_id])?;
        tx.commit()?;
        Ok(())
    }
}
