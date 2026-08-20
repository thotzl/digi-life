---
id: TCK-107
title: Refactoring, DRY, und modulare Komponenten-Sharing zwischen Sandbox, Trainer und Simulation
status: closed
assigned: Gemini-CLI
created_at: 2026-08-19
---

# TCK-107: Refactoring, DRY, und modulare Komponenten-Sharing zwischen Sandbox, Trainer und Simulation

## Beschreibung
Beseitigung von Redundanzen (DRY) und Etablierung einer Single Source of Truth (SSOT) durch Auslagerung identischer physikalischer, sensorischer und neuronaler Rechenkerne in isomorphe, geteilte TypeScript-Module. Diese Module sollen nahtlos sowohl vom Node.js-Backend als auch vom Vite-Frontend importiert werden können.

## Ziele
- **DRY (Don't Repeat Yourself):** Beseitigung aller kinetischen und sensorischen Code-Duplikate.
- **SSOT (Single Source Of Truth):** Jede physikalische Formel (z. B. Körperbiegungs-Steuerung, Impulskollision) existiert genau einmal im gesamten Projekt.
- **Isomorphes Sharing:** Maximale Wiederverwendung von logischen Hilfsfunktionen und mathematischen Einheiten zwischen Frontend (Browser-Visualizer) und Backend (Simulation-Server).

## Beobachtete Duplikate & Refactoring-Szenarien

### 1. Physik & Kinetik (Physics Unit)
- **Problem:** Die Flexions-Kurvenbewegung (body bending), Schubkrafterzeugung, Reibung, Wandreflexion und Felskollisions-Schnittpunkte sowie die **Kreatur-zu-Futter-Kollisionsauflösung (Wegschubsen und Driften von Sporen/Pellets)** sind in `src/server/index.ts`, `src/server/balanceSim.ts`, `src/trainer.ts` und `src/preview.ts` vierfach dupliziert.
- **Lösung:** Auslagerung in eine zustandslose **`src/shared/physics.ts`** Einheit.

### 2. Sensorik-Abtastung (Sensory Unit)
- **Problem:** Die räumliche Reizberechnung über Augen, Geruchs-Rezeptoren, Vibrations- und Thermal-Sensoren ist in der Ozean-Sim, dem Headless-Simulator und den Sandboxen dupliziert.
- **Lösung:** Auslagerung in ein geteiltes **`src/shared/sensory.ts`** Modul.

### 3. Gehirn-Integrator (Brain Unit)
- **Problem:** `executeBrain` (CTRNN Euler-Integration und adaptive Normalisierungen) liegt in `dna.ts` und ist mit physischen DNA-Parsern vermischt.
- **Lösung:** Isolation in eine eigenständige **`src/shared/brain.ts`** Rechen-Einheit.

### 4. Regel- und Parameter-Kapselung (Rules Config)
- **Problem:** Parameter-Bereiche und standardmäßige Diet-Kompensationen sind teilweise verstreut.
- **Lösung:** Erstellung eines statischen Config-Resolvers in `src/shared/configHelper.ts`.

## Derzeit aktive Divergenzen im Code (Dringlichkeits-Belege)

### 1. Sichtfeld-Divergenz (`halfCone`)
- **Im Ozean-Server (`index.ts` L1089) & Trainer (`trainer.ts` L408):** `halfCone` Multiplikator ist `1.5`.
- **Im balanceSim-Simulator (`balanceSim.ts` L209):** `halfCone` Multiplikator ist `2.5`.
- **Divergenz:** Kreaturen sehen im CLI-Simulator deutlich breiter als in der echten Ozean-Welt oder im Trainer.

### 2. Elastizitäts-Divergenz (Fels-Bounces)
- **Im Ozean-Server (`index.ts` L732) & Trainer (`trainer.ts` L534):** Kollisions-Dämpfung ist `0.45` (45% Kinetik-Verlust).
- **Im balanceSim-Simulator (`balanceSim.ts` L234):** Kollisions-Dämpfung ist `0.50` (50% Kinetik-Verlust).
- **Divergenz:** Abpraller an Felsen behalten im Simulator mehr Schwung als im echten Ozean.

### 3. Sensor-Empfindlichkeits-Divergenz (`match`)
- **Im Ozean-Server & Trainer:** Bandbreiten-Teiler ist `(bandwidth * 1.8 + 0.12)`.
- **Im balanceSim-Simulator:** Bandbreiten-Teiler ist `(bandwidth * 2.0 + 0.1)`.
- **Divergenz:** Augen-Sensoren fokussieren Farb- und Wärmequellen im Simulator unpräziser als im Ozean oder Trainer.

