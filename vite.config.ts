import { defineConfig } from 'vite';
import fs from 'fs';
import path from 'path';

// Define the path to our physical JSON Database file inside the workspace
const DB_FILE_PATH = path.resolve(__dirname, 'species_db.json');
const STATE_FILE_PATH = path.resolve(__dirname, 'simulation_state.json');

/**
 * Helper to ensure the physical JSON database file exists on disk with an empty list.
 */
function ensureDbFile() {
  if (!fs.existsSync(DB_FILE_PATH)) {
    fs.writeFileSync(DB_FILE_PATH, JSON.stringify([], null, 2), 'utf-8');
  }
}

/**
 * Reads all species records from the physical file.
 */
function readDb(): any[] {
  ensureDbFile();
  try {
    const data = fs.readFileSync(DB_FILE_PATH, 'utf-8');
    return JSON.parse(data);
  } catch (err) {
    console.error("Error reading species_db.json, resetting database:", err);
    return [];
  }
}

/**
 * Writes all species records back to the physical file.
 */
function writeDb(records: any[]) {
  fs.writeFileSync(DB_FILE_PATH, JSON.stringify(records, null, 2), 'utf-8');
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
          if (fs.existsSync(STATE_FILE_PATH)) {
            const data = fs.readFileSync(STATE_FILE_PATH, 'utf-8');
            res.end(data);
          } else {
            res.end(JSON.stringify({ empty: true }));
          }
          return;
        }
      }

      if (url.pathname === '/api/simulation/save') {
        if (method === 'POST') {
          const body = await parseJsonBody(req);
          if (body) {
            fs.writeFileSync(STATE_FILE_PATH, JSON.stringify(body, null, 2), 'utf-8');
            res.end(JSON.stringify({ success: true }));
          } else {
            res.statusCode = 400;
            res.end(JSON.stringify({ error: "Invalid simulation state payload" }));
          }
          return;
        }
      }

      if (url.pathname === '/api/simulation/clear') {
        if (method === 'POST') {
          if (fs.existsSync(STATE_FILE_PATH)) {
            fs.unlinkSync(STATE_FILE_PATH);
          }
          res.end(JSON.stringify({ success: true }));
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
    open: true
  },
  plugins: [
    speciesDatabasePlugin()
  ]
});
