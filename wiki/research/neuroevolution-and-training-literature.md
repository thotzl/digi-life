---
sources: [tickets/ongoing/TCK-171-training-track-epic.md]
---

# Neuroevolution and Training Literature

Literature collected on 2026-10-21 for training quality, brain design and metabolic cost. Checked via search results and abstracts, not full texts. How the project applies it is planned in epic TCK-171; this page only holds the knowledge.

## Representation and heredity

- **Locality:** efficient evolutionary search needs a representation in which small genetic changes cause small phenotypic changes; low-locality mappings make mutation operators perform poorly (Rothlauf; Galván-López et al. for grammatical evolution). Current state of this project's mapping: [[wiki/biology/ctrnn-brain]] (weight derivation).
- **Stable gene identity:** NEAT marks every connection gene with a historical innovation number, so the same structure is recognised across generations; NEAT also starts minimal and grows structure ("complexification").
- **Mutation rate:** about one mutation per child is the theoretical standard; success-based adaptation (1/5-rule family) is provably optimal for the (1+1) EA (Doerr, Doerr, Lengler).
- **Body-brain co-optimisation:** body changes usually break the adapted controller; morphological innovation protection temporarily lowers selection pressure on recently body-mutated individuals (Cheney et al. 2018); a 2025 follow-up is critical of current co-optimisation methods.

## Evaluation and selection

- **Noisy evaluation:** re-randomised initial conditions per generation hide whether behaviour improved; holding conditions until all elites succeed is one remedy (DelFly behaviour-tree study). More evaluations are not always better; larger populations do not efficiently counter large evaluation noise (PSO robotics study). Survey: Jin & Branke 2005.
- **Selection schemes:** at equal selection intensity tournament selection loses the least diversity (Blickle & Thiele); Motoki shows the loss is larger than estimated there.

## Fitness design

- **Bootstrap problem:** if all individuals fail equally early on, evolution has no gradient; the most probable cause is a fitness without a gradual path; incremental evolution only if no smooth fitness exists; multi-objective selection with ancillary objectives also helps (Mouret & Doncieux; Nelson et al. 2009 survey).
- **Novelty search:** rewarding behavioural novelty beats objective search on deceptive mazes (Lehman & Stanley 2011) but struggles in large behaviour spaces with a fixed goal; weighted blends of fitness and novelty (Cuccu & Gomez 2011) or minimal criteria do better.
- **Connection cost:** selecting for performance and low connection cost yields more modular and more evolvable networks (Clune, Mouret, Lipson 2013).
- **Curricula:** POET admits new environments only if they are "neither too easy nor too hard" (minimal criterion); teacher-student curricula pick the task with the steepest learning progress.

## Sensing and steering

- **Tropotaxis:** two sensors with crossed or uncrossed wiring turn a vehicle towards or away from a source (Braitenberg vehicles).
- **Klinotaxis:** C. elegans steers with a single sensory neuron by head swings and temporal comparison of stimulus intensity (Satoh et al. 2014).

## Metabolic cost of brains and senses

- Brain energy use scales linearly with neuron number, roughly constant per neuron across mammals (Herculano-Houzel 2011).
- Most signalling energy goes into action potentials (47 %) and synaptic transmission (34 %) and scales with activity; sparse codes save energy (Attwell & Laughlin 2001).
- Vision costs 15 % of resting metabolism in a young (1 g) Astyanax fish and 5 % in an 8.5 g fish; eyeless cavefish save this energy in food-poor caves (Moran et al. 2015). Vertebrate brains take about 2–8 % of resting metabolism, the human brain about 20 %.
- Applied in P6 (TCK-151): additive cost with senses 7.5 % and brain 7.5 % of the median resting cost, the brain share split 20 % per neuron / 80 % activity ([[wiki/biology/metabolism-and-reproduction]]).

## Sources

- Rothlauf: https://gpbib.pmacs.upenn.edu/gp-html/Rothlauf_2011_DMH_cp7.html ; Galván-López et al.: https://ncra.ucd.ie/papers/localityingpWCCI2010.pdf
- NEAT: https://axon.cs.byu.edu/~dan/778/papers/NeuroEvolution/stanley2*.pdf ; innovation numbers: https://neat-python.readthedocs.io/en/latest/innovation_numbers.html
- Doerr, Doerr, Lengler: https://arxiv.org/pdf/1902.02588
- Cheney et al. 2018: https://meclab.w3.uvm.edu/papers/2018_RoySoc_Cheney.pdf ; follow-up 2025: https://arxiv.org/pdf/2508.17464
- DelFly behaviour trees: https://arxiv.org/pdf/1411.7267 ; morphological variation: https://arxiv.org/html/2208.02809v2 ; PSO fitness noise: https://dev.nccr-robotics.ch/?p=2621 ; Jin & Branke: https://www.cs.le.ac.uk/people/sy11/ECiDUE/JinBranke-TEVC05.pdf
- Blickle & Thiele: https://gpbib.pmacs.upenn.edu/gp-html/blickle_1995_css.html ; Motoki: https://niigata-u.repo.nii.ac.jp/record/1649/files/10_4_397-422.pdf
- Mouret & Doncieux: https://hal.archives-ouvertes.fr/hal-00473147 ; https://hal.archives-ouvertes.fr/hal-00687647 ; multi-objective ER: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4546428/ ; Nelson et al.: https://gpbib.pmacs.upenn.edu/gp-html/Nelson2009345.html
- Lehman & Stanley: https://gpbib.pmacs.upenn.edu/gp-html/Lehman_2011_EC.html ; novelty blends: https://www.cs.swarthmore.edu/~meeden/cs81/f17/projects/JeffKyle.pdf , https://arxiv.org/pdf/1803.03744
- Clune, Mouret, Lipson: https://arxiv.org/pdf/1207.2743.pdf
- POET: https://arxiv.org/pdf/1901.01753 ; Enhanced POET: https://arxiv.org/pdf/2003.08536
- Braitenberg: https://en.wikipedia.org/wiki/Braitenberg_vehicle ; https://frontiersin.org/articles/10.3389/fbioe.2020.565963/full
- Satoh et al.: https://pmc.ncbi.nlm.nih.gov/articles/PMC6608435 ; klinotaxis review: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4132367/
- Herculano-Houzel: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3046985/ ; Attwell & Laughlin: https://discovery-pp.ucl.ac.uk/1340392/1/356.full.pdf ; Moran et al.: https://pmc.ncbi.nlm.nih.gov/articles/PMC4643782

## Key Takeaways

- Locality and stable gene identity are preconditions for cumulative evolution.
- Early-phase failure is a fitness-design problem first (bootstrap), then a curriculum problem.
- Biological brain cost is additive and activity-driven; senses cost a bounded share of resting metabolism.
