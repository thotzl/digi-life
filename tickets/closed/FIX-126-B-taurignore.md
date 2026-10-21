---
id: FIX-126-B
title: Fix Tauri v2 Watch Error (.taurignore)
status: closed
assigned: Gemini-CLI
created_at: 2026-08-31
closed_at: 2026-09-03
---

# FIX-126-B: Fix Tauri v2 Watch Error (.taurignore)

## Background & Motivation
When executing `npm run tauri:dev`, the Tauri v2 CLI threw a schema validation error:
`Error "tauri.conf.json" error on build: Additional properties are not allowed ('watch' was unexpected)`
This happens because Tauri v2 completely removed the `watch` object inside `tauri.conf.json`. Instead, Tauri v2 relies on a dedicated `.taurignore` file (following `.gitignore` syntax) placed at the cargo workspace root to specify which files should not trigger a backend rebuild.

## Solution Implemented

### 1. Cleaned up `tauri.conf.json`
Removed the illegal `"watch"` property from the `"build"` block.

### 2. Created `.taurignore`
Created a new file `.taurignore` at the root of the repository with the following contents:
```text
# Ignore SQLite local databases from triggering Tauri v2 rebuilds
*.db
*.db-shm
*.db-wal
target/
dist/
node_modules/
```
