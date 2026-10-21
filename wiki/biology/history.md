---
sources: [.tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, .tickets/closed/TCK-116-sequence-based-dynamic-genome-regulatory-network.md, .tickets/closed/TCK-117-biochemical-codon-groups-wobble-matching.md, .tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, .tickets/closed/TCK-121-synaptic-exuberance-and-active-lifetime-pruning.md, .tickets/closed/TCK-122-multispectral-receptive-fields-and-haptics.md, .tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, .tickets/closed/TCK-133-biophysical-sensing-upgrades-and-sensor-fusion.md, .tickets/closed/TCK-162-deterministic-latin-name.md]
---

# Biology History (implemented)

| Ticket | Date | What was implemented | Article |
| :--- | :--- | :--- | :--- |
| TCK-106 | 2026-08-19 | Evolvable hidden activations (tanh/ReLU/sigmoid/sin); prey mocking ("meatballs") for carnivore training | [[biology/ctrnn-brain]] |
| TCK-116 | 2026-08-22 | Sequence-based genome: promoter/terminator scanning, payload hashing, one neutral extraction helper; no positional loci. The planned 2-stage GRN cascade (transcription factors binding operators) is described in the ticket; current code has no such cascade | [[biology/genome-compiler]] |
| TCK-117 | 2026-08-31 | 5 biochemical codon groups, degenerate (wobble) matching, insertion/deletion mutations, length `[128, 512]` | [[biology/genome-compiler]] |
| TCK-119 | 2026-08-31 | `NEU`/`SY` genes build hidden layers and synapses; Hebbian learning with forgetting, clamp ±4; Lamarckian offset inheritance | [[biology/ctrnn-brain]], [[biology/metabolism-and-reproduction]] |
| TCK-121 | 2026-08-31 | Synaptic exuberance at birth, pruning below 0.015 | [[biology/ctrnn-brain]] |
| TCK-122 | 2026-08-31 | 5-channel multispectral receptive fields, tactile haptic channels, object emission spectra | [[biology/sensing]] |
| FIX-131 | 2026-09-03 | HOX bilateral symmetry with midline snapping; smoother organ visuals; reef collision and haptic reef sensing | [[biology/genome-compiler]], [[simulation/physics]] |
| TCK-133 | 2026-09-04 | 0° = ahead angle space; fixed body-state inputs; HOX sum/difference interneurons; rheotaxis via lateral-line pores; motion vision ΔI | [[biology/sensing]], [[biology/ctrnn-brain]] |
| TCK-162 | 2026-10-19 | Deterministic Latin name (fixed tie-break order), test `test_latin_name_is_deterministic` (RV-B11) | [[biology/genome-compiler]] |

## Key Takeaways

- The genome model changed fundamentally in TCK-116/117 (positional → promoter-based); anything mentioning "locus N" is pre-TCK-116.
- Brain and sensing were built up in TCK-119–133; motion vision introduced the channel-5 defect RV-B06.

Sources: closed tickets listed in the frontmatter.
