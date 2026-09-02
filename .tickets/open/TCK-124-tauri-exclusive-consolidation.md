---
id: TCK-124
title: Radical Workspace Convergence & Legacy TS Backend Eradication
status: open
assigned: Gemini-CLI
created_at: 2026-08-31
---

# TCK-124: Radical Workspace Convergence & Legacy TS Backend Eradication

## Background & Motivation
The user wants to operate exclusively from the repository root without ever having to `cd` into subfolders like `src-tauri`. We achieved a complete convergence: both the Rust backend (`Cargo.toml`) and the TypeScript frontend (`package.json`) now live side-by-side at the exact top-level root of the project. Furthermore, all obsolete TS backend simulation code has been ruthlessly deleted.

## Scope & Impact
- **Root Convergence:** `Cargo.toml`, `tauri.conf.json`, and `package.json` now sit at the repository root.
- **`src/` became Rust:** The Rust backend takes ownership of the top-level `src/` folder.
- **`frontend/` for UI:** The TypeScript frontend is moved to a top-level `frontend/` folder.
- **Eradication:** Complete deletion of obsolete TypeScript biology/physics/simulation logic.
- **Command Simplicity:** All commands (`cargo build`, `npm run test`, `tauri dev`) can be run directly from `/home/torsten/projects/digi-life/`.

## Solution Implemented

### 1. Rename & Flatten the Frontend
- Renamed the existing `src/` folder to `frontend/`.
- Inside `frontend/`, removed the `tauri/` sub-directory by moving its contents (`tauri_ocean.ts`, `tauri_trainer.ts`, `api.ts`, `signals.ts`, `beta.css`) up to the root of `frontend/`.
- Updated `index.html`, `vite.config.ts`, and `tsconfig.json` to reference the new `frontend/` paths instead of `src/`.

### 2. Eradication of Legacy TypeScript Backend Logic
- Deleted all TS logic inside `frontend/` that was replaced by Rust:
  - Deleted `frontend/biology/`.
  - Deleted obsolete TS files and dependencies.

### 3. Elevated Rust Backend to Root
- Moved all contents of `src-tauri/` directly to the repository root:
  - `src-tauri/Cargo.toml` -> `/Cargo.toml`
  - `src-tauri/Cargo.lock` -> `/Cargo.lock`
  - `src-tauri/tauri.conf.json` -> `/tauri.conf.json`
  - `src-tauri/build.rs` -> `/build.rs`
  - `src-tauri/icons/` -> `/icons/`
  - `src-tauri/gen/` -> `/gen/`
  - `src-tauri/capabilities/` -> `/capabilities/`
  - `src-tauri/src/` -> `/src/`
- Deleted the now-empty `src-tauri/` directory.

### 4. Configuration & Path Alignment
- Updated `tauri.conf.json` build paths to `./dist` instead of `../dist`.
- Updated `package.json` scripts.
- Uninstalled `ws` and `@types/ws` dependencies.
- Deleted legacy documentation like `OLD_SIMS_ARCHIVE.md`.
