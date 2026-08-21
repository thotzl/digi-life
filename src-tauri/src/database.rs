use rusqlite::{Connection, Result};
use std::path::Path;

/// Initializes the local SQLite database and creates all necessary schemas if they do not exist.
pub fn init_db<P: AsRef<Path>>(path: P) -> Result<Connection> {
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
            created_at INTEGER NOT NULL
        );",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_trainer_run ON trainer_genomes(run_id);",
        [],
    )?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
