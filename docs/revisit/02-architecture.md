# 02 — Architektur

## Ist-Zustand

```
Tauri main.rs
 ├─ invoke_handler: 15 Commands (api.rs) ── öffnen je Aufruf eine neue SQLite-Connection + Migration
 └─ setup: spawn_simulation_thread(window, rx)
      └─ thread::spawn(move || { ~1.970 Zeilen Closure })
           ├─ ~35 lokale `let mut` (Ocean- + Trainer-State gemischt)
           ├─ rx.try_recv() → serde_json::Value → match act_type { 16 String-Arme }
           ├─ Ocean-Tick (Physik, Brain, Metabolismus, Fressen, Beißen, Mitose, Species-DB, Restock)
           ├─ Trainer-Tick (Sandboxes, Multi-Trial, Breeding, DB)
           └─ window.emit("simulation-state", json!{...})  ← ein Event-Kanal für alles
```

---

## RV-A01 · P1 · Engine ist eine einzige God-Closure

**Ort:** `src/server/engine.rs:19-1989`

**Befund:**
- Kein `struct`, kein `impl` — der gesamte Zustand lebt als lokale Variablen in einer Closure.
- Kreatur-Konstruktion (`CreatureAgent { ... }`) ist **6× dupliziert** (Seed, Inject, Clone, Reset×2, Restock), mit jeweils leicht abweichenden Startwerten (Energie 0.6 vs 0.8, `antisense` leer vs gesetzt, `neuron_states` leer vs vorallokiert).
- Spore-Spawn mit Nutrient-Centers 3× dupliziert, `TRAINER_STATE_CHANGED`-Payload 5× dupliziert, `base_dna`-Literal 3× (+ `brain.rs`-Test + `cli_trainer.rs`).
- Ocean und Trainer teilen sich `last_emit_time` — Moduswechsel verschiebt Emissions-Takt.
- Nicht testbar: Engine braucht ein echtes `tauri::WebviewWindow`.

**Zielbild (Phase 1):**
```
struct OceanSim   { creatures, food, world, rng, ids, config } impl { fn step(&mut self) -> Vec<SimEvent> }
struct TrainerSim { sandboxes, hyperparams, generation, ... }   impl { fn step(&mut self) -> Vec<SimEvent> }
struct Engine     { ocean, trainer, mode, repo: Box<dyn Repository> }
fn run_loop(engine, rx: Receiver<Action>, sink: impl EventSink)   // Tauri-Window nur als EventSink-Impl
```
- `step()` ist pur bezüglich IO → headless testbar, Basis für CLI-Trainer und Balancing.
- Ein `CreatureFactory::spawn(genome, opts)` ersetzt die 6 Kopien.

**Nicht-Ziel:** ECS-Framework (bevy_ecs etc.) einführen. Bei 25–45 Agenten reicht `Vec<Creature>`.

---

## RV-A02 · P1 · Stringly-typed IPC in beide Richtungen

**Befund:**
- Frontend → Rust: `invoke("handle_client_action", { action: JSON.stringify({type: "..."}) })` — doppelt serialisiert, ungeprüft. Unbekannte Typen und fehlende Felder werden still ignoriert (`as_u64()` → `None`).
- Rust → Frontend: ein Event `simulation-state` mit `json!`-Payloads, ~12 Typen, Felder teils camelCase, teils snake_case (`load_learned_synapses`, `sandbox_id` vs `warpSpeed`, `runId`).
- `ocean.ts` und `trainer.ts` lauschen beide auf alle Events.

**Ziel:**
- `#[derive(Deserialize, TS)] #[serde(tag = "type")] enum Action { ToggleSimulation { running: bool }, ... }` → Tauri-Command nimmt `Action` direkt (kein `JSON.stringify`).
- `#[derive(Serialize, TS)] enum SimEvent { ... }`; getrennte Events `ocean://telemetry`, `trainer://telemetry`, `db://changed`, `log`.
- Beide Enums via ts-rs exportiert → Compile-Fehler im Frontend bei Drift.

---

## RV-A03 · P1 · ts-rs-Bindings werden generiert, aber nicht verwendet

**Ort:** `bindings/*.ts` (17 Dateien) vs. `frontend/shared/types.ts` (210 LOC, handgeschrieben)

**Befund:** Kein Import aus `bindings/` im Frontend `[verifiziert]`. README behauptet das Gegenteil. Handgeschriebene Typen weichen bereits ab (z. B. `OrganelleLocus` hat `x?/y?`, Rust `SensoryPatch` hat `geneStartIndex/geneEndIndex`).

**Ziel:** `frontend/shared/types.ts` re-exportiert aus `bindings/`; nur rein clientseitige Typen bleiben lokal. `bindings/` per CI auf Aktualität prüfen (`cargo test` + `git diff --exit-code bindings/`).

---

## RV-A04 · P2 · Keine geteilte Ressourcen-/Pfad-Verwaltung

**Befund:** Jeder der 15 Commands ruft `init_db(DB_PATH)` → neue Connection, `PRAGMA`, 6× `CREATE TABLE IF NOT EXISTS`, Migrationscheck. Engine-Thread hält eine eigene Connection. Kein WAL-Modus → Lese-Commands und Engine-Writes können sich blockieren (`SQLITE_BUSY` → als String-Error an UI).

**Ziel:** Beim Setup einmal migrieren; `Repository` mit Connection-Pool oder `Mutex<Connection>` als Tauri-State; `PRAGMA journal_mode=WAL`.

---

## RV-A05 · P2 · Doppelte Wahrheit über Weltgrößen und Konstanten

- `19200 × 10800` hart in Rust (`engine.rs:30`, `trainer_engine.rs:115`) und TS (5 Stellen `[verifiziert]`).
- `"/ 25"` Populationsanzeige hart in `ocean.ts:586`.
- Trainer-Epochendauer `300/1800` und Kammergröße `1000/3500` als Literale in Engine, nicht im Scenario-Plugin.

**Ziel:** Weltgröße kommt mit `INIT_STATE`; Scenario-Plugin liefert `chamber_size()`, `epoch_ticks()`, `spawn_min_distance()`.

---

## RV-A06 · P3 · Gemischte Einheiten bei Zeitstempeln

`now_millis` heißt überall so, ist aber an 9 Stellen Sekunden (`as_secs`), an 2 Stellen Millisekunden; `creature_catalogue.created_at` ist ein SQLite-Text-Timestamp. Siehe RV-D04.
