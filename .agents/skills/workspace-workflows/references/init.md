# WORKSPACE INITIALIZATION (INIT)

## Purpose

Initialize a new project context by analyzing the codebase and setting up a standardized `.agents/skills/` architecture. This replaces monolithic instruction files with modular, progressive-disclosure AI skills.

## Process

1. **Tech-Stack Discovery:**

   - Scan root for configuration files: `package.json`, `Cargo.toml`, `requirements.txt`, `go.mod`, etc.
   - Identify runtime (Node, Bun, Python), build tools, and core frameworks (NestJS, React, Drizzle, etc.).

2. **Architecture Mapping:**

   - Detect workspace patterns (Monorepo, Monolith, Microservices).
   - Identify primary entry points (API, Web, CLI).
   - Recognize core design patterns (ECS, MVC, Hexagonal, Plugin-based).

3. **Universal AI Skill Architecture Setup:**

   - Create `.agents/skills/` directory in the project root. This acts as the universal source of truth for ALL AI agents.
   - Create a high-density base skill: `.agents/skills/project-baseline/SKILL.md` containing the fundamental tech stack and absolute core mandates.
   - Ensure the `SKILL.md` has a YAML frontmatter with `name` and `description` (acting as the trigger).
   - **Agent Agnosticism & Self-Configuration:** Ensure the generated files instruct agents that the `.agents/skills/` structure supersedes any tool-specific configurations (like `.cursorrules`, `.gemini/`, etc.). Include a "Self-Configuration Mandate" directing the AI to automatically register the `.agents/skills/` directory in its own persistent configuration. Add a prominent directive to any existing root-level `AGENTS.md` pointing to the `.agents/skills/` directory.

4. **Context Modularity & Legacy Augmentation:**

   - Break down the architecture into specialized skill directories (e.g., `.agents/skills/api-backend/`, `.agents/skills/ecs-engine/`).
   - Create a `SKILL.md` for each with high-level rules.
   - Create a `references/` subdirectory in each skill folder to store detailed documentation.
   - **Legacy Support:** Scan the repository for existing `AGENTS.md` files (e.g., in subdirectories like `platforms/@abs/api/`). Do NOT delete them. Instead, add a directive in the relevant `SKILL.md` (e.g., `api-backend/SKILL.md`) to explicitly read and respect these local files as "Context Augmentations".

5. **Self-Replication (Portable Workflows):**

   - Create a specific skill: `.agents/skills/workspace-workflows/SKILL.md` with description "Standardized workflows for maintaining this project's AI context."
   - Copy the global rules for "init" and "dump" into `.agents/skills/workspace-workflows/references/`.
   - _Why?_ This ensures the project is a "Portable Seed". Other developers or AI agents cloning the repo immediately know how to `/init` a sub-project or `/dump` the context without needing a global config setup.

6. **Verification:**
   - **Legacy Preservation:** Do NOT delete legacy `AGENTS.md` files. They remain as a fallback for the team. Read and augment them into the skills context dynamically.
   - Verify that all `SKILL.md` descriptions clearly state _when_ the AI should read this skill.
