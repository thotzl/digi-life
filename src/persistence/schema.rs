//! Schema versioning (TCK-159) and the model-version reset (TCK-183, TCK-195).
//!
//! - **Schema version** = `PRAGMA user_version`. `MIGRATIONS` is the ordered list; `init_db` applies every migration
//!   above the stored version in one transaction and stores the new version. A database of v1.12.30 or older has
//!   version 0 and runs all of them; a new database too (migration 1 creates the tables). Migrations are history:
//!   never edit a released one, append a new one. Each repository module contributes its steps and its latest table
//!   definitions (`TABLES`); `test_migrated_schema_equals_latest_tables` keeps both in step.
//! - **Model version** = `meta.model_version` vs `MODEL_VERSION`: rows of model-scoped tables were made by an older
//!   genome/brain model and are dropped (no migration), after a backup copy of the file.

use rusqlite::{Connection, OptionalExtension, Result, params};

use super::{catalogue, ocean, species, trainer};

/// Version of the genome/brain model the stored genomes and weights were made with (TCK-183). Bump it on every change
/// that makes stored synapse weights or phenotypes invalid: `init_db` then recreates the model tables empty.
/// 2 = Brain v2 (TCK-171 P3: role keys, gene identity by promoter spelling, weight tapes).
/// 3 = Oja learning with fixed reflexes (TCK-152: `plastic` flag in stored phenotypes; weights learned under the
///     old rule, incl. drifted reflexes, are not valid under the new one).
/// 4 = Energy v2 (TCK-151/178: `basalMetabolicRate` removed from stored phenotypes; trainer fitness and weights were
///     selected and learned without an energy budget).
/// 5 = Fitness in catch units (TCK-175: stored trainer and catalogue fitness of the old formula, ~10^2..10^3, would
///     outrank every new value in the hall of fame, retention and run list, which all sort by raw fitness).
/// 6 = Thalamus sum nodes with tanh instead of sigmoid (TCK-190: stored weights were selected and learned with a constant
///     0.5 drive from every silent channel).
/// 7 = R5 model corrections (TCK-202: thermal tolerance, predator flag and names from one diet class, eyes no longer
///     limbs, one eat radius / food drift / clock / temperature model for trainer and Ocean; stored phenotypes and
///     weights were selected under the old rules).
pub const MODEL_VERSION: u32 = 7;

/// Latest definition of one table, owned by its repository module.
pub(super) struct TableDef {
    pub name: &'static str,
    /// Rows depend on `MODEL_VERSION`: dropped and recreated empty on a model change.
    pub model_scoped: bool,
    /// `CREATE TABLE IF NOT EXISTS` plus its indexes, in the latest shape.
    pub create: &'static [&'static str],
}

/// One schema migration: its steps run in order inside the migration transaction.
pub(super) struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub steps: &'static [fn(&Connection) -> Result<()>],
}

/// Ordered schema migrations; the last version is the current schema version.
pub(super) const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "baseline: tables of v1.12.30 incl. the columns added before versioning",
        steps: &[
            meta_v1,
            ocean::v1_baseline,
            species::v1_baseline,
            trainer::v1_baseline,
            catalogue::v1_baseline,
        ],
    },
    Migration {
        version: 2,
        name: "drop simulation_history and sessions (no reader, RV-D03/RV-D06)",
        steps: &[ocean::v2_drop_history_and_sessions],
    },
];

const META: &[TableDef] = &[TableDef {
    name: "meta",
    model_scoped: false,
    create: &["CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);"],
}];

/// The latest table definitions of every module.
pub(super) fn tables() -> impl Iterator<Item = &'static TableDef> {
    META.iter()
        .chain(species::TABLES)
        .chain(catalogue::TABLES)
        .chain(trainer::TABLES)
        .chain(ocean::TABLES)
}

pub(super) fn model_tables() -> Vec<&'static str> {
    tables().filter(|t| t.model_scoped).map(|t| t.name).collect()
}

pub(super) fn latest_version() -> u32 {
    MIGRATIONS.last().map_or(0, |m| m.version)
}

fn meta_v1(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        [],
    )?;
    Ok(())
}

/// Brings the schema to the latest version, then the model tables to `MODEL_VERSION`, then (re)creates missing tables.
pub(super) fn prepare(conn: &Connection) -> Result<()> {
    let model_reset = model_reset_needed(conn)?;
    if let Some(old) = &model_reset {
        backup_before_model_reset(conn, old);
    }
    migrate(conn)?;
    store_model_version(conn, model_reset.as_deref())?;
    ensure_tables(conn)
}

/// Applies every migration above `user_version` in one transaction. A version newer than this build is logged and
/// left alone (an older app opening a newer save game).
pub(super) fn migrate(conn: &Connection) -> Result<()> {
    let current: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let latest = latest_version();
    if current > latest {
        log::warn!(
            "database schema version {} is newer than this build ({}); no migration",
            current,
            latest
        );
        return Ok(());
    }
    if current == latest {
        return Ok(());
    }
    let tx = conn.unchecked_transaction()?;
    for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
        for step in migration.steps {
            step(&tx)?;
        }
        log::info!("schema migration {}: {}", migration.version, migration.name);
    }
    tx.pragma_update(None, "user_version", latest)?;
    tx.commit()
}

/// `CREATE ... IF NOT EXISTS` of every latest table (recreates the model tables after a reset).
fn ensure_tables(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    for table in tables() {
        for sql in table.create {
            tx.execute(sql, [])?;
        }
    }
    tx.commit()
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        params![name],
        |r| r.get(0),
    )
}

/// Model-version rule (TCK-183), decided before any migration: a stored `model_version` other than `MODEL_VERSION`,
/// or none in a database that already has model tables, resets the model tables. Returns the old version
/// (`"none"` if missing) when a reset is due.
fn model_reset_needed(conn: &Connection) -> Result<Option<String>> {
    let stored: Option<String> = if table_exists(conn, "meta")? {
        conn.query_row("SELECT value FROM meta WHERE key = 'model_version'", [], |row| {
            row.get(0)
        })
        .optional()?
    } else {
        None
    };
    if stored.as_deref() == Some(MODEL_VERSION.to_string().as_str()) {
        return Ok(None);
    }
    let mut has_model_tables = false;
    for table in model_tables() {
        has_model_tables |= table_exists(conn, table)?;
    }
    Ok((stored.is_some() || has_model_tables).then(|| stored.unwrap_or_else(|| "none".to_string())))
}

/// Drops the model tables if a reset is due (`ensure_tables` recreates them) and stores `MODEL_VERSION`.
fn store_model_version(conn: &Connection, reset_from: Option<&str>) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    if reset_from.is_some() {
        for table in model_tables() {
            tx.execute(&format!("DROP TABLE IF EXISTS {}", table), [])?;
        }
    }
    tx.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('model_version', ?1)",
        params![MODEL_VERSION.to_string()],
    )?;
    tx.commit()?;
    if let Some(old) = reset_from {
        log::info!("model version {} -> {}: model tables reset", old, MODEL_VERSION);
    }
    Ok(())
}

/// Save-game backup before a model reset (TCK-195): a consistent copy of the whole DB (`VACUUM INTO`, also with WAL)
/// at `<db>.model-v<old>.bak.db`, at most one per old version (an existing one is kept). Taken before the schema
/// migrations, so it is the file as the old build left it. In-memory DBs are skipped. A failed backup is logged and
/// does not block the start.
fn backup_before_model_reset(conn: &Connection, old_version: &str) {
    let Some(db_path) = conn.path().filter(|p| !p.is_empty()) else {
        return;
    };
    let backup = std::path::PathBuf::from(format!("{}.model-v{}.bak.db", db_path, old_version));
    if backup.exists() {
        return;
    }
    match conn.execute("VACUUM INTO ?1", params![backup.to_string_lossy()]) {
        Ok(_) => log::info!("backup before the model reset: '{}'", backup.display()),
        Err(e) => log::error!("backup before the model reset failed ({}); resetting anyway", e),
    }
}
