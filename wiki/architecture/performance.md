---
sources: [docs/revisit/03-performance.md, .tickets/closed/TCK-139-ocean-hot-loop-performance.md, .tickets/closed/TCK-140-trainer-shared-performance.md]
---

# Performance

Context: ~60 Hz loop with up to 35 physics steps per frame (≈ 2,100 steps/s). Every operation in the step body multiplies accordingly. Population is small (25–45) only because offspring do not live (RV-B01); after TCK-143 the remaining items become pressing. Measure a baseline (ticks/s at warp 35, population 45) before optimizing.

## Findings and status

| ID | Prio | Finding | Status |
| :--- | :--- | :--- | :--- |
| RV-P01 | P1 | Full `species_records` scan per step + O(S × C) counting | done (TCK-139): in-memory cache, dirty flag, one `HashMap` pass |
| RV-P02 | P1 | `SpatialGrid::new(19200, 10800, 80)` (32,400 cells) per step | done (TCK-139): allocate once, `clear()` |
| RV-P03 | P1 | Deep agent clones per spore, `creatures.clone()` per step, `vec![sb.agent.clone()]` per sandbox tick | done (TCK-139/140): `SporePusher`, `PeerSnapshot`, index lookups |
| RV-P04 | P2 | Nearest-food search linear over all spores, O(C × F + C²) | pool TCK-156; deliberately deferred: ring search may change tie-breaking between equidistant spores |
| RV-P05 | P1 | Events emitted inside the step loop; trainer telemetry sent full world per sandbox at 25 Hz | done: buffer + flush per frame, coalesced `DATABASE_CHANGED` (TCK-139); world removed from trainer telemetry (TCK-140) |
| RV-P06 | P2 | `mutate_genome` compiled the full phenotype to read two rates; nested reflex check | done (TCK-140): `read_evolution_rates`, reflex pairs in a `HashSet` |
| RV-P07 | P2 | Catalogue re-queried per missing creature in restock | done (TCK-139) |
| RV-P08 | P3 | Per-call allocations in `execute_brain_with_learning` / `compute_sensory_inputs` | open; only with profiler evidence |

## Equivalence safeguards

- Golden master `test_parse_genome_golden_master` (fingerprint `0xa7b82dc4fe552ac8`) unchanged across TCK-140.
- `test_spore_push_from_snapshot_matches_full_agent` (TCK-139).
- Engine changes preserve iteration order and data; the engine itself is not unit-testable (RV-A01).

## Key Takeaways

- All P1 hot-loop costs are fixed; nearest-food search and brain allocations remain.
- Optimizations are behaviour-preserving and guarded by the golden master.
- No micro-optimization without profiler evidence ([[roadmap]] non-goals).

Sources: `docs/revisit/03-performance.md`, TCK-139, TCK-140.
