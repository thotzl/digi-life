# CONTEXT SCAN (DUMP)

## Process

1. **Delta Discovery:** Scan the source roots for structural changes. In this repo: `src/` (Rust core), `frontend/` (TypeScript UI), `bindings/` (ts-rs output), root configs (`Cargo.toml`, `package.json`, `tauri.conf.json`, `config.json`).
2. **Repo Map:**
   - Update `wiki/project-map/repository-map.md` when a module is added, removed or changes responsibility.
   - Module level only: no functions, signatures or line numbers.
3. **Wiki & Skill Sync:** Map factual findings to the matching `wiki/` articles and rule changes to `.agents/skills/*/SKILL.md`. Remove statements that no longer match the code.
4. **Root Sync:** Keep the skills index in root `AGENTS.md` consistent with `.agents/skills/`.
5. **Persistence:** Write ONLY if a delta is detected. Include an ISO 8601 timestamp in generated maps.

## Format

- English only.
- High-density Markdown.
- Every path and claim verified against the code.
