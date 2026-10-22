use rusqlite::{params, Connection, OptionalExtension, Result};
use crate::biology::trainer_sim::GenomeRecord;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Pre-1.12.8 location (relative to the working directory). Still the default for the CLI
/// trainer and the one-time migration source for the desktop app.
pub const LEGACY_DB_PATH: &str = "target/pixel_life_local.db";
pub const DB_FILE_NAME: &str = "pixel_life_local.db";

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Version of the genome/brain model the stored genomes and weights were made with (TCK-183). Bump it on every change
/// that makes stored synapse weights or phenotypes invalid: `init_db` then recreates the model tables empty.
/// 2 = Brain v2 (TCK-171 P3: role keys, gene identity by promoter spelling, weight tapes).
/// 3 = Oja learning with fixed reflexes (TCK-152: `plastic` flag in stored phenotypes; weights learned under the
///     old rule, incl. drifted reflexes, are not valid under the new one).
pub const MODEL_VERSION: u32 = 3;

/// Tables whose rows depend on the model version; dropped (and recreated empty) on a version change. No migration.
const MODEL_TABLES: [&str; 5] = ["trainer_genomes", "creature_catalogue", "simulation_state", "simulation_history", "species_records"];

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

    reset_model_tables_on_version_change(&conn)?;

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

/// Model-version rule (TCK-183): a stored `model_version` other than `MODEL_VERSION`, or none in a database that
/// already has model tables, drops the model tables (`init_db` recreates them empty). Stores the current version.
fn reset_model_tables_on_version_change(conn: &Connection) -> Result<()> {
    conn.execute("CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);", [])?;
    let stored: Option<String> = conn
        .query_row("SELECT value FROM meta WHERE key = 'model_version'", [], |row| row.get(0))
        .optional()?;
    if stored.as_deref() == Some(MODEL_VERSION.to_string().as_str()) {
        return Ok(());
    }
    let has_model_tables: bool = conn.query_row(
        &format!("SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name IN ('{}'))", MODEL_TABLES.join("', '")),
        [],
        |row| row.get(0),
    )?;
    let reset = stored.is_some() || has_model_tables;
    let tx = conn.unchecked_transaction()?;
    if reset {
        for table in MODEL_TABLES {
            tx.execute(&format!("DROP TABLE IF EXISTS {}", table), [])?;
        }
    }
    tx.execute("INSERT OR REPLACE INTO meta (key, value) VALUES ('model_version', ?1)", params![MODEL_VERSION.to_string()])?;
    tx.commit()?;
    if reset {
        println!("[DB] Model version {} -> {}: model tables reset", stored.as_deref().unwrap_or("none"), MODEL_VERSION);
    }
    Ok(())
}

// ==========================================================================
// Trainer genome persistence (the IO side of `TrainerSim`)
// ==========================================================================

/// Persists a completed generation: previous elites become inactive, the new elites are
/// inserted as active parents, then the run is pruned to `retention` rows.
pub fn save_trainer_elites(conn: &Connection, run_id: &str, generation: u32, elites: &[GenomeRecord], retention: usize) {
    conn.execute("UPDATE trainer_genomes SET active = 0 WHERE run_id = ?1", params![run_id]).log_db_err("UPDATE trainer_genomes");

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    for (idx, elite) in elites.iter().enumerate() {
        let unique_id = format!("{}-{}-{}-{}", run_id, generation, idx, now_secs);
        let methylations_json = serde_json::to_string(&elite.methylations).unwrap_or_else(|_| "[]".to_string());
        let synapse_weights_json = serde_json::to_string(&elite.synapse_weights).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO trainer_genomes (id, run_id, generation, name, genome, fitness, active, created_at, methylations, synapse_weights)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9)",
            params![
                unique_id,
                run_id,
                generation as i32,
                elite.name,
                elite.genome,
                elite.fitness as f64,
                now_secs,
                methylations_json,
                synapse_weights_json
            ],
        ).log_db_err("INSERT INTO trainer_genomes");
    }

    prune_trainer_genomes(conn, run_id, retention);
}

/// Retention (TCK-183): keeps the best `retention` rows of the run by fitness plus the active
/// parents (so a restore always finds them) and deletes the rest.
pub fn prune_trainer_genomes(conn: &Connection, run_id: &str, retention: usize) {
    conn.execute(
        "DELETE FROM trainer_genomes WHERE run_id = ?1 AND active = 0 AND id NOT IN
         (SELECT id FROM trainer_genomes WHERE run_id = ?1 ORDER BY fitness DESC LIMIT ?2)",
        params![run_id, retention as i64],
    ).log_db_err("DELETE FROM trainer_genomes (retention)");
}

/// Best genomes of a run by fitness: the active parents (`active_only`) or the hall of fame (all rows).
pub fn load_trainer_genomes(conn: &Connection, run_id: &str, active_only: bool, limit: usize) -> Vec<GenomeRecord> {
    let sql = if active_only {
        "SELECT genome, name, fitness, generation, methylations, synapse_weights FROM trainer_genomes WHERE run_id = ?1 AND active = 1 ORDER BY fitness DESC LIMIT ?2"
    } else {
        "SELECT genome, name, fitness, generation, methylations, synapse_weights FROM trainer_genomes WHERE run_id = ?1 ORDER BY fitness DESC LIMIT ?2"
    };
    let mut records = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            let rows = stmt.query_map(params![run_id, limit as i64], |row| {
                Ok(GenomeRecord {
                    genome: row.get(0)?,
                    name: row.get(1)?,
                    fitness: row.get::<_, f64>(2)? as f32,
                    generation: row.get::<_, i32>(3)? as u32,
                    methylations: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    synapse_weights: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                })
            });
            match rows {
                Ok(rows) => records.extend(rows.flatten()),
                Err(e) => eprintln!("[DB] SELECT FROM trainer_genomes failed: {}", e),
            }
        }
        Err(e) => eprintln!("[DB] SELECT FROM trainer_genomes failed: {}", e),
    }
    records
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

    fn model_version(conn: &Connection) -> Option<String> {
        conn.query_row("SELECT value FROM meta WHERE key = 'model_version'", [], |r| r.get(0)).optional().unwrap()
    }

    /// One row in every model table plus a session (not a model table; inserted once).
    fn fill_model_tables(conn: &Connection) {
        conn.execute("INSERT OR IGNORE INTO sessions (id, name, config_json) VALUES ('s', 'session', '{}')", []).unwrap();
        conn.execute("INSERT INTO simulation_history (session_id, generation, average_fitness, max_fitness, population_count) VALUES ('s', 1, 1.0, 2.0, 3)", []).unwrap();
        conn.execute("INSERT INTO simulation_state (key, state_json, updated_at) VALUES ('current_state', '{}', 0)", []).unwrap();
        conn.execute("INSERT INTO species_records (id, latin_name, genome_string, birth_time, carnivory) VALUES ('sp', 'Name', 'COL', 0, 0.0)", []).unwrap();
        conn.execute("INSERT INTO creature_catalogue (id, name, genome, source, methylations, synapse_weights) VALUES ('c', 'n', 'COL', 'src', '[]', '[]')", []).unwrap();
        save_trainer_elites(conn, "run", 1, &[elite(1.0)], 10);
    }

    fn row_counts(conn: &Connection) -> Vec<i64> {
        MODEL_TABLES.iter().chain(&["sessions"]).map(|t| conn.query_row(&format!("SELECT COUNT(*) FROM {}", t), [], |r| r.get(0)).unwrap()).collect()
    }

    #[test]
    fn test_fresh_db_stores_model_version_and_reset_drops_model_tables() {
        let conn = init_db(":memory:").unwrap();
        assert_eq!(model_version(&conn), Some(MODEL_VERSION.to_string()));
        fill_model_tables(&conn);

        // Same version: untouched
        reset_model_tables_on_version_change(&conn).unwrap();
        assert_eq!(row_counts(&conn), vec![1, 1, 1, 1, 1, 1]);

        // Older version: model tables dropped (init_db recreates them), sessions kept, version updated
        conn.execute("UPDATE meta SET value = '1' WHERE key = 'model_version'", []).unwrap();
        reset_model_tables_on_version_change(&conn).unwrap();
        let tables: i64 = conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('trainer_genomes', 'creature_catalogue', 'simulation_state', 'simulation_history', 'species_records')", [], |r| r.get(0)).unwrap();
        assert_eq!(tables, 0);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
        assert_eq!(model_version(&conn), Some(MODEL_VERSION.to_string()));
    }

    #[test]
    fn test_file_db_model_version_reset_on_reopen() {
        let dir = temp_dir("model-version");
        let path = dir.join("test.db");
        fill_model_tables(&init_db(&path).unwrap());

        // Same version: reopening keeps every row
        assert_eq!(row_counts(&init_db(&path).unwrap()), vec![1, 1, 1, 1, 1, 1]);

        // Stored version differs: model tables recreated empty, version updated, other tables kept
        init_db(&path).unwrap().execute("UPDATE meta SET value = '1' WHERE key = 'model_version'", []).unwrap();
        let conn = init_db(&path).unwrap();
        assert_eq!(row_counts(&conn), vec![0, 0, 0, 0, 0, 1]);
        assert_eq!(model_version(&conn), Some(MODEL_VERSION.to_string()));
        fill_model_tables(&conn);

        // Pre-versioning database (model tables, no stored version): reset as well
        conn.execute("DELETE FROM meta", []).unwrap();
        drop(conn);
        let conn = init_db(&path).unwrap();
        assert_eq!(row_counts(&conn), vec![0, 0, 0, 0, 0, 1]);
        assert_eq!(model_version(&conn), Some(MODEL_VERSION.to_string()));
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

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

    #[test]
    fn test_trainer_retention_keeps_best_rows_and_active_parents() {
        let conn = init_db(":memory:").unwrap();
        save_trainer_elites(&conn, "run", 1, &[elite(90.0), elite(50.0)], 3);
        save_trainer_elites(&conn, "run", 2, &[elite(80.0), elite(70.0)], 3);
        save_trainer_elites(&conn, "other", 1, &[elite(1.0)], 3);
        // Gen 3 elites are worse than the top 3 but stay as active parents
        save_trainer_elites(&conn, "run", 3, &[elite(10.0), elite(5.0)], 3);

        let hof: Vec<f32> = load_trainer_genomes(&conn, "run", false, 100).iter().map(|r| r.fitness).collect();
        assert_eq!(hof, vec![90.0, 80.0, 70.0, 10.0, 5.0]);

        let parents = load_trainer_genomes(&conn, "run", true, 100);
        assert_eq!(parents.iter().map(|r| r.fitness).collect::<Vec<_>>(), vec![10.0, 5.0]);
        assert_eq!(parents[0].generation, 3);
        assert_eq!(parents[0].methylations, vec![0.5]);
        assert_eq!(parents[0].synapse_weights, vec![0.25, 0.75]);

        // Other runs are untouched
        assert_eq!(load_trainer_genomes(&conn, "other", false, 100).len(), 1);
    }
}
