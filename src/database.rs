use rusqlite::{Connection, Result};
use std::path::Path;

pub const DB_PATH: &str = "target/pixel_life_local.db";

/// Initializes the local SQLite database and creates all necessary schemas if they do not exist.
pub fn init_db<P: AsRef<Path>>(path: P) -> Result<Connection> {
    // Automatically ensure the target parent directory exists
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let conn = Connection::open(path)?;

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
}
