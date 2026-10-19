# 06 — Frontend (Vite + TypeScript + Preact Signals Core)

Build-Status: `tsc --noEmit` 0 Fehler, `vite build` OK `[verifiziert]`. Strict-Mode aktiv. Das Frontend ist in besserem Zustand als der Rust-Teil — die Probleme sind vor allem Vertrags-Drift und Legacy-Reste.

Struktur: `main.ts` mountet 4 Views (DOM per Template-Strings), danach dynamischer Import von `ocean.ts` (869 LOC) und `trainer.ts` (670 LOC) als **Seiteneffekt-Module** mit globalem Zustand und `document.getElementById`-Kopplung an die Views.

---

## RV-F01 · P1 · Handgeschriebene Typen statt ts-rs → siehe RV-A03

`any` kommt ~41× vor `[verifiziert]`, überwiegend an der IPC-Grenze (`event: any`, `safeInvoke(): Promise<any>`). Mit typisierten Events (RV-A02) fallen die meisten weg.

## RV-F02 · P1 · `innerHTML` mit Daten aus DB/Nutzereingabe bei deaktivierter CSP

**Ort:** 35 `innerHTML`-Zuweisungen, davon 11 mit Template-Interpolation `[verifiziert]`; `tauri.conf.json` → `"csp": null`.

Latin-Namen, Catalogue-Namen (vom Nutzer umbenennbar via `rename_catalogue_creature`) und Log-Messages landen ungefiltert im DOM. In einer Tauri-App hat injiziertes Script Zugriff auf alle registrierten Commands (inkl. `delete_from_catalogue`, `clear_trainer_history`). Risiko lokal begrenzt, wird aber relevant, sobald Genome/Catalogue-Einträge importiert/geteilt werden (TCK-111, WHITEPAPER "Global Ecosystem").

**Fix:** `textContent` für Datenfelder bzw. Escape-Helper; CSP setzen (`default-src 'self'`).

## RV-F03 · P1 · Legacy-Genommodell in der UI

**Ort:** `frontend/signals.ts` — `getLocusDescription(i)` beschreibt ein **positionsbasiertes** Genom (Locus 0 = Symmetrie, Locus 7 = Radius, ...), `computeActiveGeneSpans` sucht Start-Motive `"ST"`/`"GO"`. Der Rust-Compiler ist seit TCK-116 promoter-basiert (`COL`, `SIZ`, `EYE`, ...).

**Folge:** Tooltips/Genom-Visualisierung zeigen falsche Bedeutungen. `chromatinState` vom Backend ist korrekt, die Interpretation im Client nicht.

**Fix:** Gene-Spans inkl. Promotor-Name vom Backend liefern (`parse_genome` kennt sie bereits, `epigenetic_logs` enthält sie als Text) — Client nur noch rendern (SSOT).

## RV-F04 · P2 · Fehler werden an der IPC-Grenze verschluckt

`safeInvoke` fängt jeden Fehler, loggt `console.warn` und liefert `null`. Gemischt mit direktem `invoke` (ocean.ts). Kein UI-Feedback bei fehlgeschlagenem Speichern/Löschen.

## RV-F05 · P2 · Seiteneffekt-Module und doppelte Event-Listener

`ocean.ts` und `trainer.ts` registrieren beim Import globale `listen("simulation-state")`-Handler, die nie abgemeldet werden, und verarbeiten jeweils alle Event-Typen. Navigation steuert Backend-Modus über 3–4 fire-and-forget-Actions (`router.ts:16-29`) ohne Bestätigung → Race-Fenster zwischen Moduswechsel und Telemetrie.

**Ziel (Phase 1, minimal):** pro View `mount()/unmount()` mit Listener-Lifecycle; Moduswechsel als ein Command mit Rückgabe.

## RV-F06 · P2 · Doppelter Preview-Modal-Code

Catalogue-Preview mit eigenem `requestAnimationFrame`-Loop existiert in `ocean.ts:~300` und `UnifiedDiagnosticsPanel.ts:~821` (laut HANDOFF "identisch"). Eine Komponente.

## RV-F07 · P3 · Unbenutzte/fehlplatzierte Abhängigkeiten

- `zustand` in `dependencies`, nirgends importiert `[verifiziert]`.
- `concurrently`, `tsx` in devDependencies ohne Script-Nutzung.
- Stryker-Mutation-Testing konfiguriert ohne Tests (siehe RV-T02).
- `ActiveView` enthält `"creator"`, ohne View.

## RV-F08 · P3 · Hartkodierte Werte

Weltgröße 19200/10800 (5×), Populationsanzeige `/ 25`, Fallback `selectedAge = 2700` für Fossilien. → aus `INIT_STATE`/Config.
