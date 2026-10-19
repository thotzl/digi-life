---
id: TCK-162
title: Deterministic Latin Name for Tied Organ Types
status: closed
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
revisit: found during TCK-140
---

# TCK-162: Deterministic Latin Name for Tied Organ Types

## Description
`derive_ecological_metrics` (`src/biology/dna.rs`) counts organ types in a `HashMap` and sorts by count. On ties, the result depends on the per-instance random `HashMap` iteration order, so the same genome can get a different latin name suffix on every parse (e.g. across app starts, or between the Ocean and the catalogue preview). The project documents `parse_genome` as deterministic.

## Scope
- Reproduction test: parsing the same genomes repeatedly yields identical latin names.
- Fixed tie-break order (light, chem, kinetic, thermal — the order of the suffix match arms).

## Acceptance
- Test green; golden master unchanged (it excludes the latin name).

## Verification
- Reproduction test `biology::dna::tests::test_latin_name_is_deterministic` failed before the fix (`"Biplasma bilateralis ciliatum (Str. GT-4)"` vs `"... thermum ..."` for the same genome).
- Fix (`src/biology/dna.rs`, `derive_ecological_metrics`): organ-type counts in a fixed-order array; ties resolve to the first type in the order light, chem, kinetic, thermal.
- Golden master (`test_parse_genome_golden_master`) unchanged.
- `cargo test`: 43 passed, 0 failed.
- Effect: species with tied organ-type counts get one stable name. Names already stored in the DB for such species may differ from the now-stable name.
