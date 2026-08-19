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
