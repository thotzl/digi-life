# 01 — Kritische Korrektheitsfehler

Regel für alle Fixes hier: erst Repro-Test (Rust-Unit-Test oder Headless-Step), dann Fix (siehe `testing-strategies`).

---

## RV-B01 · P0 · Mitose-Nachkommen werden nie simuliert `[statisch]` — **Status: → Pool TCK-143**

**Ort:** `src/server/engine.rs:1337-1402`, Client `frontend/ocean.ts:525-564`

**Befund:** Das Kind wird nur in `newly_spawned_creatures` gepusht (`engine.rs:1399`), nie in `next_creatures`/`creatures`. Der Server simuliert es nicht. Der Client fügt es aus `newCreatures` ein und entfernt es im selben Handler wieder, weil die ID nicht in der Server-Liste steht (`ocean.ts:564`).

**Folge:** Im Ocean gibt es **keine Vererbung, keine Mutation über Generationen, keine Evolution**. `highestGeneration` steigt trotzdem (Zähler wird erhöht), Species-Records für mutierte Kinder werden angelegt — die Telemetrie suggeriert Evolution, die nicht stattfindet. Population wird ausschließlich durch Restocking (Catalogue-Klone / Zufallsgenome) gehalten. Wahrscheinlich Teilursache der in `HANDOFF.md` beschriebenen "Motionless Ocean"/"Phantom"-Symptome.

**Fix-Skizze:** Kind zusätzlich in eine `births`-Liste, nach dem Loop an `creatures` anhängen (Cap auf `max_creatures`, das aktuell ungenutzt ist). **Achtung Kopplung:** Sobald Kinder leben, explodiert die Species-Tabelle (RV-P01/RV-D03) — beide zusammen fixen.

---

## RV-B02 · P0 · Wandreflexion links/oben schiebt in die Wand `[statisch]` — **Status: ✔ 0afd463 (TCK-135)**

**Ort:** `src/shared/physics.rs:186-189` und `:198-201`

**Befund:** Bei `px < mean_radius` wird `vx = -vx.abs() * restitution` gesetzt — negative Geschwindigkeit zeigt weiter nach links. Gleiches für `py < mean_radius`. Rechte/untere Wand sind korrekt (`-abs`).

**Folge:** Kreaturen kleben an linker/oberer Wand; im Trainer zählt jeder Tick als Wandkontakt (Cooldown dämpft nur), Fitness wird verzerrt.

**Fix:** `vx = vx.abs() * restitution` bzw. `vy = vy.abs() * restitution`. Repro-Test: Agent bei `px = 1`, `vx = -2` → nach Step `vx > 0`.

---

## RV-B03 · P0 · DB-Pfad relativ zum CWD und unter `target/` `[statisch]` — **Status: ✔ cad6976 (TCK-137)**

**Ort:** `src/database.rs:4` — `DB_PATH = "target/pixel_life_local.db"`

**Folge:**
- `cargo clean` löscht den gesamten Catalogue, alle Trainer-Runs, alle Species-Records des Nutzers.
- Im installierten Release ist das CWD typischerweise das Installationsverzeichnis (Windows: `Program Files`, nicht beschreibbar). `init_db(...).expect(...)` in `engine.rs:22` lässt dann den Simulationsthread paniken → App startet ohne Simulation. `[Hypothese, hohe Wahrscheinlichkeit — kein Release-Build getestet]`

**Fix:** Pfad über Tauri `app.path().app_data_dir()` auflösen, einmal beim Setup ermitteln und an Engine + Commands übergeben (siehe RV-A04). Migration: vorhandene `target/pixel_life_local.db` einmalig kopieren.

---

## RV-B04 · P0 · `config.json` wird im Release nicht gefunden; Fallback weicht ab `[statisch]` — **Status: ✔ 5afd6d1 (TCK-138)**

**Ort:** `src/shared/types.rs:179-243`, `tauri.conf.json` (keine `bundle.resources`)

**Befund:** Config wird relativ zum CWD gesucht (`config.json`, `../config.json`, `src-tauri/config.json`). Im Bundle ist die Datei nicht enthalten. Der Fallback weicht vom echten `config.json` ab: `foodSporeCount` 600 vs 300, `thrustBaseMultiplier` 6.8 vs 3.4, `creatureMaxAgeTicks` 5400 vs 36000. Parse-Fehler (z. B. Tippfehler im JSON) fallen **still** auf den Fallback zurück.

**Folge:** Dev-Build und Release simulieren unterschiedliche Welten. "config.json ist SSOT" ist faktisch falsch.

**Fix:** `config.json` per `include_str!` als Default einkompilieren (eine Quelle), optional Override aus `app_config_dir`. Parse-Fehler laut loggen, nicht verschlucken.

---

## RV-B05 · P1 · Trainer: `epoch_ticks` pro Sandbox wird nie erhöht → Speed-Bonus konstant `[statisch]` — **Status: ✔ 1849bab (TCK-136)**

**Ort:** `src/biology/trainer_engine.rs:208`, Fitness `src/biology/scenarios.rs:150` und `:334`

**Befund:** `sb.epoch_ticks` wird nur auf 0 gesetzt, nie inkrementiert. `finish_tick` ist daher immer `Some(0)`, `speed_bonus = epoch_duration * 0.2` (bzw. `* 0.5`) für jeden erfolgreichen Kandidaten identisch.

**Folge:** Selektion auf Geschwindigkeit findet nicht statt, obwohl die Fitnessfunktion sie vorsieht.

**Fix:** In `step_trainer_sandbox_physics` `sb.epoch_ticks += 1`. Danach Fitness-Skalen neu kalibrieren (Bonus war bisher Konstante).

---

## RV-B06 · P1 · Motion-Vision überschreibt den Hauptkanal für Fleisch-Erkennung `[statisch]` — **Status: → Pool TCK-144**

**Ort:** `src/shared/brain.rs:54-72` vs. `:217-243`

**Befund:** Für Augen/Nasen wird Kanal 5 (`base_idx + 4`) jede Tick mit dem Bewegungs-Delta überschrieben. Fleisch emittiert aber genau auf Kanal 5 (IR, Gewicht 1.0). Das Fleisch-Signal erreicht das Gehirn bei Augen nur noch über Kanal 4 (Gewicht 0.5).

**Folge:** Karnivoren-Sensorik systematisch geschwächt; die Logik ist zudem an Emoji-Labels gekoppelt (`label.contains("👁️")`).

**Fix:** Entscheidung nötig (User): Motion-Vision als **eigener** 6. Kanal oder als eigener Neurontyp. Organtyp als Enum im Phänotyp statt Label-String-Matching.

---

## RV-B07 · P1 · Beißen bringt dem Angreifer keine Energie `[verifiziert: Key ungenutzt]` — **Status: → Pool TCK-145**

**Ort:** `src/server/engine.rs:1290-1323`; `bitingBaseEnergyGain` wird nirgends gelesen

**Folge:** Prädation ist für den Räuber energetisch wertlos, nur das Opfer verliert 35 Energie. Karnivorie kann sich im Ocean nur über Aas-Sporen lohnen. Zusätzlich kann ein Räuber dasselbe Opfer jeden Tick beißen (kein Cooldown) und mehrere Bite-Events pro Tick emittieren.

**Fix:** Energiegewinn = `biting_base_energy_gain * carnivory * biting_efficiency_carnivore_scale`, Bite-Cooldown pro Paar.

---

## RV-B08 · P1 · Lebensdauer ignoriert Config `[statisch]` — **Status: → Pool TCK-146**

**Ort:** `src/server/engine.rs:1191` — `max_age_ticks = mature_age * 12`

**Folge:** `creatureMaxAgeTicks` (36000) ist wirkungslos; reale Lebensdauer 3.600–32.400 Ticks je nach Genom. Widerspricht dem SSOT-Mandat.

---

## RV-B09 · P2 · UI sendet Actions, die die Engine nicht kennt `[statisch]` — **Status: teilweise ✔ 625695d (Logging, TCK-141) · Rest → Pool TCK-147**

**Ort:** `frontend/ocean.ts:759` (`SPAWN_FOOD`), `frontend/ocean.ts:535` (`CLIENT_LOG`); Engine-Match `engine.rs:1003` `_ => {}`

**Folge:** Rechtsklick-Futter-Spawn ist ein stilles No-Op. Unbekannte Actions werden ohne Log verworfen.

**Fix:** Mit RV-A02 (typisiertes Action-Enum) erledigt sich die Klasse; bis dahin `_ => eprintln!`.

---

## RV-B10 · P2 · Species-Identität = vollständiger Genom-String; Latin-Name UNIQUE `[statisch]` — **Status: → Pool TCK-148**

**Ort:** `engine.rs` (alle `species_id: genome.clone()`), `database.rs:33` (`latin_name ... UNIQUE`)

**Befund:** Jede Punktmutation erzeugt eine neue "Species". Der Latin-Name hat nur ~6.760 Strain-Kombinationen pro Präfix/Suffix; Kollisionen → `INSERT OR IGNORE` verwirft die Species still.

**Folge:** Roster ist weder biologisch sinnvoll noch vollständig; Prädations-Regel "nicht dieselbe Species beißen" greift nur bei exakten Klonen.

**Fix:** Speziation über genetische Distanz zum Gründer (wie in Skill `biology-dna` ursprünglich beschrieben), Species-ID als UUID/Counter, Latin-Name nicht UNIQUE. Design-Entscheidung — gehört in Phase 1.

---

## RV-B11 · P1 · Latin-Name nicht deterministisch `[verifiziert]` — **Status: ✔ beef2fb (TCK-162)**

**Ort:** `src/biology/dna.rs`, `derive_ecological_metrics`

**Befund (gefunden beim Golden-Master in TCK-140):** Organtypen wurden in einer `HashMap` gezählt und nur nach Anzahl sortiert. Bei Gleichstand entschied die zufällige Iterationsreihenfolge — dasselbe Genom hieß bei wiederholtem Parsen z. B. einmal `... ciliatum` und einmal `... thermum`.

**Fix:** Feste Typreihenfolge (light, chem, kinetic, thermal) als Tie-Break. Test `test_latin_name_is_deterministic`.
