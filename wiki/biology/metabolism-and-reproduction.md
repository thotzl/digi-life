---
sources: [.agents/skills/biology-dna/SKILL.md, tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, tickets/closed/TCK-129-architecture-refactoring.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md]
---

# Metabolism, Reproduction and Species

## Metabolism

- BMR scales with body and brain: `brain_complexity_multiplier = 1 + synapses · 0.06` times `(1 + k_count · 0.08)` (`dna.rs`). The Ocean compensates with a `* 0.45` factor (`engine.rs`).
- One organ adds ~10 thalamus sources → ~40–120 synapses → BMR ×3–×8: strong selection pressure against sense organs (RV-S03).
- Adrenaline is constant 1.0 (all adrenaline keys unused, RV-S01).

## Food and predation

- Herbivores eat plant spores, carnivores eat meat; dead creatures decompose into meat spores (`type_id = 2`), eaten meat recycles into algae (TCK-129).
- Biting gives the attacker no energy (`bitingBaseEnergyGain` unread); the victim loses 35. There is no bite cooldown: the same victim can be bitten every tick, with several bite events per tick (RV-B07).
- The rule "do not bite the same species" only applies to exact clones (species = genome, see below).

## Reproduction (Ocean mitosis)

- Condition: `age >= phenotype.mature_age && has_eaten && energy >= stomach_capacity * repro_threshold`; parent energy multiplied by `reproductionSplitLossRatio`.
- Child genome mutated with 35 % probability; clones (65 %) copy the agent incl. `neuron_states` and learned weights.
- **Children are not simulated:** they are pushed only to `newly_spawned_creatures`, never into the simulated list; the client adds and removes them again. The Ocean has no heredity; population comes only from restocking. `highestGeneration` and species rows still grow (RV-B01).

## Lifespan

- `max_age_ticks = mature_age * 12` (`engine.rs`), DNA-dependent (TCK-129) → 3,600–32,400 ticks. `creatureMaxAgeTicks` (36000) has no effect (RV-B08).

## Lamarckian inheritance

- Mutated children inherit the parent's learned offsets `ΔW = W_active − W_base` on homologous synapses (same from/to), scaled by `lamarckian_assimilation_chance` (TCK-119). The value acts as a scale factor, not a probability (RV-S05).
- Trainer toggle "Lamarckian Synapse Inheritance" (default off) (TCK-128).
- Catalogue "cryo-cloning" stores genome, methylations and live synapse weights; restore selectable on spawn (TCK-128).

## Species

- `species_id` is the genome string itself; any mutation creates a new species record (Latin name, parent name, carnivory). There is no distance-based speciation (RV-B10).

## Key Takeaways

- Brain cost dominates BMR and penalizes organs (RV-S03).
- Ocean offspring are not simulated, so the Ocean does not evolve (RV-B01); trainer data is the evolution signal.
- Predation gives no energy gain; species = genome string.

Sources: `.agents/skills/biology-dna/SKILL.md`, TCK-119, TCK-128, TCK-129, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`.
