# 00 — Executive Summary

## TL;DR

Die Codebasis ist ein funktionierender Prototyp mit ambitioniertem Bio-Modell, aber **die Ocean-Simulation evolviert aktuell nicht**: Mitose-Nachkommen werden nie in die Simulation übernommen (RV-B01), die Population wird nur durch Restocking aufgefüllt. Dazu kommen ein Release-Blocker (DB- und Config-Pfad relativ zum Arbeitsverzeichnis, RV-B03/B04), ein 2000-Zeilen-Thread-Closure als Engine (RV-A01), DB-Abfragen im 60-Hz-Hot-Loop (RV-P01) und großflächige Doku-Drift (RV-X01). Ein gezielter Umbau ist gerechtfertigt; ein kompletter Rewrite nicht.

## Codebasis in Zahlen

| Bereich | Umfang |
| :--- | :--- |
| Rust (`src/`) | ~6.770 LOC, davon `engine.rs` 1.989, `dna.rs` 1.894 |
| TypeScript (`frontend/`) | ~5.300 LOC + 816 LOC CSS |
| Rust-Tests | ~20 Unit-Tests (dna, brain, database) |
| Frontend-Tests | **0** (Vitest/Stryker konfiguriert, keine Testdateien) |
| Commits | 181 in ~3 Wochen (Aug/Sep 2026), 12 Minor-Releases |
| Config-Regeln | 44 in `SimulationRules`, davon **16 nie gelesen** `[verifiziert]` |

## Health-Scorecard

| Dimension | Bewertung | Kernbefund |
| :--- | :--- | :--- |
| Korrektheit Ocean | kritisch | Keine Vererbung im Ocean (RV-B01), Wand-Bug (RV-B02) |
| Korrektheit Trainer | mittel | Speed-Bonus tot (RV-B05), inkonsistente Spawn-Distanzen (RV-S08) |
| Release-Fähigkeit | kritisch | DB unter `target/`, `config.json` nicht gebündelt (RV-B03/B04) |
| Architektur | schwach | God-Function + stringly-typed IPC (RV-A01/A02) |
| Performance | schwach, skaliert nicht | DB-Scan pro Physik-Step, Klon-Orgien (RV-P01–P04) |
| Testbarkeit | schwach | Engine nicht testbar (Closure + Tauri-Window), 0 FE-Tests |
| Doku/Agent-Kontext | irreführend | Skills beschreiben WebSocket/JSON-Architektur, die nicht mehr existiert |
| Biologie-Modell | solide Basis | Kreativ und deterministisch; Balancing-Probleme (RV-S01–S07) |

## Was gut ist (behalten)

- **Genom-Compiler (`dna.rs`)**: deterministisch, promoter-basiert, gut isoliert als pure function `parse_genome`. Tragfähige Kernidee.
- **CTRNN + Thalamus-Bottleneck**: sauber sortierte Synapsen für O(S)-Pass, klare Schichtsemantik.
- **Scenario-Plugin-Trait** (`scenarios.rs`): richtige Abstraktion für TCK-123 (Curriculum).
- **ts-rs-Annotationen**: Typ-Vertrag ist vorbereitet, wird nur nicht genutzt (RV-A03).
- **Ticket-/Changelog-Disziplin**: Struktur vorhanden, nur inhaltlich gedriftet.

## Verifikationsstatus dieser Analyse

| Check | Ergebnis |
| :--- | :--- |
| `npm ci` | OK (295 Pakete, Deprecation-Warnings) `[verifiziert]` |
| `tsc --noEmit` | 0 Fehler `[verifiziert]` |
| `vite build` | OK, 88 kB Haupt-Bundle `[verifiziert]` |
| `vitest run` | nicht ausführbar sinnvoll — keine Testdateien `[verifiziert]` |
| `cargo test` / `clippy` | **nicht ausgeführt** — auf diesem Windows-Rechner ist kein Rust-Toolchain installiert. Alle Rust-Befunde sind `[statisch]`. |
| App-Laufzeit | nicht gestartet — Laufzeit-Hypothesen sind als solche markiert |

**Erste Aktion vor jeder Code-Änderung:** Rust-Toolchain (rustup, MSVC) installieren, `cargo test` + `cargo clippy` als Baseline festhalten (siehe [roadmap.md](roadmap.md), Schritt 0.1).
