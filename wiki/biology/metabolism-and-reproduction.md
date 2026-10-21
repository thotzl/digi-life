---
sources: [.agents/skills/biology-dna/SKILL.md, .tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, .tickets/closed/TCK-129-architecture-refactoring.md, .tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md]
---

# Metabolism, Reproduction and Species

## Metabolism

- BMR scales with body and brain: `brain_complexity_multiplier = 1 + synapses · 0.06` times `(1 + k_count · 0.08)` (`dna.rs`). The Ocean compensates with a `* 0.45` hack (`engine.rs`).
- **RV-S03 (TCK-151):** one organ adds ~10 thalamus sources → ~40–120 synapses → BMR ×3–×8. Strong selection pressure **against** sense organs, contrary to the design goal. Target: linear cost per active synapse, own organ cost term, move/remove the 0.45 hack.
- Adrenaline is constant 1.0 (all adrenaline keys unused, RV-S01).

## Food and predation

- Herbivores eat plant spores, carnivores eat meat; dead creatures decompose into meat spores (`type_id = 2`), eaten meat recycles into algae (TCK-129).
- **RV-B07 (TCK-145):** biting gives the attacker no energy (`bitingBaseEnergyGain` unread); the victim loses 35. No bite cooldown: same victim every tick, several bite events per tick. Target: gain = `biting_base_energy_gain * carnivory * biting_efficiency_carnivore_scale`, cooldown per pair.
- Predation rule "do not bite the same species" only applies to exact clones (species = genome, see below).

## Reproduction (Ocean mitosis)

- Condition: `age >= phenotype.mature_age && has_eaten && energy >= stomach_capacity * repro_threshold`; parent energy multiplied by `reproductionSplitLossRatio`.
- Child genome mutated with 35 % probability; clones (65 %) copy the agent incl. `neuron_states` and learned weights.
- **RV-B01 (P0, deliberately deferred, TCK-143):** children are pushed only to `newly_spawned_creatures` and never into the simulated list; the client adds and removes them again. **The Ocean has no heredity**; population comes only from restocking. `highestGeneration` and species rows still grow — telemetry suggests evolution that does not happen. Fix couples with species retention (RV-D03).

## Lifespan

- `max_age_ticks = mature_age * 12` (`engine.rs`), DNA-dependent (TCK-129) → 3,600–32,400 ticks.
- **RV-B08 (TCK-146):** `creatureMaxAgeTicks` (36000) is ignored — contradicts the config SSOT.

## Lamarckian inheritance

- Mutated children inherit the parent's learned offsets `ΔW = W_active − W_base` on homologous synapses (same from/to), scaled by `lamarckian_assimilation_chance` (TCK-119).
- Trainer toggle "Lamarckian Synapse Inheritance" (default off) (TCK-128).
- Catalogue "cryo-cloning" stores genome, methylations and live synapse weights; restore selectable on spawn (TCK-128).
- **RV-S05 (TCK-152):** the "chance" is used as a scale factor, not a probability; clones copy full weights, mutants 25 % of the offset — inconsistent semantics. Depends on RV-B01.

## Species

- `species_id` is the genome string itself; any mutation creates a new species record (Latin name, parent name, carnivory). No distance-based speciation.
- **RV-B10 (TCK-148, design decision):** target = speciation by genetic distance to the founder, species ID as UUID/counter, Latin name not UNIQUE.

## Key Takeaways

- Brain cost dominates BMR and punishes organs (TCK-151).
- Ocean offspring never live — no Ocean evolution until TCK-143; use trainer data for evolutionary trends.
- Predation, lifespan and species model each have a pending decision ticket.

Sources: `.agents/skills/biology-dna/SKILL.md`, TCK-119, TCK-128, TCK-129, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`.
