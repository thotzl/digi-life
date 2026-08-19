---
id: TCK-108
title: Migration des Evolutionary Trainers ins Node.js-Backend mit WebSocket-Replikation
status: open
assigned: Gemini-CLI
created_at: 2026-08-19
---

# TCK-108: Migration des Evolutionary Trainers ins Node.js-Backend mit WebSocket-Replikation

## Beschreibung
Verlagerung des gesamten physikalischen und neuronalen Trainings-Loops (N Sandboxen, CTRNN-Integration, Kollisionsabfragen und Selektion) aus dem single-threaded Browser-Frontend in das hochperformante Node.js-Backend. Etablierung eines reinen Client-Consumer-Modells über WebSockets (analog zum Live-Ozean).

## Vorteile & Ziele
- **Keine Browser-Drosselung:** Der Trainer läuft ungedrosselt im Node.js-Prozess, völlig unabhängig von Browser-Mainthread-Blockierungen, Tab-Suspension oder Render-Nadelöhren.
- **Echtes, permanentes Headless-Training:** Das Training läuft im Server-Prozess lautlos weiter, selbst wenn der Benutzer den Browser-Tab schließt oder den Computer sperrt.
- **WebSocket Replikation:** Das Frontend abonniert die Positions- und Neuronendaten der aktiven Sandboxen bei einer gedrosselten Rate (z. B. 25Hz) und zeichnet diese ressourcenschonend, ohne selbst Physik rechnen zu müssen.
- **Tiefes Logging & SQLite-Integration:** Server-seitig können alle Mutations-Stammbäume, Generationen-Historien und Fitnesskurven hochpräzise protokolliert und ohne Latenz direkt in SQLite persistiert werden.

## Technische Teilschritte

### 1. Backend-Rechenkern (`src/server/trainerEngine.ts`)
- Erstellung eines server-seitigen `TrainerEngine` Moduls, welches $N$ isolierte Sandboxes (Creatures + Spores + SpatialGrid) verwaltet.
- Implementierung der Steuerbefehle (`START`, `PAUSE`, `RESET`, `SET_SPEEDUP`, `SET_HYPERPARAMS`) über WebSocket-Nachrichten.
- Ausführen der 300-Tick-Epochen in ungedrosselten Node-Zyklen (Warp-Speed) oder synchron getakteten Intervallen.

### 2. WebSocket-Protokoll & Replikations-Stream
- Implementierung des Streams `TRAINER_STATE` vom Server zum Client, der die Positionen, Biegungswinkel, Sporen und neuronalen Aktivierungen der aktuell fokussierten Sandbox überträgt.
- Bereitstellung schlanker REST-Endpunkte für administrative Aufgaben (z. B. neue Sessions anlegen, Sessions löschen).

### 3. Frontend-Refactoring (`src/trainer.ts`)
- Vollständiger Rückzug der Physikschleifen (`stepPhysics`) und CTRNN-Integrationen (`executeBrain`) aus dem Frontend.
- Umbau der Canvases zu rein passiven Darstellern, die den vom WebSocket gelieferten Server-Zustand zeichnen.
- Beibehaltung der Diagnostik-Hover-Metriken und des Brain-SVG-Generators basierend auf den Server-seitig berechneten Neuron-Potentialen.

### 4. SQLite-Optimierung
- Direktes server-seitiges Schreiben von Zwischenergebnissen in die SQLite-Datenbank am Generations-Ende, wodurch sämtliche HTTP-Latenzen der bisherigen POST-Requests entfallen.
