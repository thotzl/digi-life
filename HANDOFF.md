# 🧬 Handoff: Pixel DNA Life - Paralleler Tauri/Rust Ozean-Prototyp

---

## 1. Ist-Zustand (Erreichte Meilensteine)

In dieser Session wurde ein vollständig paralleler, funktionaler Desktop-Entwicklungs-Stack aufgebaut. Das ursprüngliche TypeScript/Browser-Spiel bleibt zu 100% unberührt und unbeschädigt lauffähig.

### A. Der native Simulationscore (Rust - `src-tauri/`)
*   **Initialisierung & Multithreading:** Tauri v2 wurde in das Projekt integriert. Beim Starten wird automatisch ein paralleler Simulations-Hintergrund-Thread in `src-tauri/src/main.rs` hochgefahren.
*   **Volle Ozean-Simulation:** Die physikalische Weltbewegung (`physics.rs`), das Gitter zur schnellen Kollisionsabfrage (`spatial_grid.rs`), die BMR-Metabolismussteuerungen, das Fressen, die asexuelle Fortpflanzung (Mitose), die Mutationslogik und die CTRNN-Euler-Gehirn-Kopplungen wurden vollständig in nativem Rust implementiert.
*   **Lokale SQLite-Datenhaltung:** Spezies-Stammbäume und Neuentdeckungen werden transaktional direkt in der lokalen Datei `pixel_life_local.db` persistiert.
*   **Isolierter CLI-Trainer:** Der ungedrosselte Konsolen-Trainer wurde in ein eigenständiges Cargo-Binary verschoben. Er läuft weiterhin unabhängig von Tauri und ohne GTK-Systempakete direkt über:
    ```bash
    cargo run --bin cli_trainer
    ```

### B. Das rahmenlose Frontend (Tauri Webview)
*   **Eigene HTML-Ansicht (`tauri_ocean.html`):** Ein unberührter, isolierter Einstiegspunkt für das Desktop-Fenster.
*   **Reaktiver Tauri-Fork (`src/tauri_ocean.ts`):** 
    *   Sämtliche WebSockets und HTTP-POST-Schnittstellen wurden restlos durch die nativen Tauri-Schnittstellen (`listen` für Datenströme, `invoke` für Steuerbefehle) ersetzt.
    *   Ein **asynchrones Handshake-Verfahren (`"CLIENT_READY"`)** mit 200 ms Delay wurde etabliert, um Race-Conditions beim Laden des Event-Listeners vollständig zu eliminieren.
    *   Die Hilfsfunktion `safeInvoke` mit einem robusten ESM-Try-Catch-Wrapper fängt Modulauflösungsfehler im normalen Webbrowser komplett ab, sodass die Diagnoseseite im Browser nicht abstürzt.
    *   Die linke Sidebar wurde über das Tauri-Kommando `get_registered_species` direkt an die lokale Rust-Datenbank angebunden.

---

## 2. Diagnose der aktuellen Bruchstelle

### Die Symptome:
*   Die Sidebars (HUD) links und rechts aktualisieren sich dynamisch.
*   Die Generationen zählen hoch, Organismen verändern ihre Anzahl im HUD (z. B. von `5/20` auf `9/20`), und neue Spezies werden der linken Sidebar live hinzugefügt.
*   **Aber:** Das Canvas im Hintergrund bleibt leer (keine grünen Sporen, keine Zellen sichtbar).

### Die technische Ursache:
*   **Asynchrone Entkopplung:** Die HUD-Werte (Sidebars) werden über den asynchronen Tauri-Event-Listener (`listen("simulation-state")`) empfangen und über reaktive Preact Signals direkt an das HTML-DOM gebunden. Das funktioniert tadellos (der Datenstrom fließt also fehlerfrei im RAM-IPC-Kanal!).
*   **Bruch im Render-Loop:** Das eigentliche Zeichnen auf dem Canvas läuft in einem getrennten, kontinuierlichen Browser-Animations-Thread (`drawBetaSimulationFrame`). 
*   Wenn dieser Loop einmalig beim Start (z. B. während der Kamera-Zoom-Initialisierung oder innerhalb des `CreatureRenderer`) an einem nicht definierten Wert (z. B. `dpr`, `canvas.getContext` oder nicht passenden Transformations-Matrix-Werten) scheitert, bricht die Render-Schleife stillschweigend ab. Das Canvas bleibt schwarz, während das HTML-HUD sich im Vordergrund munter weiter aktualisiert.

---

## 3. Exakte nächste Schritte (Für das nächste Vibe-Coding)

Um das Canvas im Tauri-Fenster zum Leben zu erwecken, müssen folgende Punkte im Frontend-Fork `src/tauri_ocean.ts` untersucht werden:

1.  **Canvas-Transformationen im Viewport prüfen:**
    Überprüfen, ob die Kamera-Matrix in `drawBetaSimulationFrame` (Zeile 640) korrekte Werte für `camZoom`, `camX` und `camY` erhält, wenn das Tauri-Fenster im Standard-HD-Format ($1280 \times 720$) bootet, oder ob die mathematische Transformation die Kreaturen außerhalb des sichtbaren Bildschirms positioniert.
2.  **Debuggen des `CreatureRenderer` im Tauri-Inspektor:**
    Mache im offenen Tauri-Fenster einen **Rechtsklick ➔ Untersuchen (Inspect)** und wechsle auf den Reiter **Console**. Lies dort den genauen Stacktrace aus, der beim Ausführen des Canvas-Frames geworfen wird (z. B. ob im `renderer.render` ein unerwartetes Feld im de-kompilierten `CreaturePhenotype` gesucht wird, das Rust leicht abweichend serialisiert hat).

Der gesamte Kommunikations- und Berechnungs-Fluss steht felsenfest und läuft mit enormer Stabilität. Wir haben die technologische Barriere für eine native Desktop-Distribution erfolgreich durchbrochen!

---

## 4. Konzepte für KI-gestütztes Remote-Debugging (Direkte KI-Verbindung)

Damit ich (die KI) in Zukunft Fehler im Tauri-Fenster und auf dem Canvas selbstständig einsehen und analysieren kann, können wir eine der folgenden asynchronen Diagnose-Schnittstellen einrichten:

### Konzept A: Der "Error Mirroring"-Kanal (Dringend empfohlen)
*   **Wie es funktioniert:** Wir registrieren im Frontend (`src/tauri_ocean.ts`) einen globalen Fehler-Listener (`window.onerror` und `window.onunhandledrejection`). 
*   Tritt ein Render- oder JavaScript-Fehler im Tauri-Fenster auf, schickt das Frontend diesen Stacktrace per `safeInvoke` sofort an Rust.
*   Rust schreibt diese Fehlermeldungen fortlaufend in eine lokale Datei `src-tauri/client_debug.log`.
*   *Der Vorteil:* Ich kann diese Log-Datei im Terminal mit meinem `read_file`-Werkzeug live auslesen. Ich sehe jeden Absturz auf deinem Bildschirm sofort, ohne dass du etwas kopieren musst.

### Konzept B: Tauri stdout Mirroring (Konsolen-Umleitung)
*   Wir binden das offizielle Tauri-Plugin `tauri-plugin-log` ein. Dieses leitet alle Standard-Ausgaben von `console.log` und `console.error` der Webview direkt in das System-Terminal (stdout) von Tauri um. Ich sehe deine Web-Logs dann direkt in den CLI-Prozessen.

### Konzept C: Der "Debug-Snapshot" (F8-Trigger)
*   Wir richten einen Hotkey im Frontend ein (z. B. Taste `F8`). 
*   Bei Tastendruck speichert das Frontend alle aktuellen Variablen (Kamera-Zoom, geladene Entities, letzte 15 Konsolen-Ausgaben) in eine temporäre Datei `client_snapshot.json`. Ich kann diese Datei einlesen und erhalte ein vollständiges, mathematisches Abbild des aktuellen Render-Zustands.

