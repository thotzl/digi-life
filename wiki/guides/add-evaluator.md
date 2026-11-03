---
sources: [wiki/contracts/evaluator.md, src/trainer/evaluator.rs, src/trainer/sim.rs]
---

# Guide: add an evaluator

An evaluator is a scoring scheme of the trainer: how a breeding slot is scored in a trial and how its trials combine
into the generation fitness (best trial, tournament pairing for an arena). Read first: [[wiki/contracts/evaluator]]
and [[wiki/architecture/invariants]].

## Steps
1. Add a struct implementing `Evaluator` (`id`, `score`, `aggregate`, `evaluate`) to `src/trainer/evaluator.rs`, or
   in its own module under `src/trainer/` (declared in `src/trainer/mod.rs`).
2. `evaluate` works on clones of the given sandboxes and returns one `SlotEval` per sandbox in sandbox order; reuse
   `SoloTrials::run` for the trial loop when the scheme only changes scoring or aggregation.
3. Randomness: a scheme that pairs slots draws from `derive(seed, [g, streams::PAIRING, trial])`; nothing else
   (invariant 16).
4. Registration: there is no registry; install it with `TrainerSim::set_evaluator(Arc::new(<Name>))`. A runner flag
   or a `TrainerParams` field to select it is a follow-up once it should be selectable.
5. Config keys: none unless the scheme has a tunable; then a key in section `trainer`
   (`wiki/guides/add-config-key.md`).
6. Tests next to the scheme: `parallel` true and false give identical results, the given sandboxes stay unchanged,
   and one hand-computed small case for `aggregate`.
7. Docs, same commit: name the scheme in the contract (Purpose) and the row in "Registered plugins" of `wiki/architecture/module-map.md`.

## Done when
- `cargo test trainer::evaluator` and `cargo test trainer::sim` pass, including the new tests.
- The project gate is green: `cargo fmt --check`, `cargo clippy --all-targets --features runner -- -D warnings`,
  `cargo test`, `npm test`, `npm run lint`, `npm run build`, `node scripts/check-links.mjs`, and the four CI
  reference runs match their committed reports (commands: `.agents/skills/plugin-authoring/SKILL.md`, Gate).
- Trainer references unchanged while `SoloTrials` stays the installed default (changing `SoloTrials` or
  `aggregate_trials` is a model change, invariant 8).
