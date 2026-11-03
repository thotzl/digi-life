---
sources: [wiki/biology/genome-compiler.md, wiki/biology/sensing.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-154-trainer-scenario-parameters.md]
---

# Genome probe measurements (history)

Heredity and equipment measurements of the training epic (TCK-171 P3, P7), moved out of
[[wiki/biology/genome-compiler]] and [[wiki/biology/sensing]] in TCK-203. Numbers come from the code of their phase
and are not reproducible with later code; re-run `trainer_runner probe heredity|equipment` for current values.

## Heredity (TCK-172 probe, P3)

Probe `trainer_runner probe heredity` (seed `tck172`, 200 parents per set): parent–child Pearson correlation of non-`SY`, non-reflex synapse weights on homologous synapses (same key).

| Mutation | 1 op | 5 ops | 15 ops |
| :--- | :--- | :--- | :--- |
| Before P3: letter-soup parents, character mix | 0.943 | 0.661 | 0.318 |
| Structured parents, `mutate_genome` mix | 0.982 | 0.922 | 0.810 |
| Letter-soup parents, `mutate_genome` mix | 0.979 | 0.906 | 0.757 |
| Structured parents, legacy character mix | 0.985 | 0.916 | 0.792 |

- Children with ≥ 2× terminal-speed change (structured parents): 0.7 % after 1 op, 3.0 % after 5, 8.1 % after 15 (before P3: 1.4 % / 7.0 % / 16.9 %). Measured with the pre-P5 speed model; the P5 speed band (1–5 px/tick, TCK-150) caps speed ratios at ×5, not re-measured.
- Gene duplication and deletion keep 100 % of the keys and weights of synapses that neither belong to nor touch the mutated gene (except `SY`-owned synapses and `HOX` genes); test `test_gene_duplication_and_deletion_keep_unrelated_synapses`, measurement `measure_gene_operator_key_shift` (ignored test).
- Most of the gain over the character mix comes from Brain v2 locality, not from the operator mix (legacy and new mix differ by ≤ 0.02 on structured parents).

## Equipment of random genomes (TCK-172 probe)

1,000 genomes per set; structured = `generate_structured_genome` (trainer generation 1 and immigrants since TCK-184), letter soup = the former random 256-base genomes.

| Property | Letter soup | Structured |
| :--- | :--- | :--- |
| No organ | 21 % | 0 % |
| ≥ 1 eye / ≥ 1 nose | 29 % / 43 % | 58 % / 66 % |
| ≥ 1 food-sensing organ (distance sense) | 67 % | 99 % |
| `HOX` organ pair | 21 % | 31 % |
| Food in range / non-zero stimulus at spawn, small room | 57 % / 31 % | 86 % / 46 % |
| Food in range / non-zero stimulus at spawn, large room (650 px, before P7) | 14 % / 4.8 % | 24 % / 7.5 % |

- The large-room rows were measured at the pre-P7 spawn distance of 650 px. With 500 px since P7 (TCK-154 sweep, 2000 structured genomes) food is in sensor range at spawn for 35 % (13 % also in the view cone); small room 200 px: 89 % ([[wiki/simulation/trainer]]).
- In the large room most genomes still receive no food signal at the start of a trial.
