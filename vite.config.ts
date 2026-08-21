import { defineConfig } from 'vite';
import fs from 'fs';
import path from 'path';
import Database from 'better-sqlite3';

// Define the path to our SQLite database
const DB_PATH = path.resolve(__dirname, 'digilife.db');
const db = new Database(DB_PATH);
db.pragma('journal_mode = WAL');

/**
 * Reads all species records from the physical SQLite database.
 */
function readDb(): any[] {
  try {
    return db.prepare('SELECT * FROM species').all();
  } catch (err) {
    console.error("[Vite SQLite] Error reading species records:", err);
    return [];
  }
}

/**
 * Writes all species records back to the physical SQLite database inside a transaction.
 */
function writeDb(records: any[]) {
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

  const transaction = db.transaction((list: any[]) => {
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
    console.error('[Vite SQLite] Transaction rollback:', err);
  }
}

/**
 * Simple body parser helper for connect middleware
 */
function parseJsonBody(req: any): Promise<any> {
  return new Promise((resolve) => {
    let body = '';
    req.on('data', (chunk: any) => {
      body += chunk.toString();
    });
    req.on('end', () => {
      try {
        resolve(JSON.parse(body));
      } catch (err) {
        resolve(null);
      }
    });
  });
}

// Vite Connect Middleware Database Plugin
const speciesDatabasePlugin = () => ({
  name: 'species-database-plugin',
  configureServer(server: any) {
    server.middlewares.use(async (req: any, res: any, next: any) => {
      // Intercept api endpoints
      if (!req.url.startsWith('/api/species') && !req.url.startsWith('/api/simulation')) {
        return next();
      }

      res.setHeader('Content-Type', 'application/json');

      const url = new URL(req.url, 'http://localhost');
      const method = req.method;

      // ======================================================================
      // SIMULATION STATE RESUMPTION ENDPOINTS
      // ======================================================================
      if (url.pathname === '/api/simulation/state') {
        if (method === 'GET') {
          try {
            const row = db.prepare('SELECT state_json FROM simulation_state WHERE key = ?').get('current_state') as any;
            if (row && row.state_json) {
              res.end(row.state_json);
            } else {
              res.end(JSON.stringify({ empty: true }));
            }
          } catch (err) {
            res.end(JSON.stringify({ empty: true }));
          }
          return;
        }
      }

      if (url.pathname === '/api/simulation/save') {
        if (method === 'POST') {
          const body = await parseJsonBody(req);
          if (body) {
            try {
              db.prepare('INSERT OR REPLACE INTO simulation_state (key, state_json, updated_at) VALUES (?, ?, ?)')
                .run('current_state', JSON.stringify(body), Date.now());
              res.end(JSON.stringify({ success: true }));
            } catch (err) {
              res.statusCode = 500;
              res.end(JSON.stringify({ error: "Failed to write simulation state" }));
            }
          } else {
            res.statusCode = 400;
            res.end(JSON.stringify({ error: "Invalid simulation state payload" }));
          }
          return;
        }
      }

      if (url.pathname === '/api/simulation/clear') {
        if (method === 'POST') {
          try {
            db.prepare('DELETE FROM simulation_state WHERE key = ?').run('current_state');
            res.end(JSON.stringify({ success: true }));
          } catch (err) {
            res.statusCode = 500;
            res.end(JSON.stringify({ error: "Failed to clear state" }));
          }
          return;
        }
      }

      // GET /api/species/alive -> Returns currently alive species
      if (method === 'GET' && url.pathname === '/api/species/alive') {
        const db = readDb();
        const alive = db.filter(rec => rec.status === 'alive');
        res.end(JSON.stringify(alive));
        return;
      }

      // POST /api/species/extinct?id=... -> Marks a species as extinct
      if (method === 'POST' && url.pathname === '/api/species/extinct') {
        const id = url.searchParams.get('id');
        if (!id) {
          res.statusCode = 400;
          res.end(JSON.stringify({ error: "Missing id parameter" }));
          return;
        }

        const db = readDb();
        const record = db.find(rec => rec.id === id);
        if (record) {
          record.status = 'extinct';
          record.extinctionTime = Date.now();
          writeDb(db);
        }

        res.end(JSON.stringify({ success: true }));
        return;
      }

      // GET /api/species -> Fetch single species by ID, or ALL species if ID is omitted!
      if (method === 'GET' && url.pathname === '/api/species') {
        const id = url.searchParams.get('id');
        const db = readDb();
        if (!id) {
          res.end(JSON.stringify(db)); // return ALL species recorded on disk!
          return;
        }

        const record = db.find(rec => rec.id === id);
        res.end(JSON.stringify(record || null));
        return;
      }

      // POST /api/species -> Saves or updates a species record
      if (method === 'POST' && url.pathname === '/api/species') {
        const body = await parseJsonBody(req);
        if (!body || !body.id) {
          res.statusCode = 400;
          res.end(JSON.stringify({ error: "Invalid or empty species body" }));
          return;
        }

        const db = readDb();
        const idx = db.findIndex(rec => rec.id === body.id);
        if (idx !== -1) {
          db[idx] = body; // update
        } else {
          db.push(body); // insert
        }

        writeDb(db);
        res.end(JSON.stringify({ success: true }));
        return;
      }

      // POST /api/species/clear -> Resets evolution history
      if (method === 'POST' && url.pathname === '/api/species/clear') {
        writeDb([]);
        res.end(JSON.stringify({ success: true }));
        return;
      }

      // Fallback 404 for unknown API routes
      res.statusCode = 404;
      res.end(JSON.stringify({ error: "API Route not found" }));
    });
  }
});

export default defineConfig({
  server: {
    port: 3000,
    open: process.env.TAURI_ENV_PLATFORM === undefined
  },
  plugins: [
    speciesDatabasePlugin()
  ],
  build: {
    rollupOptions: {
      input: {
        tauri_ocean: path.resolve(__dirname, 'tauri_ocean.html'),
        tauri_trainer: path.resolve(__dirname, 'tauri_trainer.html')
      }
    }
  }
});
