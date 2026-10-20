# 08 — Dokumentation, Agent-Kontext, Tickets

Problem: Der Agent-Kontext (`AGENTS.md` → `.agents/skills/*`) ist als "Single Source of Truth" deklariert, beschreibt aber in weiten Teilen eine **frühere Architektur**. Jeder Agent, der den Skills folgt, arbeitet mit falschem Weltbild. Das ist bei KI-gestützter Entwicklung ein P0-Risiko für Folgefehler.

---

## RV-X01 · P0 · Skills beschreiben nicht existierende Architektur — **Status: ✔ TCK-142**

| Skill | Behauptung | Realität |
| :--- | :--- | :--- |
| `server-simulation` | WebSocket-Server Port 3002, Loop in `src/server/index.ts`, State in `simulation_state.json`, Species in `species_db.json` | Tauri-Events, Rust `engine.rs`, SQLite |
| `biology-dna` | Speziation über genetische Drift-Schwelle, `species_db.json`, Sigmoid-CTRNN | Species = Genom-String; SQLite; tanh/relu/sigmoid/sin gemischt |
| `client-rendering` | Renderer in `src/render/creatureRenderer.ts` | `frontend/render/creatureRenderer.ts` |
| `simulation-analytics` | Analyse über `simulation_state.json`, `species_db.json` | Dateien existieren nicht; Daten in SQLite |
| `testing-strategies` | "nicht den WebSocket-Server starten"; τ ∈ [0.5, 5.0] | kein WebSocket; τ ∈ [0.1, 2.5] |
| `project-baseline` | Verweis auf `OLD_SIMS_ARCHIVE.md`; `src-tauri/`-Pfade | Datei existiert nicht; kein `src-tauri/` |
| `workspace-workflows` | Verweis auf `/home/torsten/.gemini/skills/...`; `dump.md` scannt `packages/`, `platforms/` | Fremdpfade aus anderem Projekt |

**Ziel:** Skills neu schreiben, **nachdem** Phase-0-Fixes die Architektur stabilisiert haben (sonst doppelte Arbeit) — aber die falschesten Aussagen (WebSocket, JSON-Dateien) sofort mit einem Warnhinweis versehen.

## RV-X02 · P1 · `.agents/state/active-task.md` enthält veraltete "Safety Rules" — **Status: ✔ TCK-142**

Verweist auf `src/tauri/tauri_trainer.ts`, `src-tauri/src/biology/dna.rs`; Regel 4 nennt eine Thrust-Formel mit Faktor `6.0`, Code nutzt Config (3.4). TCK-116 ist als DONE markiert. Regeln 2 und 3 sind weiterhin gültig und gehören in Tests statt in Prosa (Warp-Cap 35 als Konstante + Test; Reset leert Newborn-Queue → mit RV-B01 obsolet).

## RV-X03 · P1 · README / WHITEPAPER / HANDOFF veraltet oder irreführend — **Status: ✔ TCK-142**

- **README:** `cargo test --manifest-path src-tauri/Cargo.toml`, `src/tauri_ocean.ts`, "re-exportiert 12 Bindings", "Rust parser decodes genomes exactly identical to the TS specifications" (es gibt keinen TS-Parser mehr), `rm -f pixel_life_local.db` (falscher Pfad).
- **WHITEPAPER:** Kapitel 5.2 "Real-time WebSocket Replication"; beschreibt globale Multi-Agent-Welt — Vision, kein Ist-Zustand. Als Vision kennzeichnen.
- **HANDOFF.md:** Offene Diagnose "Motionless Ocean" mit Vermutungen; durch RV-B01/RV-S02 präziser erklärt. Nach Phase 0 ersetzen.
- **CHANGELOG:** Marketing-Sprache ("millimeter-perfect", "100%", "flawless"-Äquivalente) verschleiert, was sich technisch geändert hat; mehrere Releases pro Tag. Künftig: was, warum, Risiko.

## RV-X04 · P2 · Ticket-Status inkonsistent — **Status: ✔ TCK-142**

- `TCK-124` und `TCK-125` liegen in `open/`, enthalten aber "Solution Implemented" → nach `closed/`.
- `TCK-123` in `open/` mit `status: active` → nach `ongoing/` oder Status korrigieren.
- Tickets nennen `assigned: Gemini-CLI` → aktualisieren oder Feld entfernen.
- `TCK-111` fordert IndexedDB-Speicherung für den Creator — widerspricht der SQLite-Catalogue-Architektur → Scope überarbeiten.
- `TCK-113` (Mobile) referenziert `beta.css` (existiert nicht mehr).

## RV-X05 · P2 · Doppelte Wahrheiten im Repo-Root — **Status: ✔ TCK-142**

- `.gitignore` listet `src-tauri/...`- und JSON-DB-Pfade, die es nicht mehr gibt; `.aiignore` enthält `!/target` (macht `target/` für KI sichtbar — inkl. Build-Artefakte).
- `gen/schemas/*.json` (Tauri-generiert) ist eingecheckt — prüfen, ob gewollt.
- `ideas.md` ist ein Recherche-Dossier, kein Backlog → in `docs/` verschieben und aus Roadmap verlinken.

## RV-X06 · P3 · Inline-Kommentare in Mischsprache und mit Historien-Narrativ — **Status: offen, nur beim Anfassen der Datei**

Kommentare wie "Phase 5 of TCK-133", "Point 1 Fallback", "matches TS index.ts!", "Brutal 95% penalty" beschreiben Entstehungsgeschichte statt Absicht. Deutsch/Englisch gemischt. Beim Anfassen der jeweiligen Datei bereinigen, kein eigener Pass.
