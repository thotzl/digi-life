---
sources: [.agents/skills/biology-dna/SKILL.md, tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, tickets/closed/TCK-129-architecture-refactoring.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-185-creature-factory.md]
---

# Metabolism, Reproduction and Species

## Metabolism

- BMR scales with body and brain: `brain_complexity_multiplier = 1 + synapses · 0.06` times `(1 + k_count · 0.08)` (`dna.rs`). The Ocean compensates with a `* 0.45` factor (`engine.rs`).
- One organ adds ~10 thalamus sources → ~40–120 synapses → BMR ×3–×8: strong selection pressure against sense organs (RV-S03).
- Adrenaline is constant 1.0 and has no metabolic effect; the adrenaline config keys and the BMR surcharge factor were removed (TCK-149).

## Start energy (TCK-185)

- Every spawned creature starts with `stomach_capacity · creatureStartStomachRatio` (0.60), defined once in `creature_factory::start_energy` and used by all Ocean spawn sites and the trainer (also on every trainer trial reset).
- Intended changes against the previous per-site values: trainer 100 → stomach × 0.60 (the brain input `internal_energy = energy / 100` changes with it, [[wiki/biology/sensing]]); `SPAWN_CLONE` and catalogue assignment to a trainer sandbox 0.8 → 0.60. `antisense` and neuron buffers are always set from the phenotype.
- Ocean mitosis children are not spawned through the factory: they clone the parent agent (below).

## Food and predation

- Herbivores eat plant spores, carnivores eat meat; dead creatures decompose into meat spores (`type_id = 2`), eaten meat recycles into algae (TCK-129).
- Biting gives the attacker no energy (`bitingBaseEnergyGain` unread, exempt in the config read-site test, TCK-145); the victim loses 35. There is no bite cooldown: the same victim can be bitten every tick, with several bite events per tick (RV-B07).
- The rule "do not bite the same species" only applies to exact clones (species = genome, see below).

## Reproduction (Ocean mitosis)

- Condition: `age >= max(phenotype.mature_age, creatureMatureAgeFloor) && has_eaten && population < maxCreatures && energy >= stomach_capacity * max(phenotype.repro_threshold, reproductionStomachThresholdFloor)`; parent energy multiplied by `reproductionSplitLossRatio` (TCK-149).
- Floors apply at the read site in `engine.rs`; `parse_genome` is unchanged. Mature age floor 600 raises phenotype values 300–600 to 600; threshold floor 0.60 equals the phenotype minimum (range 0.60–0.95), so it has no effect with current values.
- `population` = `creatures.len()` at step start, cap `maxCreatures` (45). Since children are not simulated (below), the cap has no practical effect.
- Child genome mutated with 35 % probability; clones (65 %) copy the agent incl. `neuron_states` and learned weights.
- **Children are not simulated:** they are pushed only to `newly_spawned_creatures`, never into the simulated list; the client adds and removes them again. The Ocean has no heredity; population comes only from restocking. `highestGeneration` and species rows still grow (RV-B01).

## Lifespan

- `max_age_ticks = min(mature_age * creatureLifespanMatureAgeMultiplier (12), creatureMaxAgeTicks (36000))` (`engine.rs`, TCK-149), DNA-dependent (TCK-129) → 3,600–32,400 ticks; the cap lies above the maximum and does not bind with current values.
- Lifespan and maturity are still split between phenotype and config; no single source of truth (RV-B08).

## Lamarckian inheritance

- Mutated children inherit the parent's learned offsets `ΔW = W_active − W_base` on homologous synapses (same from/to), scaled by `lamarckian_assimilation_chance` (TCK-119). The value acts as a scale factor, not a probability (RV-S05).
- Ocean only: offsets are passed only when parent `age >= lamarckianAssimilationAgeFloor` (1200); younger parents give the child its genetic base weights (TCK-149). The trainer Lamarck toggle is not age-gated.
- Trainer toggle "Lamarckian Synapse Inheritance" (default off) (TCK-128).
- Catalogue "cryo-cloning" stores genome, methylations and live synapse weights; restore selectable on spawn (TCK-128).

## Species

- `species_id` is the genome string itself; any mutation creates a new species record (Latin name, parent name, carnivory). There is no distance-based speciation (RV-B10).

## Key Takeaways

- Brain cost dominates BMR and penalizes organs (RV-S03).
- Ocean offspring are not simulated, so the Ocean does not evolve (RV-B01); trainer data is the evolution signal.
- Predation gives no energy gain; species = genome string.

Sources: `.agents/skills/biology-dna/SKILL.md`, TCK-119, TCK-128, TCK-129, TCK-149, TCK-185, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`.
