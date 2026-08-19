import Database from 'better-sqlite3';
import path from 'path';
import { SpeciesRecord } from '../shared/types';

const DB_PATH = path.resolve('./digilife.db');

// Instantiate connection
const db = new Database(DB_PATH);

// Configure WAL mode for fast concurrent operations
db.pragma('journal_mode = WAL');

// Initialize database schema
db.exec(`
  CREATE TABLE IF NOT EXISTS species (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    genome TEXT NOT NULL,
    antisense TEXT NOT NULL,
    parentSpeciesId TEXT,
    status TEXT CHECK(status IN ('alive', 'extinct')) DEFAULT 'alive',
    peakPopulation INTEGER DEFAULT 1,
    birthTime INTEGER NOT NULL,
    extinctionTime INTEGER,
    generation INTEGER DEFAULT 1,
    carnivory REAL NOT NULL
  );

  CREATE TABLE IF NOT EXISTS simulation_state (
    key TEXT PRIMARY KEY,
    state_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL
  );
`);

// Safe migration: check if run_id column exists, if not, drop table and upgrade
const tableInfo = db.pragma("table_info(trainer_genomes)") as any[];
const hasRunId = tableInfo.some(col => col.name === "run_id");
if (tableInfo.length > 0 && !hasRunId) {
  console.log("[SQLite] Upgrading trainer_genomes table to include run_id...");
  db.exec("DROP TABLE IF EXISTS trainer_genomes;");
}

db.exec(`
  CREATE TABLE IF NOT EXISTS trainer_genomes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    generation INTEGER NOT NULL,
    name TEXT NOT NULL,
    genome TEXT NOT NULL,
    fitness REAL NOT NULL,
    active INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL
  );
`);

/**
 * Reads all species records from SQLite.
 */
export function readDb(): SpeciesRecord[] {
  try {
    const rows = db.prepare('SELECT * FROM species').all();
    return rows as SpeciesRecord[];
  } catch (err) {
    console.error('[SQLite] Error reading species records:', err);
    return [];
  }
}

/**
 * Replaces or upserts species records inside a single secure SQL transaction.
 */
export function writeDb(records: SpeciesRecord[]) {
  const insertStmt = db.prepare(`
    INSERT INTO species (
      id, name, genome, antisense, parentSpeciesId, status, peakPopulation, birthTime, extinctionTime, generation, carnivory
    ) VALUES (
      @id, @name, @genome, @antisense, @parentSpeciesId, @status, @peakPopulation, @birthTime, @extinctionTime, @generation, @carnivory
    ) ON CONFLICT(id) DO UPDATE SET
      status = excluded.status,
      peakPopulation = MAX(species.peakPopulation, excluded.peakPopulation),
      extinctionTime = excluded.extinctionTime
  `);

  // Execute inside an isolated transaction
  const transaction = db.transaction((list: SpeciesRecord[]) => {
    // If the input array is empty, we wipe the database (useful for RESET_EVOLUTION)
    if (list.length === 0) {
      db.prepare('DELETE FROM species').run();
      return;
    }
    
    for (const record of list) {
      insertStmt.run({
        ...record,
        parentSpeciesId: record.parentSpeciesId !== undefined ? record.parentSpeciesId : null,
        extinctionTime: record.extinctionTime !== undefined ? record.extinctionTime : null
      });
    }
  });

  try {
    transaction(records);
  } catch (err) {
    console.error('[SQLite] Transaction rollback. Failed to write species records:', err);
  }
}

/**
 * Reads the latest saved simulation state from SQLite.
 */
export function readState(): any {
  try {
    const row = db.prepare('SELECT state_json FROM simulation_state WHERE key = ?').get('current_state') as any;
    if (row && row.state_json) {
      return JSON.parse(row.state_json);
    }
    return null;
  } catch (err) {
    console.error('[SQLite] Error reading simulation state:', err);
    return null;
  }
}

/**
 * Saves the simulation state payload securely.
 */
export function writeState(state: any) {
  try {
    const payload = JSON.stringify(state);
    db.prepare('INSERT OR REPLACE INTO simulation_state (key, state_json, updated_at) VALUES (?, ?, ?)')
      .run('current_state', payload, Date.now());
  } catch (err) {
    console.error('[SQLite] Error saving simulation state:', err);
  }
}

/**
 * Clears the simulation state from SQLite.
 */
export function clearState() {
  try {
    db.prepare('DELETE FROM simulation_state WHERE key = ?').run('current_state');
  } catch (err) {
    console.error('[SQLite] Error clearing simulation state:', err);
  }
}

/**
 * Saves a generation's elite genomes to the separate trainer_genomes SQLite table under a specific run_id.
 * Sets all older active entries of this run to active=0 (soft-delete / fossils).
 */
export function saveTrainerGeneration(runId: string, generation: number, population: { name: string; genome: string; fitness: number }[]) {
  const deactivateStmt = db.prepare('UPDATE trainer_genomes SET active = 0 WHERE run_id = ? AND active = 1');
  const insertStmt = db.prepare(`
    INSERT INTO trainer_genomes (run_id, generation, name, genome, fitness, active, created_at)
    VALUES (?, ?, ?, ?, ?, 1, ?)
  `);

  const transaction = db.transaction((list: { name: string; genome: string; fitness: number }[]) => {
    // 1. Soft-delete previous active population for this run
    deactivateStmt.run(runId);

    // 2. Insert new elite list
    const now = Date.now();
    for (const item of list) {
      insertStmt.run(runId, generation, item.name, item.genome, item.fitness, now);
    }
  });

  try {
    transaction(population);
  } catch (err) {
    console.error('[SQLite] Failed to save trainer generation:', err);
  }
}

/**
 * Reads the current active trainer genomes for a specific run_id from SQLite, sorted by fitness descending.
 */
export function getTrainerPopulation(runId: string, limit = 25): { id: number; generation: number; name: string; genome: string; fitness: number }[] {
  try {
    const rows = db.prepare('SELECT id, generation, name, genome, fitness FROM trainer_genomes WHERE run_id = ? AND active = 1 ORDER BY fitness DESC LIMIT ?').all(runId, limit);
    return rows as any[];
  } catch (err) {
    console.error('[SQLite] Error reading trainer population:', err);
    return [];
  }
}

/**
 * Reads the all-time historically best trainer genomes (Hall of Fame) for a specific run_id from SQLite.
 */
export function getTrainerHallOfFame(runId: string, limit = 10): { id: number; generation: number; name: string; genome: string; fitness: number }[] {
  try {
    const rows = db.prepare('SELECT id, generation, name, genome, fitness FROM trainer_genomes WHERE run_id = ? ORDER BY fitness DESC LIMIT ?').all(runId, limit);
    return rows as any[];
  } catch (err) {
    console.error('[SQLite] Error reading trainer Hall of Fame:', err);
    return [];
  }
}

/**
 * Reads the top-1 all-time champion genomes across all training runs in SQLite.
 */
export function getAllTrainingsChampions(): { genome: string; generation: number }[] {
  try {
    const rows = db.prepare(`
      SELECT t1.genome, t1.generation
      FROM trainer_genomes t1
      INNER JOIN (
        SELECT run_id, MAX(fitness) as max_fit
        FROM trainer_genomes
        GROUP BY run_id
      ) t2 ON t1.run_id = t2.run_id AND t1.fitness = t2.max_fit
    `).all();
    return rows as any[];
  } catch (err) {
    console.error('[SQLite] Error reading all training champions:', err);
    return [];
  }
}

/**
 * Lists all unique training session run_ids, along with their maximum generation and maximum fitness achieved.
 */
export function getTrainerRuns(): { run_id: string; max_gen: number; max_fit: number; last_updated: number }[] {
  try {
    const rows = db.prepare(`
      SELECT run_id, MAX(generation) as max_gen, MAX(fitness) as max_fit, MAX(created_at) as last_updated
      FROM trainer_genomes
      GROUP BY run_id
      ORDER BY last_updated DESC
    `).all();
    return rows as any[];
  } catch (err) {
    console.error('[SQLite] Error reading unique trainer runs:', err);
    return [];
  }
}

/**
 * Deletes all records of a specific run_id from the trainer_genomes table.
 */
export function clearTrainerHistory(runId: string) {
  try {
    db.prepare('DELETE FROM trainer_genomes WHERE run_id = ?').run(runId);
  } catch (err) {
    console.error('[SQLite] Error clearing trainer history:', err);
  }
}
