---
id: TCK-103
title: Improve Petri Dish simulation and environmental physics (Bio-Basin)
status: closed
assigned: Gemini-CLI
created_at: 2026-08-19
closed_at: 2026-08-19
---

# TCK-103: Improve Petri Dish simulation and environmental physics (Bio-Basin)

## Beschreibung
Umstellung der Simulationswelt von einer sterilen flachen Platte in ein toroidal oder geschlossen simuliertes, prozedural generiertes "Cybernetic Bio-Basin" mit physikalischen Hindernissen, Thermalströmungs-Vents und biologischen Biomen. Integration von Raumpartitionierung zur performanten Kollisionsprüfung.

## Teilaufgaben
- [x] **Central Configuration (`config.json`):** Einbindung zentraler Parameter zur BMR, Sporen- und Creature-Limits.
- [x] **Procedural World Generation (`mapGenerator.ts`):** Seed-basierte PRNG Mulberry32 Generierung von Hindernissen, Biomen und Landmarken-Vents.
- [x] **Closed-Basin Physik:** Wechsel von toroidalen Loops auf harte, elastisch abprallende Außenwände. Reaktivierung des Wandwarnungs-Sensors.
- [x] **Spatial Grid Partitioning (`spatialGrid.ts`):** Aufteilung des $19200 \times 10800$ großen Beckens in $80 \times 80$ Pixel Zellen zur Reduzierung von Kollision-Suchen von $O(N)$ auf $O(1)$.
- [x] **Physische Multi-Agent-Kollisionen:** Implementierung von biologischen Stoßbouncen (Creature-Creature) und elastischer Sporen-Verdrängung (Creature-Food).
- [x] **Synchronisation & Vorschau:** Vollständige Visualisierung im Browser und auf der Standalone-Vorschauseite (`preview.html`).

## Verification
- **Unit Tests:**
  - `src/server/spatialGrid.test.ts` (2 passing tests): Verifiziert das Einfügen, Abfragen und Begrenzen von Grid-Zellen.
  - `src/shared/mapGenerator.test.ts` (6 passing tests): Prüft PRNG-Determinismus, Biome und Hindernis-Kollisionen.
  - `src/server/simulation.test.ts` (8 passing tests): Prüft elastische Prallphysik, Sporen-Saugwellen und Massenstöße.
- **Simulation Validation:**
  - Synchroner 2.000-Tick Stabilitätslauf liefert einen Score von **100.33 / 100** bei genau 25 stabilen Organismen.
- **Type-Safety:** `npm run build` kompiliert ohne Compiler-Fehler.
