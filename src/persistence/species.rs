//! `species_records`: tables, migration steps and the SQLite `SpeciesRepo`.

use rusqlite::{params, Connection, OptionalExtension, Result};

use super::dto::*;
use super::repos::SpeciesRepo;
use super::schema::TableDef;
use super::{collect_rows, Db};
use crate::shared::time;

pub(super) const TABLES: &[TableDef] = &[TableDef {
    name: "species_records",
    model_scoped: true,
    create: &["CREATE TABLE IF NOT EXISTS species_records (
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
        );"],
}];

/// Migration 1 (frozen): the table as of v1.12.30.
pub(super) fn v1_baseline(conn: &Connection) -> Result<()> {
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
    Ok(())
}

fn status_of(text: String) -> Result<SpeciesStatus> {
    SpeciesStatus::parse(&text).ok_or_else(|| rusqlite::Error::InvalidColumnType(0, format!("status '{}'", text), rusqlite::types::Type::Text))
}

impl SpeciesRepo for Db {
    fn insert_species(&self, s: &NewSpecies) -> RepoResult<()> {
        self.conn().execute(
            "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, ?6, ?7)",
            params![s.genome, s.latin_name, s.genome, s.parent_name, time::now_secs(), s.generation, s.carnivory],
        )?;
        Ok(())
    }

    fn list_species(&self) -> RepoResult<Vec<SpeciesRecord>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, latin_name, genome_string, parent_name, status, peak_population, birth_time, extinction_time, generation, carnivory FROM species_records",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SpeciesRecord {
                id: row.get(0)?,
                latin_name: row.get(1)?,
                genome: row.get(2)?,
                parent_name: row.get(3)?,
                status: status_of(row.get(4)?)?,
                peak_population: row.get(5)?,
                birth_time: row.get(6)?,
                extinction_time: row.get(7)?,
                generation: row.get(8)?,
                carnivory: row.get(9)?,
            })
        })?;
        Ok(collect_rows(rows, "species_records"))
    }

    fn species_states(&self) -> RepoResult<Vec<SpeciesState>> {
        let mut stmt = self.conn().prepare("SELECT id, status, peak_population FROM species_records")?;
        let rows = stmt.query_map([], |row| Ok(SpeciesState { id: row.get(0)?, status: status_of(row.get(1)?)?, peak_population: row.get(2)? }))?;
        Ok(collect_rows(rows, "species_records"))
    }

    fn species_name(&self, id: &str) -> RepoResult<Option<String>> {
        Ok(self.conn().query_row("SELECT latin_name FROM species_records WHERE id = ?1", params![id], |r| r.get(0)).optional()?)
    }

    fn mark_species_alive(&self, id: &str) -> RepoResult<()> {
        self.conn().execute("UPDATE species_records SET status = 'alive', extinction_time = NULL WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn mark_species_extinct(&self, id: &str) -> RepoResult<()> {
        self.conn().execute("UPDATE species_records SET status = 'extinct', extinction_time = ?1 WHERE id = ?2", params![time::now_secs(), id])?;
        Ok(())
    }

    fn set_species_peak(&self, id: &str, peak: i32) -> RepoResult<()> {
        self.conn().execute("UPDATE species_records SET peak_population = ?1 WHERE id = ?2", params![peak, id])?;
        Ok(())
    }

    fn clear_species(&self) -> RepoResult<()> {
        self.conn().execute("DELETE FROM species_records", [])?;
        Ok(())
    }

    fn prune_extinct_species(&self, keep: usize) -> RepoResult<usize> {
        Ok(self.conn().execute(
            "DELETE FROM species_records WHERE status = 'extinct' AND id NOT IN
             (SELECT id FROM species_records WHERE status = 'extinct' ORDER BY extinction_time DESC, rowid DESC LIMIT ?1)",
            params![keep as i64],
        )?)
    }
}
