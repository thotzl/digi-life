//! `creature_catalogue`: tables, migration steps and the SQLite `CatalogueRepo`.

use rusqlite::{params, Connection, OptionalExtension, Result, Row};

use super::dto::*;
use super::repos::CatalogueRepo;
use super::schema::TableDef;
use super::{collect_rows, Db};
use crate::shared::json_col;

pub(super) const TABLES: &[TableDef] = &[TableDef {
    name: "creature_catalogue",
    model_scoped: true,
    create: &["CREATE TABLE IF NOT EXISTS creature_catalogue (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            genome TEXT NOT NULL,
            source TEXT NOT NULL,
            fitness REAL NOT NULL DEFAULT 0.0,
            carnivory REAL NOT NULL DEFAULT 0.0,
            methylations TEXT NOT NULL,
            synapse_weights TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"],
}];

/// Migration 1 (frozen): the table as of v1.12.30.
pub(super) fn v1_baseline(conn: &Connection) -> Result<()> {
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
    Ok(())
}

const COLUMNS: &str = "id, name, genome, source, fitness, carnivory, methylations, synapse_weights, created_at";

fn entry(row: &Row) -> Result<CatalogueEntry> {
    Ok(CatalogueEntry {
        id: row.get(0)?,
        name: row.get(1)?,
        genome: row.get(2)?,
        source: row.get(3)?,
        fitness: row.get(4)?,
        carnivory: row.get(5)?,
        methylations: json_col::decode(&row.get::<_, String>(6)?, "creature_catalogue.methylations").unwrap_or_default(),
        synapse_weights: json_col::decode(&row.get::<_, String>(7)?, "creature_catalogue.synapse_weights").unwrap_or_default(),
        created_at: row.get(8)?,
    })
}

impl CatalogueRepo for Db {
    fn list_catalogue(&self) -> RepoResult<Vec<CatalogueEntry>> {
        let mut stmt = self.conn().prepare(&format!("SELECT {} FROM creature_catalogue ORDER BY created_at DESC", COLUMNS))?;
        let rows = stmt.query_map([], entry)?;
        Ok(collect_rows(rows, "creature_catalogue"))
    }

    fn catalogue_entry(&self, id: &str) -> RepoResult<Option<CatalogueEntry>> {
        Ok(self.conn().query_row(&format!("SELECT {} FROM creature_catalogue WHERE id = ?1", COLUMNS), params![id], entry).optional()?)
    }

    fn save_catalogue_entry(&self, e: &NewCatalogueEntry) -> RepoResult<()> {
        self.conn().execute(
            "INSERT OR REPLACE INTO creature_catalogue (id, name, genome, source, fitness, carnivory, methylations, synapse_weights, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)",
            params![
                e.id,
                e.name,
                e.genome,
                e.source,
                e.fitness,
                e.carnivory,
                json_col::encode(&e.methylations, "creature_catalogue.methylations"),
                json_col::encode(&e.synapse_weights, "creature_catalogue.synapse_weights")
            ],
        )?;
        Ok(())
    }

    fn rename_catalogue_entry(&self, id: &str, name: &str) -> RepoResult<()> {
        self.conn().execute("UPDATE creature_catalogue SET name = ?1 WHERE id = ?2", params![name, id])?;
        Ok(())
    }

    fn delete_catalogue_entry(&self, id: &str) -> RepoResult<()> {
        self.conn().execute("DELETE FROM creature_catalogue WHERE id = ?1", params![id])?;
        Ok(())
    }
}
