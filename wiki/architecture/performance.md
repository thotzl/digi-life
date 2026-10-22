---
sources: [docs/revisit/03-performance.md, tickets/closed/TCK-139-ocean-hot-loop-performance.md, tickets/closed/TCK-140-trainer-shared-performance.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-182-brain-topology-compaction.md]
---

# Performance

Context: the UI runs a ~60 Hz loop with up to 35 physics steps per frame (≈ 2,100 steps/s); every operation in the step body multiplies accordingly. The headless trainer runner has no warp cap ([[wiki/simulation/analytics-and-balancing]]). The Ocean population is 25–45 because offspring are not simulated (RV-B01); higher populations will stress the remaining linear searches.

## Implemented optimizations (behaviour-preserving)

| Area | Before | Now | Ref |
| :--- | :--- | :--- | :--- |
| Species tracking | full `species_records` SQL scan per step + O(S × C) counting | in-memory cache, re-read only after insert/reset, one `HashMap` pass | RV-P01, TCK-139 |
| Spatial grid | `SpatialGrid::new(19200, 10800, 80)` (32,400 cells) per step | allocated once, `clear()` per step, buckets hold indices | RV-P02, TCK-139 |
| Agent clones | deep clones per spore, `creatures.clone()` per step, `vec![sb.agent.clone()]` per sandbox tick | `SporePusher` and `PeerSnapshot` snapshots, index lookups | RV-P03, TCK-139/140 |
| IPC | events emitted inside the step loop; full world per trainer sandbox at 25 Hz | buffered, flushed once per frame, `DATABASE_CHANGED` coalesced; no world in trainer telemetry | RV-P05, TCK-139/140 |
| Mutation | `mutate_genome` compiled the full phenotype to read two rates; nested reflex check | `read_evolution_rates`; reflex pairs in a `HashSet` | RV-P06, TCK-140 |
| Restock | catalogue re-queried per missing creature | loaded once per restock pass | RV-P07, TCK-139 |
| Brain step | emoji `contains` per organ and `String` match per neuron every tick; pruned synapses visited | `BrainRuntime` cache per agent: activation enum, integration factor, ΔI organs from roles, active-synapse list without pruned weights ([[wiki/biology/ctrnn-brain]]) | TCK-173, TCK-144 |
| Brain size | fully wired: synapses p50 142 (random genomes), incl. dead motors and silent thalamus nodes | targeted wiring of live structure: p50 41 (letter soup) / 78 (structured); trainer throughput at pop 16 about ×2 (model change, not behaviour-preserving) | TCK-182 |
| Trainer buffers | sensor inputs, brain outputs and large-room visible foods allocated per tick | per-sandbox reused buffers (`compute_sensory_inputs_into`, visible-food scratch), outputs as a fixed array | RV-P08 (trainer path), TCK-173 |
| Trainer fitness | `calculate_fitness` every tick for every sandbox | only for the UI telemetry (`track_live_fitness`); the runner computes it at epoch end | TCK-173 |
| Trainer parallelism | sandboxes stepped sequentially | sandboxes and breeding slots on the rayon pool, one RNG stream each (results identical to sequential); used by the UI (`step(true)`) and the runner (`--parallel true`) | TCK-173 |

## Remaining costs

- Nearest-food search scans all spores twice plus all creatures, O(C × F + C²); the grid is not used for it (RV-P04). A grid ring search could change tie-breaking between equidistant spores.
- The Ocean still calls the allocating `compute_sensory_inputs` per creature and tick (RV-P08); irrelevant below ~50 agents.
- Brain step per creature and tick: one pass over the active synapses for signals, one for Hebbian learning. Synapse counts see [[wiki/biology/ctrnn-brain]].
- Trainer generation end (final fitness, stats, sort, breeding incl. `parse_genome` for every new sandbox) dominates the headless wall time: standard pop 300 spends 22.9 of 26.2 s in generation-end ticks over 20 generations (P2 baseline).

## Trainer throughput (TCK-173)

Measured with the headless runner (release build), `perf` block of the report; machine-dependent.
- Per-tick step after the brain/buffer changes: +20–30 % sandbox-ticks/s.
- Parallel breeding: generation end at standard pop 300 2.79 s → 0.67 s (2-generation benchmark).
- P2 baseline totals: standard pop 300 parallel ≈ 206k sandbox-ticks/s (20 generations incl. breeding), exploration pop 16 parallel ≈ 469k, exploration pop 300 ≈ 717k (10 generations).
- Brain v2 (TCK-182), pop 16, seed b1, same machine: standard 158k → 323k, exploration 425k → 763k sandbox-ticks/s.

## Equivalence safeguards

- Golden master `test_parse_genome_golden_master` unchanged across TCK-140 and TCK-173 (`0xa7b82dc4fe552ac8`); the Brain v2 model change (TCK-171 P3) set it to `0x65f701a6937c1e71` on purpose ([[wiki/quality/testing]]).
- Trainer report bit-identical before/after the TCK-173 performance work; parallel == sequential tests ([[wiki/quality/testing]]); CI reference run ([[wiki/quality/ci-and-tooling]]).
- `test_spore_push_from_snapshot_matches_full_agent` (TCK-139).
- Engine changes preserve iteration order and data.
- Optimizations need profiler evidence ([[wiki/decisions/adr-001-non-goals]]).

## Key Takeaways

- Per-step SQL, allocations, clones and per-step IPC are gone; the brain step does no string matching and skips pruned synapses.
- Trainer sandboxes and breeding run in parallel, deterministically.
- Remaining hot spots: linear nearest-food search, Ocean sensor allocation, trainer generation end.

Sources: `docs/revisit/03-performance.md`, TCK-139, TCK-140, TCK-173, TCK-182.
