---
sources: [tickets/closed/TCK-197-r0-safety-net.md, tickets/closed/TCK-156-engine-decomposition.md, tickets/closed/TCK-158-seedable-rng-headless-runner.md, tickets/closed/TCK-202-r5-model-corrections.md, tickets/closed/TCK-161-test-suite-csp-lint.md, src/shared/rng.rs, src/shared/fmath.rs, tests/reference/compare-trainer-report.mjs]
---

# ADR-004: Deterministic Simulations, Platform-Independent Math, Reference Runs

- Status: accepted
- Date: 2026-11-03
- Tickets: TCK-197, TCK-156, TCK-158, TCK-202, TCK-161

## Context

The trainer was already seeded (TCK-173) and checked by one CI reference run, but the Ocean used `thread_rng` in eleven places and could only be judged statistically. Parts of the simulation used std `sin`/`cos`, whose results may differ between platforms, while CI runs on Linux and the app ships on Windows. The golden master hashed the full serde output of the phenotype, so moving a display label changed it. A structure-only rebuild needs a way to prove "behaviour unchanged".

## Decision

1. **Every simulation is IO-free and seeded.** Trainer and Ocean (`OceanSim`) draw only from `SimRng` streams derived from one run seed with named stream constants (`derive(seed, [STREAM, ...])`); parallel and sequential runs give identical results. The Ocean stores seed, tick and release count in its snapshot, so a restored Ocean continues identically.
2. **Platform-independent math.** All simulation code (`src/sim`, `src/biology`, `src/trainer`, `src/shared`, engine) uses `fmath` instead of std trigonometry and transcendentals; std math is allowed only in tests and diagnostics output.
3. **Golden master over model fields only.** `GOLDEN_FINGERPRINT` hashes the model fields of compiled phenotypes (traits, organs, neurons, synapse keys/weights/owners), not display strings or dead fields.
4. **Reference runs in CI.** Four committed reports are reproduced on every push and compared value by value: trainer standard, trainer curriculum with final evaluation, trainer curriculum with rehearsal, and a headless Ocean run (`trainer_runner ocean`). The comparator ignores only `perf` and `meta.version`.
5. **A changed fingerprint or reference is either a bug or an intended model change.** Intended changes are bundled, stated in the ticket with old and new values, and come with a `MODEL_VERSION` bump when stored data becomes invalid (invariants 8, 9).

## Consequences

- Structure-only work is gated mechanically: unchanged golden master and four matching references (used for every slice of TCK-196).
- Ocean behaviour can be tested in unit tests and measured headless (population, births, deaths by cause, species over time).
- New randomness needs a named stream; a new world setting or a math change shows up as a reference diff and must be justified.
- The Ocean reference changed twice by design (TCK-202 model corrections, TCK-143 simulated offspring), each with the reason recorded.
