---
id: FIX-126
title: Fix Infinite Rebuild Loop & Port Mismatch
status: closed
assigned: Gemini-CLI
created_at: 2026-08-31
closed_at: 2026-09-03
---

# FIX-126: Fix Infinite Rebuild Loop & Port Mismatch

## Background & Motivation
The user experienced two critical issues when running `tauri dev`:
1.  **Port Mismatch (404 Error):** The frontend was returning a JSON 404 error (`{"message":"Cannot GET /","error":"Not Found","statusCode":404}`). This happened because port `3000` was likely occupied, causing Vite to silently fallback to port `3001` while Tauri was still hardcoded to look for `3000`.
2.  **Infinite Rebuild Loop:** The console output showed an infinite loop where the Rust backend was constantly rebuilding because `pixel_life_local.db` was being modified by the running simulation. Tauri's watcher saw the database file change and triggered a hot-reload, which restarted the app, which wrote to the DB, causing another restart.

## Solution Implemented

### 1. Unique Port & Strict Binding
- Edited `vite.config.ts`:
  - Set the development port to a highly unique and exotic port (**`8765`**).
  - Enforced `strictPort: true` to ensure Vite crashes loudly if the port is occupied instead of secretly shifting to another port.
- Edited `tauri.conf.json`:
  - Mapped `"devUrl": "http://localhost:8765/"`.

### 2. Watch Ignore for Database File
- Added `watch.ignore` configurations to `tauri.conf.json` so that SQLite files do not trigger Rust backend hot-rebuilds:
  ```json
  "watch": {
    "ignore": [
      "**/*.db",
      "**/*.db-shm",
      "**/*.db-wal"
    ]
  }
  ```
