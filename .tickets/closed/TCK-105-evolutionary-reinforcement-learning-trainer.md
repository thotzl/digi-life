Frontmatter:
id: TCK-105
title: Evolutionary Reinforcement Learning Trainer für fitte Progenitor-Zellen
status: closed
assigned: Gemini-CLI
created_at: 2026-08-19
closed_at: 2026-08-19
---

# TCK-105: Evolutionary Reinforcement Learning Trainer für fitte Progenitor-Zellen

## Beschreibung
Entwicklung einer hocheffizienten, isolierten Trainingsumgebung zur Neuroevolution. Über ein Grid aus N Mini-Canvases können Benutzer in Echtzeit beobachten, wie Kreaturen in 5-Sekunden-Epochen das zielgerichtete Navigieren zu Futtersporen erlernen, um fitte Basis-Genome zu exportieren.

## Teilaufgaben
- [x] **Frontend-Layout (`trainer.html` / `src/trainer.ts`):** Erstellung der Oberfläche mit einem responsiven Grid aus N Mini-Canvases und einem Dashboard (Tuning-Variablen, Generationen-Tracker, Export-Button).
- [x] **Isolierte Physik-Sandbox:** Implementierung einer schlanken In-Browser-Physikschleife pro Zelle, die eine Kreatur und eine Spore simuliert.
- [x] **Fitness-Heuristik:** Integration des Bewertungsschlüssels (Erhalt bei Verzehr im Zeitlimit, Distanzabzug bei Misserfolg).
- [x] **Evolutionäre Operatoren:** Culling der unteren 50%, Klonierung der Elite und kontrollierte Mutation zur Neubesetzung.
- [x] **Champion-Export & Hauptsim-Integration:** Möglichkeit zum Export des Champion-Genoms und Einspeisung als Basis-Template für "Founder Cells" im Hauptbecken.

## Verification
- **Compilation:** `npm run build` is 100% green and type-safe.
- **Unit Testing:** `npm run test` passes all 39 tests cleanly inside `src/biology/dna.test.ts`, `src/server/simulation.test.ts`, and others.
- **Browser Automation Verification:** Run transition and loops verified inside Chrome DevTools over 100+ generations.
