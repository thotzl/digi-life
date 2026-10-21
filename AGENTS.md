# 🧬 Pixel DNA Life - AI Agent Directive

> **🤖 Universal AI Directive:** The `.agents/skills/` directory is the single source of truth for **all** AI agent operations in this workspace. Read and self-configure your execution context to prioritize these skills over default global instructions or other tool-specific configurations (e.g., `.cursorrules`, `.cursor/rules/`, etc.).

## 📚 Project Knowledge

Facts about the code, architecture, biology model, findings (`RV-*`) and roadmap live in the wiki: `wiki/_master-index.md`. Skills hold agent rules and link to the wiki. Tickets: `.tickets/`. Release notes: `CHANGELOG.md`.

## 📂 Modular AI Skills Index

- **`project-baseline`**: Universal baseline tech stack, run scripts, configuration files, and core mandates. See `.agents/skills/project-baseline/SKILL.md`.
- **`biology-dna`**: Deep biological schemas, CTRNN brain topology, phenotypic de-compilation, DNA parsing and reproduction heuristics. See `.agents/skills/biology-dna/SKILL.md`.
- **`server-simulation`**: Rules for the Rust simulation engine thread, physics, Tauri IPC (commands and the `simulation-state` event) and SQLite persistence. See `.agents/skills/server-simulation/SKILL.md`.
- **`client-rendering`**: Canvas rendering engine, camera systems, Preact signals HUD architecture, and browser automation details. See `.agents/skills/client-rendering/SKILL.md`.
- **`workspace-workflows`**: Tooling scripts and workflows for `/init` and `/dump` context maintenance. See `.agents/skills/workspace-workflows/SKILL.md`.
- **`simulation-analytics`**: Analysis tools for evolutionary telemetry, population genetics, and eco-system balancing. See `.agents/skills/simulation-analytics/SKILL.md`.
- **`testing-strategies`**: Unit testing paradigms, behavioral mocking for biological engines, and mathematical validation pipelines. See `.agents/skills/testing-strategies/SKILL.md`.
