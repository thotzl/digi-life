---
sources: [tickets/closed/TCK-201-r3-rooms-evaluator-breeder.md, src/trainer/evaluator.rs, src/trainer/sim.rs, src/trainer/breeder.rs, src/trainer/streams.rs, src/diagnostics/probes.rs]
---

# Contract: evaluator

## Purpose

An evaluator is an evaluation scheme of the trainer: how a breeding slot is scored in a trial and how its trials
combine into the generation fitness that selection ranks (TCK-201). `SoloTrials` (`src/trainer/evaluator.rs`) is
today's trainer: every slot alone in its chamber, all slots on the same shared layout, scored by the room
(`Room::score` of entity 0, [[wiki/contracts/room]]), the generation fitness the mean or median of the trial fitness.
It is used by `TrainerSim` (live score, trial end, generation end), `TrainerSim::evaluate_population` (final
evaluation of the runner) and the diagnosis probes (`hebbian`). Planned: a tournament that pairs slots into shared
chambers (arena). Selection, breeding and the 1/5 rule are not part of an evaluator; they live in `trainer::breeder`.
Guide: `wiki/guides/add-evaluator.md`.

## Signature

```rust
// src/trainer/evaluator.rs
pub struct SlotEval { pub trial_fitness: Vec<f32>, pub successes: u32, pub catches: u32 }
pub trait Evaluator: Send + Sync + Debug {          // object-safe, held as Arc<dyn Evaluator>
    fn id(&self) -> &str;
    fn score(&self, sb: &TrainerSandbox) -> f32;    // slot fitness in the running trial
    fn aggregate(&self, trial_fitness: &[f32], aggregation: TrialAggregation) -> f32;  // generation fitness
    fn evaluate(&self, room: &Arc<dyn Room>, sandboxes: &[TrainerSandbox], trials: u32,
                layout: &(dyn Fn(u32) -> SimRng + Sync), config: &AppConfig, parallel: bool) -> Vec<SlotEval>;
}
pub fn aggregate_trials(fitness: &[f32], aggregation: TrialAggregation) -> f32;  // mean | median
pub struct SoloTrials;
impl SoloTrials {   // the trial loop shared by evaluate and the probes
    pub fn run(&self, sb: &mut TrainerSandbox, trials: u32, fresh: bool, layout: impl Fn(u32) -> SimRng, config: &AppConfig) -> SlotEval;
}

// src/trainer/sim.rs
impl TrainerSim {
    pub fn evaluator(&self) -> &dyn Evaluator;
    pub fn set_evaluator(&mut self, evaluator: Arc<dyn Evaluator>);   // default Arc::new(SoloTrials)
    pub fn evaluate_population(&self, room: &Arc<dyn Room>, trials: Option<u32>, parallel: bool) -> PopulationEval;
}
```

## Lifecycle / call order

1. Every tick, before the sandboxes step (only with `track_live_fitness`, the UI): `sb.current_fitness = score(sb)`.
2. Trial end (all chambers `is_done`): `score(sb)` per slot, recorded per slot and trial.
3. Generation end, with `multi_trial`: `current_fitness = aggregate(trial_fitness, params.trial_aggregation)`; then
   `Breeder::select` ranks by it.
4. Batch (`evaluate_population`, probes): `evaluate` runs clones of the sandboxes for `trials` complete trials, trial
   `t` on `layout(t)` (`SoloTrials::run`: `reset_trial(layout(t))`, then step until `is_done`, then `score`); the
   caller aggregates.

## Invariants

- Pure and deterministic: no IO, no global RNG; the result never depends on `parallel` (results in sandbox order).
- `evaluate` never changes the given sandboxes (it works on clones).
- Learned synapse weights are kept across the trials of one slot (as in training); everything else of a trial is reset
  by `TrainerSandbox::reset_trial`.
- An evaluator that pairs slots draws its pairing from `derive(seed, [g, streams::PAIRING, trial])`
  (`src/trainer/streams.rs`), never from another stream.
- Changing `SoloTrials` or `aggregate_trials` is a model change: the trainer references change
  ([[wiki/architecture/invariants]] 8).

## Registration point

There is no registry: `TrainerSim::restore_at_layout` installs `SoloTrials`; a headless experiment installs another
scheme with `TrainerSim::set_evaluator`. A new scheme is one `impl Evaluator` in `src/trainer/evaluator.rs` (or its
own module under `src/trainer/`). Selecting it from the runner or the UI is a follow-up (a runner flag or a
`TrainerParams` field) once a second scheme exists.

## Config keys

None of its own. `SoloTrials` aggregates with `TrainerParams::trial_aggregation` (config
`trainer.trainerTrialAggregation`, runner `--aggregation`); the trial count is `trainer.trainerTrials` (`--trials`).

## Events / IPC

None. The generation fitness reaches the UI and the report through `GenerationStats`
([[wiki/simulation/analytics-and-balancing]]).

## Test recipe

`cargo test trainer::evaluator` (`test_solo_run_equals_a_hand_written_trial_loop`: `run` equals the explicit trial
loop, `evaluate` is parallel-safe) and `cargo test trainer::sim` (`test_live_telemetry_fitness_equals_epoch_end_fitness`,
`test_generation_fitness_is_the_trial_mean_without_gate`, `test_evaluate_population_is_deterministic_and_leaves_the_sim_unchanged`).
A new scheme adds a determinism test (`parallel` true == false) and a test against a hand-computed small case.

## Minimal example

```rust
// Best trial instead of mean/median: the same trials, a different generation fitness
#[derive(Debug)]
pub struct BestTrial;

impl Evaluator for BestTrial {
    fn id(&self) -> &str { "best-trial" }
    fn score(&self, sb: &TrainerSandbox) -> f32 { sb.score() }
    fn aggregate(&self, trial_fitness: &[f32], _aggregation: TrialAggregation) -> f32 {
        trial_fitness.iter().copied().fold(0.0, f32::max)
    }
    fn evaluate(&self, room: &Arc<dyn Room>, sandboxes: &[TrainerSandbox], trials: u32,
                layout: &(dyn Fn(u32) -> SimRng + Sync), config: &AppConfig, parallel: bool) -> Vec<SlotEval> {
        SoloTrials.evaluate(room, sandboxes, trials, layout, config, parallel)
    }
}
// sim.set_evaluator(Arc::new(BestTrial));
```
