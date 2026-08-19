# CONTEXT SCAN (DUMP)

## Process

1. **Delta Discovery:** Scan `packages/` & `platforms/` for structural changes.
2. **Repo Map (Skeleton):**
   - Extract signatures (classes, methods, interfaces) into `.agents/skills/architecture-overview/REPO_MAP.md`.
   - No implementation logic. Only structure.
3. **Skill Sync:** Map findings to `.agents/skills/*/SKILL.md`.
4. **Legacy Sync:** Mirror updates from root `AGENTS.md` into skills.
5. **Persistence:** Write ONLY if delta detected. Include ISO 8601 timestamp.

## Format

- English only.
- High-density Markdown.
- Universal AI Agent Directive included in new skills.
