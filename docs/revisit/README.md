# Revisit Library — Pixel DNA Life

Stand: 2026-10-19 · Basis-Commit: `998244e` (v1.12.2) · Erstellt durch statische Tiefenanalyse + Frontend-Build-Verifikation.

Zweck: Sammeln, **was vor der Weiterentwicklung repariert/umgebaut werden muss** und **was danach weiterentwickelt wird**. Kein Overhaul um seiner selbst willen — jeder Punkt muss einen konkreten Defekt, ein Risiko oder einen messbaren Blocker benennen.

## Index

| Datei | Inhalt |
| :--- | :--- |
| [00-executive-summary.md](00-executive-summary.md) | TL;DR, Health-Scorecard, Verifikationsstatus |
| [01-critical-bugs.md](01-critical-bugs.md) | P0/P1-Korrektheitsfehler mit Beleg und Fix-Skizze |
| [02-architecture.md](02-architecture.md) | Engine-Monolith, IPC-Protokoll, Typ-Vertrag, Zielbild |
| [03-performance.md](03-performance.md) | Hot-Loop-Kosten, DB im Tick, IPC-Volumen |
| [04-simulation-model.md](04-simulation-model.md) | Biologie, Brain, Physik, Trainer-Fitness, tote Config |
| [05-persistence.md](05-persistence.md) | DB-Pfad, Schema, Wachstum, Zeitstempel |
| [06-frontend.md](06-frontend.md) | Typ-Duplikate, tote Actions, XSS/CSP, Legacy-Logik |
| [07-testing-ci-tooling.md](07-testing-ci-tooling.md) | Testabdeckung, CI, Toolchain |
| [08-docs-and-agent-context.md](08-docs-and-agent-context.md) | Doku-Drift, Skills, Tickets, Changelog |
| [roadmap.md](roadmap.md) | Phase 0 → 1 → 2, Reihenfolge, Definition of Done |

## Legende

**Priorität**
- `P0` — Bricht Kernfunktion oder Release. Vor jeder Feature-Arbeit beheben.
- `P1` — Verfälscht Simulationsergebnisse oder blockiert geplante Features.
- `P2` — Wartbarkeit/Performance-Schuld mit absehbaren Folgekosten.
- `P3` — Hygiene. Nur mitnehmen, wenn die Datei ohnehin angefasst wird.

**Beleg**
- `[verifiziert]` — durch Ausführung bestätigt (Build, Test, Grep-Zählung).
- `[statisch]` — durch Code-Lesen eindeutig belegt, nicht ausgeführt.
- `[Hypothese]` — plausibel, braucht Messung/Repro vor dem Fix.

**ID-Schema:** `RV-<Bereich><Nr>` — `B` Bugs, `A` Architektur, `P` Performance, `S` Simulation, `D` Datenbank, `F` Frontend, `T` Test/CI, `X` Doku/Kontext.

## Arbeitsweise

1. Phase 0 aus [roadmap.md](roadmap.md) abarbeiten; jedes `RV-*` wird beim Start ein Ticket in `.tickets/open/`.
2. Erledigte Punkte hier nicht löschen, sondern mit `✔ <commit>` markieren (Audit-Trail).
3. Neue Befunde während der Arbeit hier ergänzen, nicht nur im Chat.
