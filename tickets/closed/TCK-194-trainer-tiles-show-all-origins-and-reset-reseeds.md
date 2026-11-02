---
id: TCK-194
title: Trainer Tiles Show Every Origin Group, Reset Draws a New Run Seed
status: closed
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: Training Track (post-fix of TCK-192)
depends_on: TCK-192, TCK-191
---

# TCK-194: Trainer Tiles Show Every Origin Group, Reset Draws a New Run Seed

User test of the trainer room after v1.12.25 (2026-11-02).

## Description
1. **Tiles show only elite parents.** Breeding still fills every generation with elites 25 % | hall of fame 15 % | random immigrants 15 % | mutants 45 %, in this slot order (`TrainerSim::breed`, `src/biology/trainer_sim.rs`). Since TCK-192 the population (default 100) is independent of the visible tiles (default 16), and `TrainerTelemetryTick::build` (`src/server/trainer_ipc.rs`) sends the first N slots, i.e. only elites (slots 0..24 at pop 100). From generation 2 on every tile reads "Elite Parent"; before TCK-192 (pop = tiles = 16) the grid showed 4 / 2 / 2 / 8. Display bug only, selection and breeding are unaffected.
2. **Reset replays the same run.** `TRAINER_RESET` (`src/server/engine.rs`) builds `TrainerSim::new` with `seed_from_str("{config.seed}:{run_id}")`; the run id does not change, so every reset yields the identical random generation 1 and the identical run afterwards. Determinism is wanted for the runner, tests and CI reference, not for the reset button.

## Scope
- Telemetry sends a representative sample of the population: per origin group `round(N · group / population)` slots (at least one per present group, the rest to the group with the most spare slots), first slots of each group, in slot order. The simulation never depends on it.
- Frontend tiles are keyed by tile index and show the real sandbox id; selection and brain view follow the tile's current sandbox.
- A reset draws a new run seed (`run_seed(config.seed, run_id, nonce)`, nonce from the clock) and stores it with the run state (`trainer_run_state.seed`); restore and app restart continue with the stored seed. Runs without a stored seed keep `seed_from_str("{config.seed}:{run_id}")`. Runner, tests and CI reference unchanged.

## Acceptance
- Pop 100, 16 tiles: tiles show 4 elite / 2 hall of fame / 2 random / 8 mutants with their real ids; selection, brain view and assignment act on the shown sandbox.
- Two resets give different generation-1 genomes; restart / run switch after a reset continue the same run.
- `cargo test`, `npm test`, `npm run build` green; golden master and `tests/reference/trainer-ci.json` unchanged.

## Verification (2026-11-02, v1.12.26)
- **Reproduction first:** `test_telemetry_samples_every_origin_group` (pop 100 restored at generation 2, 16 tiles) failed before the fix with `left: [16, 0, 0, 0] right: [4, 2, 2, 8]` (elite / hof / random / mutant). The reset replay sat in the engine closure (`TrainerSim::new(.., trainer_seed(&trainer_run_id), ..)`, not testable without a window); covered by the pure `run_seed` test and the UI check below.
- **Tiles:** `display_slots` in `src/server/trainer_ipc.rs` (per group `round(N · group / population)`, at least one per present group, rest to the group with the most spare slots, first slots per group, slot order); `TrainerTelemetryTick::build` sends those slots. `frontend/trainer.ts`: tile DOM keyed by tile index (`sandbox-card-<idx>`, `sb-id-lbl-<idx>`, ...), tile `i` takes telemetry entry `i` and its real id, click selects that id, `selectSandbox` re-sends `SELECT_TRAINER_SANDBOX` when the selected tile's id changes.
- **Reset seed:** `run_seed(config_seed, run_id, nonce)` and `TrainerSim::seed()` in `src/biology/trainer_sim.rs`; `TrainerRunState.seed`; `trainer_run_state.seed` (TEXT, `ALTER TABLE` in `init_db`); engine reset draws a clock nonce and saves the run state at once, `restore_trainer` uses the stored seed on load (else the seed without nonce) and the current trainer's seed otherwise.
- **Tests:** `cargo test` 181 passed, 1 ignored (the existing manual key-shift measurement). New: `test_telemetry_samples_every_origin_group`, `test_display_slots_edge_cases` (generation 1 all random, n ≥ population, n = 2 → slots [0, 25], every n 1..=100 gives n distinct sorted slots), `test_run_seed_nonce_draws_a_new_generation_one`; extended: `test_telemetry_carries_only_the_displayed_sandboxes` (pop 12, N 5 → ids [1, 4, 6, 8, 9], one per group plus a second mutant), run-state round trip with a seed beyond `i64::MAX`, v1.12.23 table migration gains `seed` (None), run-state apply test. `npm run build` and `npm test` (10 tests) green. CI trainer reference (`ci-reference`, 3 gens, pop 16) matches; golden master unchanged; binding drift only the doc comment of `TrainerTelemetryTick`.
- **UI check** (`tauri dev`, app DB backed up and restored, `.agents/artifacts/uicheck/out-tck194`): after a reset 16 tiles "Fresh Random" #1–#16; generation 2 at pop 100: 4 elite / 2 hall of fame / 2 random / 8 mutants, ids 1–4, 26–27, 41–42, 56–63; clicking tile 16 shows "Sandbox #63" in the inspector; two resets show different creatures in tile 0; 0 console errors. Standard smoke checklist (`out-tck194-smoke`) OK: Ocean, generation, metrics, curriculum, room switch, pause (0 completions), reset to generation 1, RTT 9 ms at max warp.
