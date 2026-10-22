---
sources: [.agents/skills/biology-dna/SKILL.md, tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, tickets/closed/TCK-129-architecture-refactoring.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-182-brain-topology-compaction.md, tickets/closed/TCK-152-hebbian-plasticity-model.md]
---

# Metabolism, Reproduction and Species

## Metabolism

- BMR (`parse_genome`): `(body area term + organ tax) · (1 + organs · 5 · 0.08) · 1.4 · stiffness · (1 + synapses · 0.06)`. The Ocean compensates with a `* 0.45` factor (`engine.rs`).
- Brain v2 (TCK-182) changed only the synapse count: an organ gene now adds one sum thalamus node per live channel (up to 6 for distance senses, 2 for tactile kinds; plus as many difference nodes for a `HOX` pair), each wired to 2 motors and to every hidden neuron, instead of ~10 fully wired thalamus sources. Median synapses 142 → 41 (letter soup), 78 for structured genomes, so the synapse term is lower; the organ term still counts 5 channels per organ (ΔI not counted). The multiplicative brain cost still penalizes organs (RV-S03).
- `CreatureAgent.adrenaline` is removed (TCK-149); the adrenaline keys and the always-1 BMR surcharge were removed in P1.

## Start energy (TCK-185)

- Every spawned creature starts with `stomach_capacity · creatureStartStomachRatio` (0.60), defined once in `creature_factory::start_energy` and used by all Ocean spawn sites and the trainer (also on every trainer trial reset).
- Intended changes against the previous per-site values: trainer 100 → stomach × 0.60 (the brain input `internal_energy = energy / 100` changes with it, [[wiki/biology/sensing]]); `SPAWN_CLONE` and catalogue assignment to a trainer sandbox 0.8 → 0.60. `antisense` and neuron buffers are always set from the phenotype.
- Ocean mitosis children also go through the factory (`spawn_from_phenotype`), but take the parent's post-split energy instead of the start energy (below, TCK-152).

## Food and predation

- Herbivores eat plant spores, carnivores eat meat; dead creatures decompose into meat spores (`type_id = 2`), eaten meat recycles into algae (TCK-129).
- Biting gives the attacker no energy (`bitingBaseEnergyGain` unread, exempt in the config read-site test, TCK-145); the victim loses 35. There is no bite cooldown: the same victim can be bitten every tick, with several bite events per tick (RV-B07).
- The rule "do not bite the same species" only applies to exact clones (species = genome, see below).

## Reproduction (Ocean mitosis)

- Condition: `age >= max(phenotype.mature_age, creatureMatureAgeFloor) && has_eaten && population < maxCreatures && energy >= stomach_capacity * max(phenotype.repro_threshold, reproductionStomachThresholdFloor)`; parent energy multiplied by `reproductionSplitLossRatio` (TCK-149).
- Floors apply at the read site in `engine.rs`; `parse_genome` is unchanged. Mature age floor 600 raises phenotype values 300–600 to 600; threshold floor 0.60 equals the phenotype minimum (range 0.60–0.95), so it has no effect with current values.
- `population` = `creatures.len()` at step start, cap `maxCreatures` (45). Since children are not simulated (below), the cap has no practical effect.
- Child genome mutated with 35 % probability (`thread_rng`); clones (65 %) keep the parent genome and phenotype. Clone and mutant are both built by `creature_factory::spawn_from_phenotype` (TCK-152): parent position and heading, generation + 1, fresh neuron states and `BrainRuntime`, velocity 0, energy = parent energy after the split loss, `species_id` = child genome. Learned weights come only through Lamarckian inheritance (below); neither clones nor mutants copy the parent agent.
- **Children are not simulated:** they are pushed only to `newly_spawned_creatures`, never into the simulated list; the client adds and removes them again. The Ocean has no heredity; population comes only from restocking. `highestGeneration` and species rows still grow (RV-B01).

## Lifespan

- `max_age_ticks = min(mature_age * creatureLifespanMatureAgeMultiplier (12), creatureMaxAgeTicks (36000))` (`engine.rs`, TCK-149), DNA-dependent (TCK-129) → 3,600–32,400 ticks; the cap lies above the maximum and does not bind with current values.
- Lifespan and maturity are still split between phenotype and config; no single source of truth (RV-B08).

## Lamarckian inheritance

- `creature_factory::inherit_learned_weights(parent_synapses, parent_weights, child_synapses, chance, rng)` (TCK-179, TCK-152): for each **plastic** child synapse whose **synapse key** matches a plastic parent synapse, one draw from `rng`; with probability `chance` the child gets `child genome weight + (parent learned − parent genome weight)`, clamped to ±4 (`WEIGHT_CLAMP`), otherwise its genome weight. Fixed reflexes, child synapses without a homologue and every synapse when the parent weight vector has the wrong length keep the genome weight. With `chance` 1 and an unmutated genome the child gets exactly the parent's learned plastic weights. Tests `test_lamarck_chance_is_a_probability_per_synapse` (inherited share ≈ chance), `test_lamarck_draws_are_seeded`.
- `chance` = `lamarckianAssimilationChance` (0.25) in both Ocean and trainer: a probability per synapse, not a scale factor on the offset.
- Ocean mitosis: clones and mutants alike, `thread_rng`; only from parents with `age >= lamarckianAssimilationAgeFloor` (1200), younger parents give the child its genome weights (TCK-149).
- Trainer (toggle "Lamarckian Synapse Inheritance", default off, TCK-128; not age-gated): elites and mutants draw from the slot's seeded breed stream, so runs stay reproducible; an unmutated elite also inherits only ~25 % of its parent's learned weights ([[wiki/simulation/trainer]]).
- Catalogue "cryo-cloning" stores genome, methylations and live synapse weights; restore selectable on spawn (TCK-128).

## Species

- `species_id` is the genome string itself; any mutation creates a new species record (Latin name, parent name, carnivory). There is no distance-based speciation (RV-B10).

## Key Takeaways

- Brain cost still multiplies BMR (RV-S03), with fewer synapses since TCK-182; Lamarckian offsets follow synapse keys (TCK-179) and are inherited per plastic synapse with probability 0.25, the same rule in Ocean and trainer (TCK-152).
- Ocean offspring are not simulated, so the Ocean does not evolve (RV-B01); trainer data is the evolution signal.
- Predation gives no energy gain; species = genome string.

Sources: `.agents/skills/biology-dna/SKILL.md`, TCK-119, TCK-128, TCK-129, TCK-149, TCK-152, TCK-179, TCK-182, TCK-185, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`.
