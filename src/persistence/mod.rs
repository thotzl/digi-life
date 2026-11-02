//! Persistence (TCK-159): the only module that contains SQL.
//!
//! | File | Content |
//! | :--- | :--- |
//! | `mod.rs` | DB location, `Db` (SQLite store), `init_db`, `open_read_only`, `DbState`, `LogDbError` |
//! | `schema.rs` | `user_version` migrations, `MODEL_VERSION` reset with backup |
//! | `repos.rs` | repository traits `SpeciesRepo`, `CatalogueRepo`, `TrainerRepo`, `OceanSnapshotRepo` |
//! | `dto.rs` | plain rows the traits take and return, `RepoError` |
//! | `species.rs`, `catalogue.rs`, `trainer.rs`, `ocean.rs` | per table group: latest tables, migration steps, SQLite impl |
//! | `memory.rs` | `MemoryDb`, the in-memory implementation of all traits |
//!
//! Contract: `wiki/contracts/persistence-repo.md`; guide: `wiki/guides/add-table.md`.

mod catalogue;
pub mod dto;
pub mod memory;
mod ocean;
pub mod repos;
mod schema;
mod species;
mod trainer;
#[cfg(test)]
mod tests;

pub use dto::*;
pub use memory::MemoryDb;
pub use repos::{CatalogueRepo, OceanSnapshotRepo, SpeciesRepo, TrainerRepo};
pub use schema::MODEL_VERSION;

use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// Pre-1.12.8 location (relative to the working directory). Still the default for the CLI
/// trainer and the one-time migration source for the desktop app.
pub const LEGACY_DB_PATH: &str = "target/pixel_life_local.db";
pub const DB_FILE_NAME: &str = "pixel_life_local.db";

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Sets the database location once at startup (the desktop app passes its app data dir).
pub fn set_db_path(path: PathBuf) {
    if DB_PATH.set(path).is_err() {
        log::warn!("database path was already set; ignoring the second assignment");
    }
}

pub fn db_path() -> PathBuf {
    DB_PATH.get().cloned().unwrap_or_else(|| PathBuf::from(LEGACY_DB_PATH))
}

/// Copies a legacy database to the new location if the new one does not exist yet.
/// Never overwrites an existing target. Returns whether a copy happened.
pub fn migrate_legacy_db(legacy: &Path, target: &Path) -> std::io::Result<bool> {
    if target.exists() || !legacy.exists() {
        return Ok(false);
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(legacy, target)?;
    Ok(true)
}

/// The SQLite store: one connection, implements every repository trait.
pub struct Db {
    conn: Connection,
}

impl Db {
    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

/// Shared store for Tauri commands, initialized once at startup.
pub struct DbState(pub Mutex<Db>);

/// Opens (creating if needed) the database at `path` (`":memory:"` for tests): WAL journal, 5 s busy timeout,
/// foreign keys, schema migrations, model-version check, missing tables created.
pub fn init_db<P: AsRef<Path>>(path: P) -> RepoResult<Db> {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(path)?;
    // WAL lets the simulation thread write while commands read; busy timeout instead of SQLITE_BUSY errors
    conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get::<_, String>(0))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    schema::prepare(&conn)?;
    Ok(Db { conn })
}

/// Opens an existing database strictly read-only (probes, analysis): no migration, no model reset, writes fail.
pub fn open_read_only<P: AsRef<Path>>(path: P) -> RepoResult<Db> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
    Ok(Db { conn })
}

/// Unique suffix for generated row ids: Unix ms plus a process-wide counter, so two saves within the same second or
/// millisecond never collide (the old `<run>-<gen>-<idx>-<secs>` ids did, TCK-159).
pub(crate) fn unique_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("{}-{}", crate::shared::time::now_ms(), COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Unreadable rows of a list query are skipped with a warning naming the table.
fn collect_rows<T>(rows: impl Iterator<Item = rusqlite::Result<T>>, table: &str) -> Vec<T> {
    rows.filter_map(|row| match row {
        Ok(item) => Some(item),
        Err(e) => {
            log::warn!("skipping unreadable {} row: {}", table, e);
            None
        }
    })
    .collect()
}

/// Logs a failed database operation instead of silently discarding it.
pub trait LogDbError {
    fn log_db_err(self, context: &str);
}

impl<T, E: std::fmt::Display> LogDbError for Result<T, E> {
    fn log_db_err(self, context: &str) {
        if let Err(e) = self {
            log::error!("{} failed: {}", context, e);
        }
    }
}
