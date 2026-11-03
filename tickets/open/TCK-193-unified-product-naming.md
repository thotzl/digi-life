---
id: TCK-193
title: Unified Product Naming (and Optional Rename)
status: open
priority: low
decision: partial (product name BroodBattle decided 2026-11-02; identifier alpha-only)
assigned: unassigned
created_at: 2026-10-26
---

# TCK-193: Unified Product Naming (and Optional Rename)

## Update (2026-11-03, after TCK-196)
- Done so far: product name **BroodBattle** in `productName`, `mainBinaryName`, window/page titles, main menu (TCK-195); release name in `.github/workflows/release.yml` (TCK-161); `README.md`, `AGENTS.md`, `wiki/_master-index.md`, skill descriptions (TCK-203). The default session name "Tauri Desktop Laboratory Studio" is gone with the `sessions` table (TCK-159).
- Still open: crate `src-tauri` (lib `src_tauri`), `package.json` name `digi-life`, DB file `pixel_life_local.db` (`DB_FILE_NAME` in `src/persistence/mod.rs`; a rename needs a one-time copy), `sync-tickets.yml` / User-Agent, test temp folders `digi-life-test-*`, `wiki/overview/whitepaper.md` (user's vision document, still "Pixel DNA Life"), repo name. Identifier is alpha-only (`de.pixlich.broodbattle`, TCK-195).

Low priority, "when there is time". The project uses four name families plus technical leftovers; the app shows four different titles. Decide one name, then align every place below. The repo may be renamed but does not have to be.

## Partial decision (2026-11-02, TCK-195)
- Product name **BroodBattle** (quick web check only, no trademark search). Set in TCK-195: `productName`, `mainBinaryName` (`BroodBattle.exe`), window and page titles, main-menu heading.
- Identifier `de.pixlich.broodbattle` (dev `de.pixlich.broodbattle.dev`) is **alpha only**: before the beta a final domain is chosen and the identifier changed once, with a one-time copy of the alpha save folder (details in TCK-195 section Identifier).
- Still open here: crate `src-tauri`, `package.json` name, DB file name, workflows, User-Agent, test folders, default session name, docs/skills/wiki, repo.

## Current names (inventory 2026-10-26, `git grep`)
1. **"Pixel DNA Life"** (product name, most widespread): `tauri.conf.json` (`productName` "Pixel DNA Life", window title "Pixel DNA Life - Evolution Laboratory"), `README.md`, `AGENTS.md` ("🧬 Pixel DNA Life"), `capabilities/default.json`, `gen/schemas/capabilities.json`, `.github/workflows/release.yml` (release name), `frontend/components/Icons.ts` (comment), wiki (`overview/whitepaper.md` 7×, `_master-index.md`, `architecture/system-overview.md`, `research/alife-reference-simulators.md`), `ideas.md` (4×), descriptions of all six project skills in `.agents/skills/*`.
2. **"Pixel DNA"**: `index.html` `<title>` "Pixel DNA - Desktop Laboratory (Tauri SPA Edition)", `frontend/main.ts` `document.title` "Pixel DNA - Laboratory Main Hub", `frontend/views/MainMenu.ts` heading "Pixel DNA"; identifier `com.pixeldna.life` (`tauri.conf.json`; decides the AppData folder and thus the DB location; wiki `persistence.md`, `quality/runtime-verification.md`).
3. **"pixel_life"**: DB file `pixel_life_local.db` (`src/database.rs` `DB_FILE_NAME`, `LEGACY_DB_PATH` `target/pixel_life_local.db`), `.gitignore`, wiki (`persistence.md`, `simulation/analytics-and-balancing.md`), closed tickets TCK-137, FIX-126.
4. **"digi-life" / "Digi-Life"**: repo `thotzl/digi-life` and folder name, `package.json` `"name": "digi-life"`, `.github/workflows/sync-tickets.yml` ("Digi-Life Ticket Synchronization"), `.github/scripts/sync_github.cjs` (User-Agent "Digi-Life-Sync-Script"), test temp folders `digi-life-test-*` (`src/database.rs`), TCK-189 (`gh api` paths), closed tickets TCK-115, TCK-124.
5. **Technical leftovers**: crate `src-tauri` (`Cargo.toml`, lib `src_tauri`, imported in `src/main.rs` and `src/bin/trainer_runner.rs`), default session name "Tauri Desktop Laboratory Studio" (`src/server/engine.rs`).

## Rename impact
- Identifier: moves the AppData folder; existing DBs are not found without a one-time copy (pattern: `migrate_legacy_db`).
- DB file name: one-time copy as well.
- Crate name: `use src_tauri::…` imports in 2 files; no runtime effect.
- Repo: GitHub redirects the old URL; update the local remote, `gh api` paths, links/badges.
- Titles, `package.json`, workflow names, docs, skills: plain text, no runtime risk.

## Name ideas
The whitepaper describes a game (train creatures, then send them into arenas), so the name may express that. No trademark/domain/store check done yet.
- **Keep / small step:** Pixel DNA Life (keep, lowest effort), Pixel DNA (shorter, already the main-menu heading), Digi Life (matches repo, sounds generic).
- **Breeding + fighting:** BroodBattle (user favourite), Brood & Battle, Spawn & Strife, Hatch & Clash, Breed Arena, Brawlbreed.
- **DNA / genome as the weapon:** Codon Clash (codons = the three-letter words of DNA, fits the letter genomes), Helix Arena, GeneForge (fits the Creator), Splice Wars, Allele Arena, Genome Arena, Pixel DNA Arena.
- **Pixel + biology:** Pixel Brood, Bitbreed, Pixel Primordia, Protopix.
- **Creature / microbe feel:** Cilia Clash, Microbe Arena, Plasmids, Petri Wars (lab dish → arena).
- **Brain and learning:** Neuroids, Synapse Arena, Wired to Hunt.
- Agent shortlist: BroodBattle, Codon Clash, Petri Wars, Pixel Brood.

## Scope (once a name is chosen)
- One product name in one place (`tauri.conf.json`), all titles derived from it (window title = product name, views as subtitle, e.g. "<Name> – Training Room").
- Technical names in the same family: `package.json`, crate, DB file (with copy), workflows, User-Agent, test temp folders, default session name.
- Identifier: keep unless the new name makes it misleading (then move with a DB copy).
- Docs, README, AGENTS.md, skills, wiki; optional repo rename.

## Acceptance
- `git grep` finds no old name outside history (CHANGELOG, closed tickets); app titles consistent; existing user DB still found after the change (UI smoke); CI green.
