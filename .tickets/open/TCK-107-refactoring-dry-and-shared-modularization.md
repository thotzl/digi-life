---
id: TCK-107
title: Refactoring, DRY, und modulare Komponenten-Sharing zwischen Sandbox, Trainer und Simulation
status: open
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
- **Problem:** Die Flexions-Kurvenbewegung (body bending), Schubkrafterzeugung, Reibung, Wandreflexion und Felskollisions-Schnittpunkte sind in `src/server/index.ts`, `src/server/balanceSim.ts`, `src/trainer.ts` und `src/preview.ts` vierfach dupliziert.
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
