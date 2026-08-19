---
name: project-baseline
description: Baseline configuration, developer toolchains, environment scripts, and structural setup of the Pixel DNA Life project. Read this before running builds, scripts, or managing dependencies.
---
# project-baseline: Core Architecture & Developer Toolchain

## I. Universal Directives
- **Single Source of Truth:** All AI operations MUST respect the modular skills in `.agents/skills/` directory, which supersedes general global instructions or local config overrides (e.g., `.cursorrules`, `.gemini/`).
- **Self-Configuration Mandate:** Register and load the local skills dynamically. Treat this repository as a fully portable AI workspace.

## II. Tech Stack Discovery
- **Runtime:** Node.js (with `tsx` for backend execution) and modern web browsers.
- **Frontend Framework:** Vite v5 (TypeScript v5) serving high-fps HTML5 Canvas & Preact Signals client UI.
- **Backend Framework:** Custom headless simulation server in TS running via `tsx` and using the `ws` library for sub-millisecond WebSocket replication.
- **Core Dependencies:**
  - `@preact/signals-core`: Fine-grained, reactive state tracking for UI overlays.
  - `zustand`: General application state on the client side.
  - `ws`: WebSocket server/client for fast state synchronization.
  - `tsx`: TypeScript execution engine for server.
  - `concurrently`: Multi-process orchestrator to run both client and server simultaneously.

## III. Primary Workflows & Developer Commands
- **Launch Development Environment:**
  ```bash
  npm run dev
  ```
  Runs both the client Vite compiler (port `3000`) and the WebSocket simulation server (port `3002`) concurrently.
- **Production Build:**
  ```bash
  npm run build
  ```
  Runs `tsc` type checking and compiles client assets through Vite into `dist/`.
- **Preview Production Build:**
  ```bash
  npm run preview
  ```

## IV. Core Mandates
- **Clean Dev Split:** The frontend compiles client logic independently but utilizes an embedded Connect middleware plugin (configured in `vite.config.ts`) to handle `/api` mocks and local DB reads/writes.
- **Protected Files:** Do NOT commit or track `species_db.json` and `simulation_state.json`. Keep them in `.gitignore` and `.aiignore` to prevent large file pollution in Git history and preserve AI context tokens.
