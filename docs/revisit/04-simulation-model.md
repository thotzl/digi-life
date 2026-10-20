# 04 — Simulationsmodell (Biologie, Brain, Physik, Trainer)

Hier geht es um **semantische** Probleme: Code tut, was er sagt, aber das Modell erzeugt nicht das intendierte Verhalten. Viele Punkte sind Balancing-Fragen und brauchen Messung (Headless-Läufe, siehe RV-T03), bevor man Konstanten dreht.

---

## RV-S01 · P1 · Tote Config: 16 von 44 Regeln werden nie gelesen `[verifiziert]` — **Status: → Pool TCK-149**

Ungenutzt in `SimulationRules`:
`creatureMaxAgeTicks`, `creatureMatureAgeFloor`, `lamarckianAssimilationAgeFloor`, `adrenalineMaxMultiplier`, `adrenalineIncreaseRate`, `adrenalineDecayRate`, `bioluminescenceFlashCost`, `biomeHazardDamageScale`, `biomeAlgaeSporeSpawnThreshold`, `bitingBaseEnergyGain`, `reproductionStomachThresholdFloor`, `steerTorqueBaseMultiplier`, `predatorSavageThrustThreshold`, `predatorSavageThrustMultiplier`, `dragLateralCoefficient`, `restockFounderGeneInheritChance`.

Zusätzlich auf `AppConfig`-Ebene ungenutzt: `maxCreatures`, `seed`. In `config.json` stehen 9 Keys, die gar nicht im Struct existieren (`obstacles*`, `vents*`, `disableVentsForce`) und von serde still ignoriert werden.

**Konsequenzen:**
- **Adrenalin ist eine Konstante (1.0)** — nichts erhöht es. Der Brain-Input "Adrenaline Level" ist immer 0, der Metabolismus-Surcharge immer 1.
- **Biolum-Output** (Motor-Neuron 3) und "Reserved" (4) haben keine Wirkung, kosten aber Synapsen und damit BMR (RV-S03).
- **Biome** werden generiert und gerendert, haben aber keinen Simulationseffekt.

**Entscheidung pro Key (User):** implementieren, oder aus Config + Struct entfernen. Ziel: `#[serde(deny_unknown_fields)]` und jeder Key hat mindestens eine Lesestelle.

## RV-S02 · P1 · Hypothese "Motionless Ocean": Schub skaliert für viele Genome gegen Null `[Hypothese]` — **Status: → Pool TCK-150**

**Ort:** `src/shared/physics.rs:91-103`, `:163-167`

Überschlag: `thrust = stiffness · (pulse·1000)² · 3.4 · eta · out_thrust`. Bei `pulse = 0.0015` (Genom-Minimum) und `stiffness = 0.15` → ~1.1 · eta. Masse `r^1.5 · L/25` ≈ 230–1.180. Mit Dämpfung 0.94 ergibt sich eine Endgeschwindigkeit im Bereich 10⁻³–10⁻² px/Tick in einer 19.200 px breiten Welt. Für solche Genome ist die Kreatur faktisch unbeweglich. Zusätzlich: `out_thrust` ist `tanh` ∈ [-1, 1], negativ = rückwärts.

Diese Hypothese erklärt das in `HANDOFF.md` offene Symptom besser als die dort vermutete Zeitskala. **Vor Fix messen:** Verteilung der Endgeschwindigkeit über 1.000 Zufallsgenome (Headless-Test).

## RV-S03 · P1 · BMR bestraft Gehirngröße multiplikativ und quadratisch `[statisch]` — **Status: → Pool TCK-151**

**Ort:** `src/biology/dna.rs:1417-1430`

`brain_complexity_multiplier = 1 + synapses · 0.06` und zusätzlich `(1 + k_count · 0.08)`. Die Synapsenzahl wächst mit `(7 + 10·Organe + H) × (4 + H)` (exuberant fully-connected). Ein Organ ergibt +10 Thalamus-Quellen → ~+40–120 Synapsen → BMR ×3–×8. Der Ocean kompensiert mit dem Hack `* 0.45` (`engine.rs:1171`).

**Folge:** Starker Selektionsdruck **gegen** Sinnesorgane — widerspricht dem Designziel "Organe emergieren durch Selektion".

**Ziel:** Kosten pro *aktiver* (nicht geprunter) Synapse linear, Organe mit eigenem Kostenterm; `0.45`-Hack in Config überführen oder entfernen.

## RV-S04 · P2 · Hebbian Learning ohne Normalisierung, auch auf Reflexbahnen `[statisch]` — **Status: → Pool TCK-152**

**Ort:** `src/shared/brain.rs:109-137`

- Reines `Δw = η · pre · post` ohne Oja/BCM-Normalisierung → positive Rückkopplung, Gewichte laufen über lange Lebenszeiten (36k Ticks) gegen die Clamp-Grenze ±4.
- Die fest verdrahteten Thalamus-Reflexsynapsen (±1.0, `dna.rs:1323-1361`) werden mitgelernt und können geprunt werden → Sum/Diff-Semantik zerfällt.
- `_hebbian_decay` (stiffness decay) wird übergeben und ignoriert.

**Ziel:** Synapsen-Flag `plastic: bool`; Reflexe fix. Normalisierte Lernregel; Messung der Gewichtsverteilung nach N Ticks als Test.

## RV-S05 · P2 · Ocean ohne funktionierende Vererbung von Gelerntem — **Status: → Pool TCK-152**

Abhängig von RV-B01. Zusätzlich: `lamarckian_assimilation_chance` wird als **Skalierungsfaktor** auf Gewichtsoffsets verwendet (`engine.rs:1362`), nicht als Wahrscheinlichkeit; bei Klon-Geburten (65 %) wird `child = agent.clone()` inkl. `neuron_states` und vollständig gelernten Gewichten übernommen, bei Mutanten nur 25 % des Offsets. Inkonsistente Semantik.

## RV-S06 · P2 · Brain-Layer-Latenz durch ID-Reihenfolge `[statisch]` — **Status: → Pool TCK-153**

Neuron-IDs: `[Inputs][Outputs][Thalamus][Hidden]`. Der Euler-Pass läuft aufsteigend → Outputs werden **vor** Thalamus und Hidden berechnet und lesen deren Aktivierung vom Vortick. Sensor → Motor hat damit 2 Ticks Latenz. Für CTRNN zulässig, sollte aber eine bewusste Entscheidung sein (dokumentieren oder Reihenfolge `Inputs → Thalamus → Hidden → Outputs`).

## RV-S07 · P2 · Ocean-Ökologie extrem dünn `[Hypothese]` — **Status: → Pool TCK-150**

300 Sporen auf 19.200 × 10.800 px (≈ 1 Spore pro 830 × 830 px), Sensorreichweite `scale · 550` (190–990 px), 25 Agenten. Kombiniert mit RV-S02 ist zufälliges Finden von Nahrung selten. Balancing erst nach RV-B01/S02 und mit Messreihen anpassen (Skill `simulation-analytics`).

## RV-S08 · P2 · Trainer: inkonsistente Spawn-Distanzen in Exploration `[statisch]` — **Status: → Pool TCK-154**

- Initialisierung: `min_dist = 650` (`scenarios.rs:195`)
- Re-Trial im Multi-Trial: `min_dist = 1200` (`engine.rs:1743`)
- Respawn nach Fressen: `550` (`trainer_engine.rs:218`)

Trial 1 ist damit systematisch leichter als Trial 2/3. Zudem sind Rejection-Sampling-Schleifen (`while !valid_spawn`) ohne Iterationslimit — bei künftigen kleineren Kammern/größeren Distanzen Endlosschleife.

**Ziel:** Werte gehören ins Scenario-Plugin (`spawn_min_distance()`), Re-Trial ruft `plugin.initialize` statt duplizierter Logik (`engine.rs:1745-1820` dupliziert `initialize`).

## RV-S09 · P2 · Trainer-Gen-1 ignoriert `base_dna` — **Status: → Pool TCK-154**

`rebuild_sandbox_grid` übergibt `base_dna`, `init_rust_sandbox` ersetzt es bei `origin == "random"` durch `generate_random_genome(256)`. Reine Zufallsgenome enthalten selten `NEU`/`SY`/`EYE`-Promotoren — Gen 1 startet meist blind und mit Fallback-Brain. Bewusst so? (CHANGELOG sagt "pristine wildtypes are born blind" → vermutlich gewollt, dann `base_dna`-Parameter entfernen.)

## RV-S10 · P3 · Modell-Semantik in Freitext-Strings — **Status: → Pool TCK-155**

Aktivierungsfunktion (`Option<String>` "relu"/"sigmoid"/...), Organtyp (Emoji-Label), Symmetrie (`"vertical"/"quad"`), Szenario (`"standard"/"exploration"`), Origin (`"elite"/"hof"/...`), Diet-Class — alles Strings. Ein Tippfehler fällt still auf den Default. → Enums (mit ts-rs-Export).

## RV-S11 · P3 · `cli_trainer` testet ein veraltetes Modell — **Status: → Pool TCK-158**

`src/bin/cli_trainer.rs` füttert `vec![1.0, 0.0, 0.5]` als Inputs (3 Sensoren-Modell von vor TCK-122) und schreibt in dieselbe Produktiv-DB. `npm run sim:balance` liefert damit keine aussagekräftigen Werte. Ersetzen durch Headless-Runner auf Basis von RV-A01 (`OceanSim::step` / `TrainerSim::step`).
