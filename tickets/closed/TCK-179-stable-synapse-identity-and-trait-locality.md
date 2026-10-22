---
id: TCK-179
title: Stable Synapse Identity and Trait Locality (Heredity)
status: closed
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-172, TCK-173
---

# TCK-179: Stable Synapse Identity and Trait Locality (Heredity)

Part of epic TCK-171. Main suspect for training that does not accumulate improvements. Intended golden-master change.

## Description
The genome compiler has low locality: small genetic changes cause large phenotype changes, so children barely resemble their parents (code-verified 2026-10-21, details in TCK-172 finding 1):
- Exuberant synapse weights are read from `active_dna` at `(from + to*13) % len` (`src/biology/dna.rs:1323-1325`, `dna.rs:674-686`); an indel shifts every later position, a created/destroyed gene changes `len` and remaps all weights.
- Neuron ids depend on the organ count (`dna.rs:12-26`); one organ more/less shifts all ids, so weights and explicit `SY` synapses (modulo mapping, `dna.rs:1251-1258`) re-map.
- Body traits without their own gene fall back to `active_dna` (`dna.rs:800-861`), coupling brain mutations to body traits such as speed.

Literature: efficient evolutionary search needs high locality (Rothlauf); NEAT keeps a stable identity per connection via innovation numbers (sources in TCK-171).

## Scope
- Stable synapse key independent of absolute neuron indices and genome positions, e.g. `(source kind, organ index, channel, target kind, target index)`; the initial (exuberant) weight derives from that key plus the genes that belong to it, not from a position hash over `active_dna`.
- Neuron addressing by role (input organ/channel, motor, thalamus block, hidden index) so adding or removing an organ does not remap unrelated synapses.
- Body traits only from their own gene; a missing gene yields a fixed default, not an `active_dna` fallback.
- Explicit `SY` genes address roles instead of modulo over the current source/destination lists (relative addressing beyond this stays in TCK-168).
- Golden master: new fingerprint, documented in this ticket.
- Property tests (subset of TCK-161): `parse_genome` never panics on ASCII 0..1024 chars; `mutate_genome` keeps length in [128, 512]; locality test: a point mutation outside a gene changes no synapse weight; a point mutation inside a gene changes only that gene's synapses/traits.

## Acceptance
- TCK-172 heredity probe before/after: parent-child correlation of weights and traits after 1 / 5 / 15 mutations rises clearly (numbers in `## Verification`).
- Trainer report (TCK-173 runner): success rate over generations compared with the baseline.
- `wiki/biology/genome-compiler.md` and `wiki/biology/ctrnn-brain.md` describe the new addressing.

## Coordination
Same golden-master change window as TCK-144 (extra ΔI channel), optionally TCK-153 (evaluation order) and the biolum/reserved output decision of TCK-149, so the fingerprint changes once.

## Verification (P3, v1.12.17)
- Roles instead of absolute ids (`NeuronRole`); every synapse has a stable `key` (hash of both roles) and one `owner` gene (index into `phenotype.genes`); initial weights come from the owner's payload tape (slot = hash of the roles without the owner's own identity), no `active_dna` position hash.
- Gene identity `GeneRef{promoter, spelling, occurrence}` (exact promoter spelling, protected from mutation): gene duplication/deletion keep 100 % of unrelated synapse keys and weights (key-shift measurement, 1000 structured genomes; before: 77 % unchanged for shifting duplications, 73 % for shifting deletions).
- Trait locality: every body trait only from its own gene; missing gene = fixed default (h 0.5); `active_dna` removed.
- `SY` genes address roles (modulo within the role class).
- Property tests: random ASCII 0..1024 never panics; length bounds; point mutation outside genes changes nothing; inside gene G only G's synapses/traits/organs (1,578 mutants); appending, duplicating and deleting an organ/NEU gene keeps unrelated keys and weights; promoter spellings never change under letter mutations (20k).
- Heredity probe (exuberant-weight r after 1/5/15 trainer ops): before 0.943/0.661/0.318 (letter soup, old compiler) -> after 0.982/0.922/0.810 (structured parents, new compiler and operators); wildtype 15 ops 0.28 -> 0.78. Children with >= 2x speed change after 15 ops 17 % -> 7.5 % (slice A, old operators).
- Trainer (5 seeds, pop 16, 30 gens, mean success of the last 5 gens): small room 0.040 -> 0.122, large 0.023 -> 0.033.
- Golden master `0xa7b82dc4fe552ac8` -> `0x65f701a6937c1e71` (one intended Brain v2 change for TCK-179/182/144/153/184/149; ADR-002). CI reference `tests/reference/trainer-ci.json` regenerated. `cargo test` 94 passed (1 ignored measurement), 0 warnings; `npm run build` and `npm test` green.
