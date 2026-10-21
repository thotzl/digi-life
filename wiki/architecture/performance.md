---
sources: [docs/revisit/03-performance.md, tickets/closed/TCK-139-ocean-hot-loop-performance.md, tickets/closed/TCK-140-trainer-shared-performance.md]
---

# Performance

Context: ~60 Hz loop with up to 35 physics steps per frame (≈ 2,100 steps/s); every operation in the step body multiplies accordingly. The Ocean population is 25–45 because offspring are not simulated (RV-B01); higher populations will stress the remaining linear searches.

## Implemented optimizations (behaviour-preserving)

| Area | Before | Now | Ref |
| :--- | :--- | :--- | :--- |
| Species tracking | full `species_records` SQL scan per step + O(S × C) counting | in-memory cache, re-read only after insert/reset, one `HashMap` pass | RV-P01, TCK-139 |
| Spatial grid | `SpatialGrid::new(19200, 10800, 80)` (32,400 cells) per step | allocated once, `clear()` per step, buckets hold indices | RV-P02, TCK-139 |
| Agent clones | deep clones per spore, `creatures.clone()` per step, `vec![sb.agent.clone()]` per sandbox tick | `SporePusher` and `PeerSnapshot` snapshots, index lookups | RV-P03, TCK-139/140 |
| IPC | events emitted inside the step loop; full world per trainer sandbox at 25 Hz | buffered, flushed once per frame, `DATABASE_CHANGED` coalesced; no world in trainer telemetry | RV-P05, TCK-139/140 |
| Mutation | `mutate_genome` compiled the full phenotype to read two rates; nested reflex check | `read_evolution_rates`; reflex pairs in a `HashSet` | RV-P06, TCK-140 |
| Restock | catalogue re-queried per missing creature | loaded once per restock pass | RV-P07, TCK-139 |

## Remaining costs

- Nearest-food search scans all spores twice plus all creatures, O(C × F + C²); the grid is not used for it (RV-P04). A grid ring search could change tie-breaking between equidistant spores.
- `execute_brain_with_learning` and `compute_sensory_inputs` allocate per call (RV-P08); irrelevant below ~50 agents.
- Brain step per creature and tick: one pass over all synapses for signals, one for Hebbian learning (`src/shared/brain.rs:80-133`); pruned synapses (weight 0) are still visited in the signal pass; organ type is detected by an emoji `contains` per organ (`brain.rs:56`) and the activation by a `String` match per neuron (`brain.rs:98-104`). Synapse counts and dead wiring see [[wiki/biology/ctrnn-brain]].
- Trainer sandboxes are stepped sequentially in the engine thread.

## Equivalence safeguards

- Golden master `test_parse_genome_golden_master` (fingerprint `0xa7b82dc4fe552ac8`) unchanged across TCK-140.
- `test_spore_push_from_snapshot_matches_full_agent` (TCK-139).
- Engine changes preserve iteration order and data.
- Optimizations need profiler evidence ([[wiki/decisions/adr-001-non-goals]]).

## Key Takeaways

- Per-step SQL, allocations, clones and per-step IPC are gone.
- Remaining hot spots: linear nearest-food search and per-call brain allocations.

Sources: `docs/revisit/03-performance.md`, TCK-139, TCK-140.
