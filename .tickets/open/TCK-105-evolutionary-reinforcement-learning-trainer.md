---
id: TCK-105
title: Evolutionary Reinforcement Learning Trainer für fitte Progenitor-Zellen
status: open
assigned: Gemini-CLI
created_at: 2026-08-19
---

# TCK-105: Evolutionary Reinforcement Learning Trainer für fitte Progenitor-Zellen

## Beschreibung
Entwicklung einer hocheffizienten, isolierten Trainingsumgebung zur Neuroevolution. Über ein Grid aus N Mini-Canvases können Benutzer in Echtzeit beobachten, wie Kreaturen in 5-Sekunden-Epochen das zielgerichtete Navigieren zu Futtersporen erlernen, um fitte Basis-Genome zu exportieren.

## Teilaufgaben
- [ ] **Frontend-Layout (`trainer.html` / `src/trainer.ts`):** Erstellung der Oberfläche mit einem responsiven Grid aus N Mini-Canvases und einem Dashboard (Tuning-Variablen, Generationen-Tracker, Export-Button).
- [ ] **Isolierte Physik-Sandbox:** Implementierung einer schlanken In-Browser-Physikschleife pro Zelle, die eine Kreatur und eine Spore simuliert.
- [ ] **Fitness-Heuristik:** Integration des Bewertungsschlüssels (Erhalt bei Verzehr im Zeitlimit, Distanzabzug bei Misserfolg).
- [ ] **Evolutionäre Operatoren:** Culling der unteren 50%, Klonierung der Elite und kontrollierte Mutation zur Neubesetzung.
- [ ] **Champion-Export & Hauptsim-Integration:** Möglichkeit zum Export des Champion-Genoms und Einspeisung als Basis-Template für "Founder Cells" im Hauptbecken.
