---
sources: [wiki/architecture/system-overview.md, wiki/architecture/simulation-engine.md, wiki/architecture/ipc-protocol.md, wiki/frontend/structure.md, wiki/simulation/trainer.md, wiki/biology/genome-compiler.md, wiki/biology/metabolism-and-reproduction.md, tickets/closed/TCK-115-master-architectural-refactoring.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-156-engine-decomposition.md, tickets/closed/TCK-157-typed-ipc-contract.md, tickets/closed/TCK-160-frontend-structure.md, tickets/closed/TCK-202-r5-model-corrections.md]
---

# Replaced models and behaviour (history)

Designs, rules and defects that the current-state articles no longer describe, moved out of them in TCK-203 so those
articles hold only the current state. Each entry names the ticket that replaced it. The per-correction table of the R5
model change is in [[wiki/history/model-corrections-r5]].

## Architecture (TCK-107..115, TCK-124)

- The project started as a browser/Node.js prototype with WebSocket replication and isomorphic TS modules (TCK-107,
  TCK-108). The Rust core (TCK-110) and the Tauri shell (TCK-109) replaced it; TCK-115 removed the Express server and
  the duplicate TS parsers; TCK-124 moved the crate to the repo root. Anything mentioning WebSocket, `src-tauri/` or
  JSON database files describes the retired design.
- The project was called Pixel DNA Life; the app is BroodBattle since the first Windows release (TCK-195).

## Engine (before TCK-156)

- One ~1,465-line closure in `engine.rs` held the Ocean step, action dispatch and the SQL; Ocean and trainer shared one
  emit clock.
- Every navigation to the trainer rebuilt its population from the store.
- A failing engine DB connection panicked the engine thread (`expect`) without any message to the UI.

- Ocean behaviour change of TCK-156 (by design, no Ocean reference existed then): seeded RNG instead of `thread_rng`, so every Ocean run differs from the former ones; eaten-spore respawn and corpse decomposition applied after each creature's step instead of inside its eat loop (creature arithmetic otherwise as in the former `engine.rs`); nutrient centres survive a restart (before, a restart drew new centres for the restored spores); the stale "15 unique wildtype cells" log line after a reset removed.
- Ocean mitosis children before TCK-143: reported to the UI but never simulated, and every mutated child created a species that was marked extinct one tick later (RV-B01).

## IPC (before TCK-157)

- An unknown action type was only logged by the engine, missing fields were ignored silently (RV-A02).
- Events were `json!` payloads, which widened f32 fields to f64 (0.9 → 0.8999999761581421).
- The frontend called Tauri through `safeInvoke` (`frontend/api.ts`), which returned `null` on errors (RV-F04).

## Frontend (before TCK-160)

- Every navigation paused the Ocean, so entering it always stopped it; the boot pushed the client trainer defaults
  (N 16, warp 1, `default_run`) over the restored run settings.
- One selection signal per view: a sandbox id could be read as an Ocean creature id, "Assign From Catalogue" got an
  Ocean or catalogue id, a catalogue string id sat in a number signal.
- "Save to Catalogue" read a nonexistent `phenotype.fitness` (always 0) and stored the compiled weights.
- Ocean bite flashes were drawn at the wrong place (canvas pixels inside the camera transform); the hover radius came
  from the trainer's phenotype cache (always the 100-unit fallback in the Ocean).
- Diet thresholds 0.55 / 0.3 in roster, catalogue and pickers (trainer 0.6 / 0.4); the badge colour was guessed from
  `genome.charCodeAt(17)`; `shared/types.ts` and `signals.ts` held hand-written model types (RV-A03).
- `styled` compiled once per element, so every list refresh and dialog added stylesheet rules; dialogs appended
  `<style>` tags with conflicting `.modal-card` rules.
- The renderer drew every organ with `expressionStyle ≥ 0.72` as a limb, eyes included (until TCK-202).

## Trainer (before TCK-202)

- The trainer had its own clock (`fmath` vs the Ocean's std `sin`), its own food-drift order (after each creature's
  move) and its own eat radius (`max(r + 10, ..)`, smaller than the Ocean's for r > 12), so trainer-bred creatures met
  another catch geometry in the Ocean.
- Surviving elites were re-added to the hall of fame every generation, so the HoF slots mostly re-mutated copies of
  the current parents.
- Before P7 (TCK-174) a mutant applied `round(genome.len() · mutation_rate)` operator calls (0.06 → ≈ 15 at 256 bases).

## Energy model (before P6, TCK-151)

- The cost was a multiplicative BMR compiled into the phenotype, `(body area + organ tax) · (1 + organs · 5 · 0.08) ·
  1.4 · stiffness · (1 + synapses · 0.06)`, drained in the Ocean as `basal_metabolic_rate · bmrBaseScale ·
  basalMetabolicRateMultiplier · 0.45`; the trainer had no drain. One sense organ raised the BMR several-fold through
  the synapse factor (RV-S03).
- Removed (TCK-151): `brain_complexity_multiplier`, the organ tax as a factor, the phenotype field
  `basal_metabolic_rate`, the Ocean `* 0.45` literal, the keys `basalMetabolicRateMultiplier` and `bmrBaseScale`. The
  organ load keeps the shape of the former organ tax.
- `CreatureAgent.adrenaline` and the always-1 surcharge factor were removed in P1 (TCK-149).
