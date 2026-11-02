//! `simulation_state` (Ocean snapshot): tables, migration steps and the SQLite `OceanSnapshotRepo`. Also owns the
//! history of the removed `sessions` / `simulation_history` tables (migrations 1 and 2).

use rusqlite::{params, Connection, OptionalExtension, Result};

use super::dto::*;
use super::repos::OceanSnapshotRepo;
use super::schema::TableDef;
use super::Db;
use crate::shared::time;

pub(super) const TABLES: &[TableDef] = &[TableDef {
    name: "simulation_state",
    model_scoped: true,
    create: &["CREATE TABLE IF NOT EXISTS simulation_state (
            key TEXT PRIMARY KEY,
            state_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );"],
}];

/// Migration 1 (frozen): `sessions`, `simulation_history` and `simulation_state` as of v1.12.30.
pub(super) fn v1_baseline(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            config_json TEXT NOT NULL
        );",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS simulation_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            generation INTEGER NOT NULL,
            average_fitness REAL NOT NULL,
            max_fitness REAL NOT NULL,
            population_count INTEGER NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
        );",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_history_session ON simulation_history(session_id);", [])?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS simulation_state (
            key TEXT PRIMARY KEY,
            state_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );",
        [],
    )?;
    Ok(())
}

/// Migration 2 (TCK-159): `simulation_history` (Ocean energy time series every 10 s) had no reader and grew without
/// bound (RV-D03); `sessions` only existed for its foreign key (RV-D06). Both are dropped with their rows.
pub(super) fn v2_drop_history_and_sessions(conn: &Connection) -> Result<()> {
    conn.execute("DROP TABLE IF EXISTS simulation_history", [])?;
    conn.execute("DROP TABLE IF EXISTS sessions", [])?;
    Ok(())
}

impl OceanSnapshotRepo for Db {
    fn save_ocean_snapshot(&self, state_json: &str) -> RepoResult<()> {
        self.conn().execute(
            "INSERT OR REPLACE INTO simulation_state (key, state_json, updated_at) VALUES ('current_state', ?1, ?2)",
            params![state_json, time::now_ms()],
        )?;
        Ok(())
    }

    fn ocean_snapshot(&self) -> RepoResult<Option<String>> {
        Ok(self.conn().query_row("SELECT state_json FROM simulation_state WHERE key = 'current_state'", [], |row| row.get(0)).optional()?)
    }

    fn save_progenitor_genome(&self, genome: &str) -> RepoResult<()> {
        self.conn().execute(
            "INSERT OR REPLACE INTO simulation_state (key, state_json, updated_at) VALUES ('progenitor_genome', ?1, ?2)",
            params![genome, time::now_secs()],
        )?;
        Ok(())
    }

    fn clear_ocean_state(&self) -> RepoResult<()> {
        self.conn().execute("DELETE FROM simulation_state", [])?;
        Ok(())
    }
}
