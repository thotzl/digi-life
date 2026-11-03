---
sources: [wiki/biology/ctrnn-brain.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md]
---

# Brain measurements (history)

Measurements of the brain model, moved out of [[wiki/biology/ctrnn-brain]] in TCK-203. Numbers come from the code of
their phase (TCK-171 P3–P5, model versions before 7) and are not reproducible with later code; re-run the probes
(`trainer_runner probe hebbian|steering`) for current values.

## Brain size (TCK-172 topology probe)

- **Size** (TCK-172 topology probe, 1,000 genomes): synapses p10/p50/p90 52/142/249 before P3 → 14/41/103 on letter-soup genomes and 48/78/148 on structured genomes (mean 150 → 52 / 88); wildtype 249 → 118 synapses, 72 → 67 neurons.

## Learning (P4, TCK-152)

Measured with `probe hebbian` (P4, `probe hebbian`, learning on, best 14–15 elites of a seeded trainer run)

| | Before P4 (unnormalized Hebbian + forgetting) | Oja + fixed reflexes |
| :--- | :--- | :--- |
| Long life 36,000 ticks: mean \|w\| (plastic) | 1.318 | 0.342 |
| Long life: share at the clamp (\|w\| ≥ 3.9) | 10.3 % | 0.22 % |
| Long life: share pruned | 39.7 % | 51.4 % |
| Long life: reflex drift (mean \|learned − genome\|) | 0.097 | 0 |
| 900-tick trial (standard room): mean \|w\| / pruned | 0.270 / 5.3 % | 0.258 / 8.7 % |

- **Benefit of lifetime learning:** not measurable at the current success level. Paired fitness delta learning on − off (median): standard −1.4, exploration −8.4; the means are dominated by single genomes. Success is 0–2.4 % in all conditions.
- Rejected variants (5-seed runs, see [[wiki/simulation/trainer]]): no pruning (lower success), pruning only with active presynaptic input (no measurable change).

## Steering at random initialisation (TCK-172 steering probe)

Food fixed at 300 px and ±45°, real brain with learning, heading change after 100 ticks; 500 genomes per set. "Bearing reduced" = the minimum bearing to the food fell below 35° for both food sides.

| | Before P3 (letter soup) | P3–P4, letter soup | P3–P4, structured | After P5, letter soup | After P5, structured |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Turns toward / away from food | 3.2 % / 5.0 % | 3.8 % / 4.4 % | 9.0 % / 8.2 % | 7.0 % / 6.2 % | 16.0 % / 14.6 % |
| Toward food on both sides | 0.4 % | 0.6 % | 0.2 % | 0.4 % | 1.0 % |
| Bearing reduced on both sides | – | 23.6 % | 27.4 % | 56.4 % | 64.0 % |
| Mean thrust negative | 48 % | 52 % | 49 % | 52 % | 49 % |

- Steering is still undirected at random initialisation (toward ≈ away); the ΔI channel and latency 1 make directed turning learnable but not innate. The P5 rise in turning and bearing reduction comes from the mechanics (turning at rest and faster turning of slow bodies, TCK-150, [[wiki/simulation/physics]]); the brain model is unchanged in P5.
- Heredity of weights under mutation: [[wiki/history/probe-measurements]].
