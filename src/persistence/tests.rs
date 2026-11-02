use super::schema::{latest_version, model_tables, tables, MIGRATIONS};
use super::*;
use rusqlite::{params, Connection, OptionalExtension};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("digi-life-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn model_version(conn: &Connection) -> Option<String> {
    conn.query_row("SELECT value FROM meta WHERE key = 'model_version'", [], |r| r.get(0)).optional().unwrap()
}

fn user_version(conn: &Connection) -> u32 {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap()
}

fn genome(fitness: f64, generation: i64) -> TrainerGenome {
    TrainerGenome { name: "elite".to_string(), genome: format!("GENOME{}", fitness), fitness, generation, methylations: vec![0.5], synapse_weights: vec![0.25, 0.75] }
}

/// One row in every model table.
fn fill_model_tables(db: &Db) {
    db.insert_species(&NewSpecies { genome: "COL".into(), latin_name: "Name".into(), parent_name: None, generation: 1, carnivory: 0.0 }).unwrap();
    db.save_catalogue_entry(&NewCatalogueEntry {
        id: "c".into(),
        name: "n".into(),
        genome: "COL".into(),
        source: "src".into(),
        fitness: 0.0,
        carnivory: 0.0,
        methylations: vec![],
        synapse_weights: vec![],
    })
    .unwrap();
    db.save_trainer_generation("run", &[genome(1.0, 1)], Some(10)).unwrap();
    db.save_trainer_run_state(
        "run",
        &TrainerRunStateRow { scenario: "standard".into(), level: 0.0, level_auto: true, layout_id: 0, mutation_ops_max: 3, population: None, seed: None },
    )
    .unwrap();
    db.save_ocean_snapshot("{}").unwrap();
}

fn row_counts(conn: &Connection) -> Vec<i64> {
    model_tables().iter().map(|t| conn.query_row(&format!("SELECT COUNT(*) FROM {}", t), [], |r| r.get(0)).unwrap()).collect()
}

/// Columns (name, type, notnull, default, pk) of every table and the index names: the comparable schema.
fn schema_shape(conn: &Connection) -> Vec<String> {
    let mut shape = Vec::new();
    let mut stmt = conn.prepare("SELECT type, name FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name").unwrap();
    let objects: Vec<(String, String)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(|r| r.unwrap()).collect();
    for (kind, name) in objects {
        shape.push(format!("{} {}", kind, name));
        if kind == "table" {
            let mut info = conn.prepare(&format!("PRAGMA table_info({})", name)).unwrap();
            let mut columns: Vec<String> = info
                .query_map([], |r| Ok(format!("  {} {} notnull={} default={:?} pk={}", r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, i64>(5)?)))
                .unwrap()
                .map(|r| r.unwrap())
                .collect();
            columns.sort();
            shape.extend(columns);
        }
    }
    shape
}

/// The schema `init_db` of v1.12.30 (user_version 0) left, built with its exact statements, model version 6 stored.
/// `old_columns` builds the older shapes the removed column checks handled (trainer_genomes without
/// methylations/synapse_weights, trainer_run_state of v1.12.23 without population/seed).
fn build_v1_12_30_schema(conn: &Connection, old_columns: bool) {
    conn.execute_batch(
        "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO meta (key, value) VALUES ('model_version', '6');
         CREATE TABLE sessions (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, config_json TEXT NOT NULL);
         CREATE TABLE species_records (id TEXT PRIMARY KEY, latin_name TEXT NOT NULL UNIQUE, genome_string TEXT NOT NULL, parent_name TEXT,
             status TEXT CHECK(status IN ('alive', 'extinct')) DEFAULT 'alive', peak_population INTEGER DEFAULT 1, birth_time INTEGER NOT NULL,
             extinction_time INTEGER, generation INTEGER DEFAULT 1, carnivory REAL NOT NULL, registered_at DATETIME DEFAULT CURRENT_TIMESTAMP);
         CREATE TABLE simulation_history (id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL, generation INTEGER NOT NULL,
             average_fitness REAL NOT NULL, max_fitness REAL NOT NULL, population_count INTEGER NOT NULL, timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
             FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE);
         CREATE INDEX idx_history_session ON simulation_history(session_id);
         CREATE TABLE simulation_state (key TEXT PRIMARY KEY, state_json TEXT NOT NULL, updated_at INTEGER NOT NULL);
         CREATE TABLE creature_catalogue (id TEXT PRIMARY KEY, name TEXT NOT NULL, genome TEXT NOT NULL, source TEXT NOT NULL,
             fitness REAL NOT NULL DEFAULT 0.0, carnivory REAL NOT NULL DEFAULT 0.0, methylations TEXT NOT NULL, synapse_weights TEXT NOT NULL,
             created_at DATETIME DEFAULT CURRENT_TIMESTAMP);",
    )
    .unwrap();
    if old_columns {
        conn.execute_batch(
            "CREATE TABLE trainer_genomes (id TEXT PRIMARY KEY, run_id TEXT NOT NULL, generation INTEGER NOT NULL, name TEXT NOT NULL,
                 genome TEXT NOT NULL, fitness REAL NOT NULL, active INTEGER DEFAULT 1, created_at INTEGER NOT NULL);
             CREATE TABLE trainer_run_state (run_id TEXT PRIMARY KEY, scenario TEXT NOT NULL, level REAL NOT NULL, level_auto INTEGER NOT NULL,
                 layout_id TEXT NOT NULL, mutation_ops_max INTEGER NOT NULL, updated_at INTEGER NOT NULL);",
        )
        .unwrap();
    } else {
        conn.execute_batch(
            "CREATE TABLE trainer_genomes (id TEXT PRIMARY KEY, run_id TEXT NOT NULL, generation INTEGER NOT NULL, name TEXT NOT NULL,
                 genome TEXT NOT NULL, fitness REAL NOT NULL, active INTEGER DEFAULT 1, created_at INTEGER NOT NULL,
                 methylations TEXT NOT NULL DEFAULT '[]', synapse_weights TEXT NOT NULL DEFAULT '[]');
             CREATE TABLE trainer_run_state (run_id TEXT PRIMARY KEY, scenario TEXT NOT NULL, level REAL NOT NULL, level_auto INTEGER NOT NULL,
                 layout_id TEXT NOT NULL, mutation_ops_max INTEGER NOT NULL, updated_at INTEGER NOT NULL, population INTEGER, seed TEXT);",
        )
        .unwrap();
    }
    conn.execute_batch("CREATE INDEX idx_trainer_run ON trainer_genomes(run_id);").unwrap();
}

/// Rows of v1.12.30 in every table (written with that version's statements).
fn fill_v1_12_30_rows(conn: &Connection, old_columns: bool) {
    conn.execute_batch(
        "INSERT INTO sessions (id, name, config_json) VALUES ('desktop-session-001', 'Tauri Desktop Laboratory Studio', '{}');
         INSERT INTO simulation_history (session_id, generation, average_fitness, max_fitness, population_count) VALUES ('desktop-session-001', 3, 1.5, 2.5, 20);
         INSERT INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory)
             VALUES ('GENOME-A', 'Aqueus pulsa', 'GENOME-A', NULL, 'alive', 4, 1700000000, 1, 0.25);
         INSERT INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, extinction_time, generation, carnivory)
             VALUES ('GENOME-B', 'Velox minor', 'GENOME-B', 'Aqueus pulsa', 'extinct', 2, 1700000100, 1700000200, 2, 0.5);
         INSERT INTO simulation_state (key, state_json, updated_at) VALUES ('current_state', '{\"creatures\":[],\"highestGeneration\":7}', 1700000000000);
         INSERT INTO creature_catalogue (id, name, genome, source, fitness, carnivory, methylations, synapse_weights, created_at)
             VALUES ('cryo-123', 'Super Predator', 'COLOOOEN', 'Trainer FastRun', 12.5, 0.25, '[0.5,0.0,0.9]', '[1.5,-2.0,0.4]', '2026-10-01 12:00:00');",
    )
    .unwrap();
    if old_columns {
        conn.execute_batch(
            "INSERT INTO trainer_genomes (id, run_id, generation, name, genome, fitness, active, created_at) VALUES ('run-5-0-1700000000', 'run', 5, 'elite', 'GEN5', 9.5, 1, 1700000000);
             INSERT INTO trainer_run_state VALUES ('run', 'curriculum', 0.3, 1, '5', 2, 0);",
        )
        .unwrap();
    } else {
        conn.execute_batch(
            "INSERT INTO trainer_genomes (id, run_id, generation, name, genome, fitness, active, created_at, methylations, synapse_weights)
                 VALUES ('run-5-0-1700000000', 'run', 5, 'elite', 'GEN5', 9.5, 1, 1700000000, '[0.5]', '[0.25,0.75]');
             INSERT INTO trainer_run_state (run_id, scenario, level, level_auto, layout_id, mutation_ops_max, updated_at, population, seed)
                 VALUES ('run', 'curriculum', 0.3, 1, '18446744073709551614', 2, 0, 64, '18446744073709551613');",
        )
        .unwrap();
    }
}

/// TCK-159: a save game of v1.12.30 (user_version 0, same model version) keeps every row of the tables the app reads
/// through the migration; `simulation_history` and `sessions` (no reader) are dropped; the schema afterwards equals a
/// fresh one.
#[test]
fn test_v1_12_30_database_migrates_without_data_loss() {
    for old_columns in [false, true] {
        let dir = temp_dir(&format!("migrate-v1-12-30-{}", old_columns));
        let path = dir.join("test.db");
        {
            let conn = Connection::open(&path).unwrap();
            build_v1_12_30_schema(&conn, old_columns);
            fill_v1_12_30_rows(&conn, old_columns);
            assert_eq!(user_version(&conn), 0);
        }

        let db = init_db(&path).unwrap();
        assert_eq!(user_version(db.conn()), latest_version());
        assert_eq!(model_version(db.conn()), Some(MODEL_VERSION.to_string()));
        assert!(!dir.read_dir().unwrap().any(|e| e.unwrap().file_name().to_string_lossy().contains(".bak.")), "no model reset, no backup");

        let species = db.list_species().unwrap();
        assert_eq!(species.len(), 2);
        assert_eq!((species[0].id.as_str(), species[0].latin_name.as_str(), species[0].status, species[0].peak_population, species[0].birth_time), ("GENOME-A", "Aqueus pulsa", SpeciesStatus::Alive, 4, 1700000000));
        assert_eq!((species[1].status, species[1].parent_name.as_deref(), species[1].extinction_time, species[1].generation, species[1].carnivory), (SpeciesStatus::Extinct, Some("Aqueus pulsa"), Some(1700000200), 2, 0.5));

        let catalogue = db.list_catalogue().unwrap();
        assert_eq!(catalogue.len(), 1);
        assert_eq!((catalogue[0].name.as_str(), catalogue[0].fitness, catalogue[0].created_at.as_str()), ("Super Predator", 12.5, "2026-10-01 12:00:00"));
        assert_eq!((catalogue[0].methylations.clone(), catalogue[0].synapse_weights.clone()), (vec![0.5, 0.0, 0.9], vec![1.5, -2.0, 0.4]));

        assert_eq!(db.ocean_snapshot().unwrap().as_deref(), Some("{\"creatures\":[],\"highestGeneration\":7}"));

        let parents = db.trainer_genomes("run", true, 10).unwrap();
        assert_eq!(parents.len(), 1);
        assert_eq!((parents[0].genome.as_str(), parents[0].fitness, parents[0].generation), ("GEN5", 9.5, 5));
        let state = db.trainer_run_state("run").unwrap().unwrap();
        if old_columns {
            assert_eq!((parents[0].methylations.clone(), parents[0].synapse_weights.clone()), (vec![], vec![]), "added columns default to []");
            assert_eq!((state.scenario.as_str(), state.layout_id, state.mutation_ops_max, state.population, state.seed), ("curriculum", 5, 2, None, None));
        } else {
            assert_eq!((parents[0].methylations.clone(), parents[0].synapse_weights.clone()), (vec![0.5], vec![0.25, 0.75]));
            assert_eq!((state.layout_id, state.population, state.seed), (u64::MAX - 1, Some(64), Some(u64::MAX - 2)));
        }

        for dropped in ["simulation_history", "sessions"] {
            let exists: bool = db.conn().query_row("SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE name = ?1)", params![dropped], |r| r.get(0)).unwrap();
            assert!(!exists, "{} dropped", dropped);
        }
        assert_eq!(schema_shape(db.conn()), schema_shape(init_db(":memory:").unwrap().conn()), "migrated schema equals a fresh one");

        // Reopening runs no migration again and keeps everything
        drop(db);
        let db = init_db(&path).unwrap();
        assert_eq!((db.list_species().unwrap().len(), db.list_catalogue().unwrap().len(), db.trainer_genomes("run", false, -1).unwrap().len()), (2, 1, 1));
        drop(db);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// The latest table definitions (`TABLES`, used after a model reset) and the migration chain (used for new and old
/// databases) must describe the same schema.
#[test]
fn test_migrated_schema_equals_latest_tables() {
    let migrated = init_db(":memory:").unwrap();
    let latest = Connection::open_in_memory().unwrap();
    for table in tables() {
        for sql in table.create {
            latest.execute(sql, []).unwrap();
        }
    }
    assert_eq!(schema_shape(migrated.conn()), schema_shape(&latest));
    // Versions are strictly increasing from 1
    for (i, m) in MIGRATIONS.iter().enumerate() {
        assert_eq!(m.version, i as u32 + 1, "migration '{}'", m.name);
    }
}

#[test]
fn test_newer_schema_version_is_left_alone() {
    let dir = temp_dir("newer-schema");
    let path = dir.join("test.db");
    init_db(&path).unwrap().conn().pragma_update(None, "user_version", latest_version() + 1).unwrap();
    let db = init_db(&path).unwrap();
    assert_eq!(user_version(db.conn()), latest_version() + 1);
    fill_model_tables(&db);
    drop(db);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_file_db_uses_wal_journal() {
    let dir = temp_dir("wal");
    let db = init_db(dir.join("test.db")).unwrap();
    let mode: String = db.conn().query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap();
    assert_eq!(mode, "wal");
    drop(db);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_read_only_store_reads_and_refuses_writes() {
    let dir = temp_dir("read-only");
    let path = dir.join("test.db");
    fill_model_tables(&init_db(&path).unwrap());
    let db = open_read_only(&path).unwrap();
    assert_eq!(db.ranked_trainer_genomes().unwrap().len(), 1);
    assert!(db.save_ocean_snapshot("{}").is_err());
    drop(db);
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

#[test]
fn test_fresh_db_stores_model_and_schema_version() {
    let db = init_db(":memory:").unwrap();
    assert_eq!(model_version(db.conn()), Some(MODEL_VERSION.to_string()));
    assert_eq!(user_version(db.conn()), latest_version());
    assert_eq!(row_counts(db.conn()), vec![0; model_tables().len()]);
}

/// TCK-195: a model-version reset first copies the whole DB to `<db>.model-v<old>.bak.db`; an existing backup of
/// the same old version is never overwritten; no reset, no backup.
#[test]
fn test_model_reset_backs_up_the_old_data_first() {
    let dir = temp_dir("model-backup");
    let path = dir.join("test.db");
    let backup = dir.join("test.db.model-v1.bak.db");
    let full = vec![1; model_tables().len()];
    fill_model_tables(&init_db(&path).unwrap());
    drop(init_db(&path).unwrap());
    assert!(!backup.exists(), "same version: no backup");

    init_db(&path).unwrap().conn().execute("UPDATE meta SET value = '1' WHERE key = 'model_version'", []).unwrap();
    let db = init_db(&path).unwrap();
    assert_eq!(row_counts(db.conn()), vec![0; model_tables().len()], "model tables reset");
    let old = Connection::open(&backup).unwrap();
    assert_eq!(row_counts(&old), full, "backup holds the old data");
    assert_eq!(model_version(&old), Some("1".to_string()));
    drop(old);

    // A second reset from the same old version keeps the first backup
    fill_model_tables(&db);
    db.conn().execute("UPDATE meta SET value = '1' WHERE key = 'model_version'", []).unwrap();
    db.clear_species().unwrap();
    drop(db);
    drop(init_db(&path).unwrap());
    assert_eq!(row_counts(&Connection::open(&backup).unwrap()), full);
    assert_eq!(std::fs::read_dir(&dir).unwrap().filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().contains(".bak.")).count(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_file_db_model_version_reset_on_reopen() {
    let dir = temp_dir("model-version");
    let path = dir.join("test.db");
    let full = vec![1; model_tables().len()];
    let empty = vec![0; model_tables().len()];
    fill_model_tables(&init_db(&path).unwrap());

    // Same version: reopening keeps every row
    assert_eq!(row_counts(init_db(&path).unwrap().conn()), full);

    // Stored version differs: model tables recreated empty, version updated, meta kept
    init_db(&path).unwrap().conn().execute("UPDATE meta SET value = '1' WHERE key = 'model_version'", []).unwrap();
    let db = init_db(&path).unwrap();
    assert_eq!(row_counts(db.conn()), empty);
    assert_eq!(model_version(db.conn()), Some(MODEL_VERSION.to_string()));
    assert_eq!(user_version(db.conn()), latest_version());
    fill_model_tables(&db);

    // Pre-versioning database (model tables, no stored version): reset as well
    db.conn().execute("DELETE FROM meta", []).unwrap();
    drop(db);
    let db = init_db(&path).unwrap();
    assert_eq!(row_counts(db.conn()), empty);
    assert_eq!(model_version(db.conn()), Some(MODEL_VERSION.to_string()));
    drop(db);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A model reset of a v1.12.30 database (user_version 0, old model version): the migration runs, then the model
/// tables are recreated empty in the latest shape.
#[test]
fn test_old_model_and_schema_version_together() {
    let dir = temp_dir("old-model-and-schema");
    let path = dir.join("test.db");
    {
        let conn = Connection::open(&path).unwrap();
        build_v1_12_30_schema(&conn, true);
        fill_v1_12_30_rows(&conn, true);
        conn.execute("UPDATE meta SET value = '5' WHERE key = 'model_version'", []).unwrap();
    }
    let db = init_db(&path).unwrap();
    assert_eq!(row_counts(db.conn()), vec![0; model_tables().len()]);
    assert_eq!((user_version(db.conn()), model_version(db.conn())), (latest_version(), Some(MODEL_VERSION.to_string())));
    assert_eq!(schema_shape(db.conn()), schema_shape(init_db(":memory:").unwrap().conn()));
    // The backup is the file as v1.12.30 left it (taken before the migration)
    let old = Connection::open(dir.join("test.db.model-v5.bak.db")).unwrap();
    assert_eq!(user_version(&old), 0);
    assert_eq!(old.query_row("SELECT COUNT(*) FROM simulation_history", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
    drop((db, old));
    let _ = std::fs::remove_dir_all(&dir);
}

/// TCK-159 reproduction: trainer genome ids were `<run>-<gen>-<idx>-<secs>`, so a second save of the same generation
/// within one second failed on the primary key and lost its elites (logged only).
#[test]
fn test_trainer_genome_ids_never_collide_within_one_second() {
    let db = init_db(":memory:").unwrap();
    db.save_trainer_generation("run", &[genome(1.0, 1)], Some(10)).unwrap();
    db.save_trainer_generation("run", &[genome(2.0, 1)], Some(10)).unwrap();
    db.add_trainer_genome("run", &genome(3.0, 1)).unwrap();
    db.add_trainer_genome("run", &genome(4.0, 1)).unwrap();
    assert_eq!(db.trainer_genomes("run", false, 100).unwrap().len(), 4);
}

// ==========================================================================
// Contract suite: the SQLite store and the in-memory store behave the same
// ==========================================================================

mod contract {
    use super::*;

    fn species(genome: &str, name: &str) -> NewSpecies {
        NewSpecies { genome: genome.into(), latin_name: name.into(), parent_name: Some("Parent".into()), generation: 2, carnivory: 0.5 }
    }

    fn species_ledger(repo: &impl SpeciesRepo) {
        repo.insert_species(&species("G1", "One")).unwrap();
        repo.insert_species(&species("G2", "Two")).unwrap();
        repo.insert_species(&species("G1", "Other name")).unwrap(); // same genome: ignored
        repo.insert_species(&species("G3", "Two")).unwrap(); // same latin name: ignored
        let list = repo.list_species().unwrap();
        assert_eq!(list.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), vec!["G1", "G2"]);
        assert_eq!((list[0].status, list[0].peak_population, list[0].generation, list[0].parent_name.as_deref(), list[0].extinction_time), (SpeciesStatus::Alive, 1, 2, Some("Parent"), None));
        assert!(list[0].birth_time > 1_600_000_000, "birth time in Unix seconds");

        repo.set_species_peak("G1", 7).unwrap();
        repo.mark_species_extinct("G2").unwrap();
        let states = repo.species_states().unwrap();
        assert_eq!(states, vec![SpeciesState { id: "G1".into(), status: SpeciesStatus::Alive, peak_population: 7 }, SpeciesState { id: "G2".into(), status: SpeciesStatus::Extinct, peak_population: 1 }]);
        assert!(repo.list_species().unwrap()[1].extinction_time.is_some());
        repo.mark_species_alive("G2").unwrap();
        assert_eq!(repo.list_species().unwrap()[1].extinction_time, None);
        assert_eq!(repo.species_name("G2").unwrap().as_deref(), Some("Two"));
        assert_eq!(repo.species_name("missing").unwrap(), None);

        // Retention: alive species stay, only the newest `keep` extinct ones are kept
        for i in 0..5 {
            repo.insert_species(&species(&format!("X{}", i), &format!("Extinct {}", i))).unwrap();
            repo.mark_species_extinct(&format!("X{}", i)).unwrap();
        }
        assert_eq!(repo.prune_extinct_species(2).unwrap(), 3);
        let ids: Vec<String> = repo.list_species().unwrap().into_iter().map(|s| s.id).collect();
        assert_eq!(ids, vec!["G1", "G2", "X3", "X4"], "same extinction second: the later insert counts as newer");
        assert_eq!(repo.prune_extinct_species(2).unwrap(), 0);

        repo.clear_species().unwrap();
        assert!(repo.list_species().unwrap().is_empty());
    }

    #[test]
    fn test_species_repo_contract() {
        species_ledger(&init_db(":memory:").unwrap());
        species_ledger(&MemoryDb::new());
    }

    fn entry(id: &str, name: &str) -> NewCatalogueEntry {
        NewCatalogueEntry {
            id: id.into(),
            name: name.into(),
            genome: "COLOOOEN".into(),
            source: "Trainer FastRun".into(),
            fitness: 1250.5,
            carnivory: 0.25,
            methylations: vec![0.5, 0.0, 0.9],
            synapse_weights: vec![1.5, -2.0, 0.4],
        }
    }

    fn catalogue(repo: &impl CatalogueRepo) {
        repo.save_catalogue_entry(&entry("cryo-1", "Aqueus pulsa")).unwrap();
        repo.save_catalogue_entry(&entry("cryo-2", "Second")).unwrap();
        let one = repo.catalogue_entry("cryo-1").unwrap().unwrap();
        assert_eq!((one.name.as_str(), one.fitness, one.carnivory, one.methylations.clone(), one.synapse_weights.clone()), ("Aqueus pulsa", 1250.5, 0.25, vec![0.5, 0.0, 0.9], vec![1.5, -2.0, 0.4]));
        assert_eq!(one.created_at.len(), "YYYY-MM-DD HH:MM:SS".len());
        assert_eq!(repo.list_catalogue().unwrap().len(), 2);

        repo.rename_catalogue_entry("cryo-1", "Super Predator").unwrap();
        assert_eq!(repo.catalogue_entry("cryo-1").unwrap().unwrap().name, "Super Predator");
        repo.save_catalogue_entry(&entry("cryo-1", "Replaced")).unwrap();
        assert_eq!((repo.list_catalogue().unwrap().len(), repo.catalogue_entry("cryo-1").unwrap().unwrap().name), (2, "Replaced".to_string()));

        repo.delete_catalogue_entry("cryo-1").unwrap();
        assert_eq!(repo.catalogue_entry("cryo-1").unwrap(), None);
        assert_eq!(repo.list_catalogue().unwrap().len(), 1);
    }

    #[test]
    fn test_catalogue_repo_contract() {
        catalogue(&init_db(":memory:").unwrap());
        catalogue(&MemoryDb::new());
    }

    fn trainer(repo: &impl TrainerRepo) {
        repo.save_trainer_generation("b", &[genome(5.0, 1), genome(3.0, 1)], None).unwrap();
        repo.save_trainer_generation("b", &[genome(4.0, 2)], None).unwrap();
        repo.save_trainer_generation("a", &[genome(1.0, 9)], None).unwrap();
        assert_eq!(repo.trainer_genomes("b", true, -1).unwrap().iter().map(|g| g.fitness).collect::<Vec<_>>(), vec![4.0]);
        assert_eq!(repo.trainer_genomes("b", false, -1).unwrap().iter().map(|g| g.fitness).collect::<Vec<_>>(), vec![5.0, 4.0, 3.0]);
        assert_eq!(repo.trainer_genomes("b", false, 2).unwrap().len(), 2);
        assert_eq!(
            repo.trainer_runs().unwrap(),
            vec![TrainerRunSummary { run_id: "a".into(), max_generation: 9, max_fitness: 1.0 }, TrainerRunSummary { run_id: "b".into(), max_generation: 2, max_fitness: 5.0 }]
        );

        // An assignment of the same slot in the same generation replaces the previous one; it is an active parent
        repo.assign_trainer_genome("b", 3, &genome(0.0, 2)).unwrap();
        repo.assign_trainer_genome("b", 3, &genome(0.5, 2)).unwrap();
        repo.add_trainer_genome("b", &genome(2.0, 2)).unwrap();
        assert_eq!(repo.trainer_genomes("b", true, -1).unwrap().iter().map(|g| g.fitness).collect::<Vec<_>>(), vec![4.0, 2.0, 0.5]);

        // Retention keeps the best N plus every active row
        repo.prune_trainer_genomes("b", 1).unwrap();
        assert_eq!(repo.trainer_genomes("b", false, -1).unwrap().iter().map(|g| g.fitness).collect::<Vec<_>>(), vec![5.0, 4.0, 2.0, 0.5]);

        let ranked = repo.ranked_trainer_genomes().unwrap();
        assert_eq!(ranked.iter().map(|g| (g.run_id.as_str(), g.fitness)).collect::<Vec<_>>(), vec![("b", 5.0), ("b", 4.0), ("b", 2.0), ("a", 1.0), ("b", 0.5)]);
        assert_eq!(ranked[3].generation, 9);

        repo.delete_trainer_run("b").unwrap();
        assert!(repo.trainer_genomes("b", false, -1).unwrap().is_empty());
        assert_eq!(repo.trainer_runs().unwrap().len(), 1);
    }

    #[test]
    fn test_trainer_repo_contract() {
        trainer(&init_db(":memory:").unwrap());
        trainer(&MemoryDb::new());
    }

    fn ocean(repo: &impl OceanSnapshotRepo) {
        assert_eq!(repo.ocean_snapshot().unwrap(), None);
        repo.save_ocean_snapshot("{\"a\":1}").unwrap();
        repo.save_ocean_snapshot("{\"a\":2}").unwrap();
        repo.save_progenitor_genome("COL").unwrap();
        assert_eq!(repo.ocean_snapshot().unwrap().as_deref(), Some("{\"a\":2}"));
        repo.clear_ocean_state().unwrap();
        assert_eq!(repo.ocean_snapshot().unwrap(), None);
    }

    #[test]
    fn test_ocean_snapshot_repo_contract() {
        ocean(&init_db(":memory:").unwrap());
        ocean(&MemoryDb::new());
    }
}
