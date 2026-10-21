---
id: TCK-106
title: Advanced Carnivore Training and Adaptive CTRNN Physiology
status: closed
assigned: Gemini-CLI
created_at: 2026-08-19
closed_at: 2026-08-19
---

# TCK-106: Advanced Carnivore Training and Adaptive CTRNN Physiology

## Beschreibung
Erweiterung der biologischen und physischen Realität im evolutionären Trainer. Integration von adaptivem Preymocking (Meatballs) für Fleischfresser-Sensoren (Wärme, Geruch, Vibration, Augen), evolutionär mutierenden Aktivierungsfunktionen (tanh, ReLU, Sigmoid, Sinus) für Hidden-Neuronen, sowie einem 100% spalten-stabilen 3-Säulen-IDE-Dashboard mit schwebendem Hover-Tooltip.

## Teilaufgaben
- [x] **Prey-Peer Mocking (Meatballs):** Simulation einer roten Fleischspore als lebende Beute mit metabolischen, thermischen und vibrierenden Reizen zur Jägerzucht.
- [x] **Evolvierbare Aktivierungsfunktionen:** Genomische Codierung an DNA Locus 21, die es Hidden-Neuronen erlaubt, sich autonom in ReLU-Gatter oder Sinus-Oszillatoren weiterzuentwickeln.
- [x] **Immersives 3-Spalten-Terminal:** Kapselung des Sandboxes-Rasters in einem scrollbaren Mittelfeld und Fixierung der Steuerung links sowie der Diagnostik rechts.
- [x] **60Hz Sidebar-Live-Metriken:** Anzeige von Live-Zellpotentialen ($s$), Feuerraten ($a$), Formeln, bias und tau beim Hovern ohne Ruckler oder Layout-Shifts.
- [x] **Löschbare Trainingsliste:** Custom Dropdown mit inline roten `✕` Deletions-Buttons und automatischem Revert auf default_run bei Löschung des aktiven Trainings.

## Verification
- **Compilation:** `npm run build` is 100% green and type-safe.
- **Unit Testing:** configured updated Vitest bounds for ReLU hidden neurons, verified 39/39 passing tests.
- **Diagnostics Preview:** live 2.2x scale animated rendering of hovered creature validated inside Chrome view.
