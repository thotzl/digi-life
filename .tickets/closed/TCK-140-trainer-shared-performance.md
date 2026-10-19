---
id: TCK-140
title: Trainer and Genome Compiler Performance (behaviour-preserving)
status: closed
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
revisit: RV-P06, RV-P03 (trainer), RV-P05 (trainer)
---

# TCK-140: Trainer and Genome Compiler Performance (behaviour-preserving)

## Description
- `mutate_genome` runs a full `parse_genome` (incl. brain synthesis) only to read the insertion/deletion rates.
- Brain reflex detection is a nested loop per candidate synapse.
- The scenario step clones the full agent every tick for spore push physics.
- Trainer telemetry sends each sandbox's full procedural world at 25 Hz although the frontend may not need it per tick (to verify).

## Scope
- Extract a rate reader; reflex lookup via a set; kinematics-only spore push input; drop the redundant telemetry payload only if the frontend does not use it.
- Equivalence tests (same genome gives an identical phenotype/brain before and after).

## Acceptance
- `cargo test` green, equivalence tests green.

## Verification
- Golden master `biology::dna::tests::test_parse_genome_golden_master`: fingerprint of `parse_genome` over 300 deterministic genomes (incl. organ/HOX/NEU/SY motifs) recorded before the refactor (`0xa7b82dc4fe552ac8`) and unchanged after it.
- RV-P06 `src/biology/dna.rs`: `normalize_genome`, `scan_chromatin`, `active_dna_of` extracted and shared by `parse_genome` and the new `read_evolution_rates`; `mutate_genome` no longer compiles a full phenotype (incl. brain) per mutation. Test `test_read_evolution_rates_matches_parse_genome` asserts identical rates for all golden genomes.
- Brain compiler: thalamus reflex pairs precomputed into a `HashSet` instead of a nested scan per candidate synapse (same rule; covered by the golden master).
- RV-P05 (trainer): `world` removed from `TrainerTelemetrySandbox` (sent per sandbox at 25 Hz, never read by the frontend; verified by grep). Binding `bindings/TrainerTelemetrySandbox.ts` regenerated.
- RV-P03 (trainer) was already done in TCK-139 (scenarios push spores via `SporePusher`).
- Finding while building the golden master: `latinName` differs between processes for genomes with tied organ-type counts (HashMap iteration order in `derive_ecological_metrics`). Excluded from the fingerprint; fixed separately in TCK-162.
- `cargo test`: 42 passed, 0 failed; `npm run build`: success.
