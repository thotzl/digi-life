use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Pre-1.12.8 location (relative to the working directory). Still the default for the CLI
/// trainer and the one-time migration source for the desktop app.
pub const LEGACY_DB_PATH: &str = "target/pixel_life_local.db";
pub const DB_FILE_NAME: &str = "pixel_life_local.db";

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Sets the database location once at startup (the desktop app passes its app data dir).
pub fn set_db_path(path: PathBuf) {
    if DB_PATH.set(path).is_err() {
        eprintln!("[DB] Database path was already set; ignoring second assignment.");
    }
}

pub fn db_path() -> PathBuf {
    DB_PATH.get().cloned().unwrap_or_else(|| PathBuf::from(LEGACY_DB_PATH))
}

/// Shared connection for Tauri commands, initialized once at startup.
pub struct DbState(pub Mutex<Connection>);

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

/// Logs a failed database operation instead of silently discarding it.
pub trait LogDbError {
    fn log_db_err(self, context: &str);
}

impl<T> LogDbError for Result<T> {
    fn log_db_err(self, context: &str) {
        if let Err(e) = self {
            eprintln!("[DB] {} failed: {}", context, e);
        }
    }
}

/// Initializes the local SQLite database and creates all necessary schemas if they do not exist.
pub fn init_db<P: AsRef<Path>>(path: P) -> Result<Connection> {
    // Automatically ensure the target parent directory exists
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let conn = Connection::open(path)?;

    // WAL lets the simulation thread write while commands read; busy timeout instead of SQLITE_BUSY errors
    conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get::<_, String>(0))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    // Enable foreign keys for data integrity
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    // 1. Create sessions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            config_json TEXT NOT NULL
        );",
        [],
    )?;

    // 2. Create species_records table (lineage tracker matching TS perfectly)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS species_records (
            id TEXT PRIMARY KEY,
            latin_name TEXT NOT NULL UNIQUE,
            genome_string TEXT NOT NULL,
            parent_name TEXT,
            status TEXT CHECK(status IN ('alive', 'extinct')) DEFAULT 'alive',
            peak_population INTEGER DEFAULT 1,
            birth_time INTEGER NOT NULL,
            extinction_time INTEGER,
            generation INTEGER DEFAULT 1,
            carnivory REAL NOT NULL,
            registered_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
        [],
    )?;

    // 3. Create simulation_history table (time-series logging)
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

    // Indexing for faster history lookups by session_id
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_history_session ON simulation_history(session_id);",
        [],
    )?;

    // 4. Create simulation_state table (session persistence)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS simulation_state (
            key TEXT PRIMARY KEY,
            state_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );",
        [],
    )?;

    // 5. Create trainer_genomes table (RL evolutionary trainer lineage)
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

    // Auto-migrate if columns are missing in older databases
    {
        let mut stmt = conn.prepare("PRAGMA table_info(trainer_genomes);")?;
        let mut columns = Vec::new();
        let mut rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        while let Some(Ok(col_name)) = rows.next() {
            columns.push(col_name);
        }

        if !columns.contains(&"methylations".to_string()) {
            let _ = conn.execute("ALTER TABLE trainer_genomes ADD COLUMN methylations TEXT NOT NULL DEFAULT '[]';", []);
        }
        if !columns.contains(&"synapse_weights".to_string()) {
            let _ = conn.execute("ALTER TABLE trainer_genomes ADD COLUMN synapse_weights TEXT NOT NULL DEFAULT '[]';", []);
        }
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_trainer_run ON trainer_genomes(run_id);",
        [],
    )?;

    // 6. Create creature_catalogue table (Persistent cross-simulation library)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS creature_catalogue (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            genome TEXT NOT NULL,
            source TEXT NOT NULL,
            fitness REAL NOT NULL DEFAULT 0.0,
            carnivory REAL NOT NULL DEFAULT 0.0,
            methylations TEXT NOT NULL,
            synapse_weights TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
        [],
    )?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    #[test]
    fn test_db_initialization_in_memory() {
        // Run tests inside memory database to isolate tests from file systems
        let result = init_db(":memory:");
        assert!(result.is_ok());

        let conn = result.unwrap();
        
        // Assert that we can insert a test session
        let session_inserted = conn.execute(
            "INSERT INTO sessions (id, name, config_json) VALUES (?1, ?2, ?3)",
            &["test-session-123", "Default Ocean", "{}"],
        );
        assert_eq!(session_inserted.unwrap(), 1);

        // Assert foreign key cascade
        let history_inserted = conn.execute(
            "INSERT INTO simulation_history (session_id, generation, average_fitness, max_fitness, population_count) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            &["test-session-123", "1", "12.5", "45.0", "150"],
        );
        assert!(history_inserted.is_ok());
    }

    #[test]
    fn test_creature_catalogue_crud_and_migrations() {
        let conn = init_db(":memory:").expect("Failed to initialize memory DB");

        // Test insertion of a complete cryo-clone
        let insert_res = conn.execute(
            "INSERT INTO creature_catalogue (id, name, genome, source, fitness, carnivory, methylations, synapse_weights)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                "cryo-123",
                "Aqueus pulsa",
                "COLOOOEN",
                "Trainer FastRun",
                1250.5,
                0.25,
                "[0.5,0.0,0.9]",
                "[1.5,-2.0,0.4]"
            ],
        );
        assert_eq!(insert_res.unwrap(), 1);

        // Test querying and deserialization check
        let mut stmt = conn.prepare("SELECT name, fitness, methylations, synapse_weights FROM creature_catalogue WHERE id = 'cryo-123'").unwrap();
        let mut rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?
            ))
        }).unwrap();

        let (name, fitness, methylations_str, synapse_weights_str) = rows.next().unwrap().unwrap();
        assert_eq!(name, "Aqueus pulsa");
        assert_eq!(fitness, 1250.5);
        assert_eq!(methylations_str, "[0.5,0.0,0.9]");
        assert_eq!(synapse_weights_str, "[1.5,-2.0,0.4]");

        // Test renaming
        conn.execute("UPDATE creature_catalogue SET name = 'Super Predator' WHERE id = 'cryo-123'", []).unwrap();
        let new_name: String = conn.query_row("SELECT name FROM creature_catalogue WHERE id = 'cryo-123'", [], |r| r.get(0)).unwrap();
        assert_eq!(new_name, "Super Predator");

        // Test deletion
        conn.execute("DELETE FROM creature_catalogue WHERE id = 'cryo-123'", []).unwrap();
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM creature_catalogue", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("digi-life-test-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_file_db_uses_wal_journal() {
        let dir = temp_dir("wal");
        let conn = init_db(dir.join("test.db")).unwrap();
        let mode: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap();
        assert_eq!(mode, "wal");
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_legacy_db_is_copied_once_and_never_overwrites() {
        let dir = temp_dir("migrate");
        let legacy = dir.join("legacy.db");
        let target = dir.join("appdata").join("pixel_life_local.db");

        // No legacy DB: nothing happens
        assert!(!migrate_legacy_db(&legacy, &target).unwrap());
        assert!(!target.exists());

        // Legacy DB present, target missing: copied
        std::fs::write(&legacy, b"legacy-content").unwrap();
        assert!(migrate_legacy_db(&legacy, &target).unwrap());
        assert_eq!(std::fs::read(&target).unwrap(), b"legacy-content");

        // Target already present: never overwritten
        std::fs::write(&legacy, b"newer-legacy").unwrap();
        assert!(!migrate_legacy_db(&legacy, &target).unwrap());
        assert_eq!(std::fs::read(&target).unwrap(), b"legacy-content");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
