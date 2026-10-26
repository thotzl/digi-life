---
sources: [.agents/skills/biology-dna/SKILL.md, tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, tickets/closed/TCK-129-architecture-refactoring.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-182-brain-topology-compaction.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-145-predation-energy-and-bite-cooldown.md, src/biology/metabolism.rs]
---

# Metabolism, Reproduction and Species

## Energy model (TCK-151, P6)

One energy function for Ocean and trainer: `src/biology/metabolism.rs`. `drain_energy(agent, out_thrust, config.metabolism, scale)` runs right after the brain step and subtracts `tick_cost(...).total() · scale` (Ocean scale 1, trainer `trainerMetabolicScale`, [[wiki/simulation/trainer]]).

`cost per tick = body + senses + brain + movement`, purely additive:

| Term | Formula | Config key (`metabolism`) |
| :--- | :--- | :--- |
| body | `bodyCostPerMass · mass` | `bodyCostPerMass` 2.171e-5 |
| senses | `organCostPerLoad · Σ scale² · (1.1 − bandwidth)` over the organs | `organCostPerLoad` 6.369e-4 |
| brain, per neuron | `neuronCost · neurons` | `neuronCost` 1.589e-5 |
| brain, activity | `activityCost · Σ abs(w · a_source)` | `activityCost` 9.26e-5 |
| movement | `movementCostPerMass · mass · abs(out_thrust)` | `movementCostPerMass` 2.554e-5 |

- `mass` = the physics mass `r^1.5 · L / 25` (`physics::body_mass`, also used for drag, [[wiki/simulation/physics]]).
- `neurons` = thalamus + hidden + motor neurons (`brain_neurons`). Sensor and systemic slots are inputs written by the sensing code; their cost is the organ term.
- Activity = `BrainRuntime::activity()`: `Σ |w · a_source|` over the active synapses of the last brain step, summed in the existing signal pass (0 before the first step). Pruned and zero weights cost nothing; a silent brain pays only the per-neuron term ([[wiki/biology/ctrnn-brain]]).
- Backward thrust costs the same as forward thrust; no thrust costs nothing.
- Ocean only, on top: thermal stress (`thermal_stress · thermalStressPenaltyScale` when the local temperature is more than 0.1 outside the tolerance) and the photosynthesis gain in the light zone ([[wiki/simulation/ocean]]).

### Calibration (derived, not tuned)

- Inputs: `trainer_runner probe metabolism --seed tck151` over 1000 structured genomes ([[wiki/simulation/analytics-and-balancing]]); medians mass 580.355, organ load 1.74513, neurons 14, activity 9.60623 and |thrust| 0.7029 (p10 0.160, p90 0.968) (means per tick over one standard-room trial, learning on, no drain).
- Old Ocean drain of the same genomes, median `D = 27 (BMR) · 0.002 · 1.0 · 0.45 = 0.0243` per tick.
- Shares at rest for the median phenotype: body 85 %, senses 7.5 %, brain 7.5 % (together 15 %, TCK-171 band 10–20 %); the brain share splits 20 % per neuron / 80 % activity. Basis: cost per neuron roughly constant (Herculano-Houzel 2011), signalling cost activity-driven (Attwell & Laughlin 2001), vision 5–15 % of resting metabolism in fish (Moran et al. 2015) ([[wiki/research/neuroevolution-and-training-literature]]).
- Movement: full thrust costs the resting cost of the median body; for any body full thrust costs about its resting cost (`body / 0.85`), so swimming is not free.
- Absolute scale (lead decision, after the first P6 Ocean check): the median **typical total** cost (resting at the genome's trial activity + movement at its trial mean |thrust|) equals `D`, so active creatures pay what the old model charged. With the shares above at unit scale that median was 0.039834, so all five constants are multiplied by `0.0243 / 0.039834 = 0.6100`; shares and the movement ratio are unchanged. Matching only the resting cost to `D` (the first P6 calibration) doubled the Ocean death turnover (module docs of `metabolism.rs`).
- Measured by the probe after P6 (structured set, p10 / p50 / p90): resting cost 0.0100 / 0.0152 / 0.0222 per tick; typical total cost 0.0146 / 0.0243 / 0.0381; brain + senses share of the resting cost per genome 7.2 % / 15.2 % / 29.2 %; full-thrust / resting cost 0.83 / 1.00 / 1.09. Letter-soup set: share median 10.4 %, typical total median 0.0237.
- Tests in `metabolism::tests`: median-phenotype and per-genome shares in [10 %, 20 %] (1000 genomes); `test_typical_total_cost_matches_the_old_ocean_drain` (200 genomes: median activity within ±15 % of 9.606, median |thrust| within ±15 % of 0.7029, median typical total cost within ±10 % of 0.0243); full-thrust ratio in [0.8, 1.25]; silent synapses cost nothing and the cost is the plain sum of its terms ([[wiki/quality/testing]]).

### Replaced model

- Before P6 the cost was a multiplicative BMR compiled into the phenotype, `(body area + organ tax) · (1 + organs · 5 · 0.08) · 1.4 · stiffness · (1 + synapses · 0.06)`, drained in the Ocean as `basal_metabolic_rate · bmrBaseScale · basalMetabolicRateMultiplier · 0.45`; the trainer had no drain. One sense organ raised the BMR several-fold through the synapse factor (RV-S03).
- Removed (TCK-151): `brain_complexity_multiplier`, the organ tax as a factor, the phenotype field `basal_metabolic_rate` (`basalMetabolicRate` in the bindings), the Ocean `* 0.45` literal, the keys `basalMetabolicRateMultiplier` and `bmrBaseScale`. RV-S03 is resolved: organs and neurons add cost linearly. The organ load keeps the shape of the former organ tax.
- `CreatureAgent.adrenaline` and the always-1 surcharge factor were removed in P1 (TCK-149).

## Start energy (TCK-185)

- Every spawned creature starts with `stomach_capacity · creatureStartStomachRatio` (0.60), defined once in `creature_factory::start_energy` and used by all Ocean spawn sites and the trainer (also on every trainer trial reset).
- Intended changes against the previous per-site values: trainer 100 → stomach × 0.60 (the brain input `internal_energy = energy / 100` changes with it, [[wiki/biology/sensing]]); `SPAWN_CLONE` and catalogue assignment to a trainer sandbox 0.8 → 0.60. `antisense` and neuron buffers are always set from the phenotype.
- Ocean mitosis children also go through the factory (`spawn_from_phenotype`), but take the parent's post-split energy instead of the start energy (below, TCK-152).

## Food and predation

- Herbivores eat plant spores, carnivores eat meat; dead creatures decompose into meat spores (`type_id = 2`), eaten meat recycles into algae (TCK-129).
- **Food energy** (TCK-178): one rule `metabolism::food_energy` for Ocean and trainer, `foodSporeEnergy (15) · diet efficiency · scale`; plant `(1 − carnivory) · grazingEfficiencyHerbivoreScale` (1.5), meat `carnivory · bitingEfficiencyCarnivoreScale` (1.25). A pure herbivore gets 22.5 per plant spore, a pure carnivore 18.75 per meat spore. Energy is capped at `stomach_capacity`; eating sets `has_eaten` (Ocean).
- **Predation** (Ocean only, TCK-145): an agent with `carnivory ≥ bitingCarnivoryThreshold` (0.35) bites at most once per tick, the nearest eligible victim within `mean_radius · bitingRadiusMultiplier + bitingRadiusOffset` (`bite_target` in `engine.rs`). Not eligible: itself, its own species, and a victim this attacker bit within the last `biteCooldownTicks` (60; per attacker–victim pair, map cleared on `RESET_EVOLUTION`). The value 60 is a default without measurement.
- A bite deals `bitingEnergyDamage` (35) to the victim (applied after the step) and gives the attacker `bitingBaseEnergyGain · carnivory · bitingEfficiencyCarnivoreScale` (35 · carnivory · 1.25), capped by the damage dealt and by the victim's energy (`metabolism::bite_energy_gain`): a bite moves energy, it never creates it. Full gain (35) from carnivory 0.8; at the threshold 0.35 the gain is ≈ 15.3. The attacker's energy is capped at `stomach_capacity`; a bite sets `has_eaten`, so predators can reach mitosis by biting. RV-B07 is resolved.
- The victim-energy cap uses what the victim still has after earlier bites in the same tick (energy at step start minus the damage already recorded), so several attackers together never take more than the victim had.
- The rule "do not bite the same species" only applies to exact clones (species = genome, see below).

## Starvation and death

- Ocean: an agent dies at `energy ≤ 0` or at its maximum age (lifespan below) and decomposes into meat spores. The damage pass after the step removes every agent with energy ≤ 0.1 (also after metabolism or thermal stress).
- Death log (`death_log` in `engine.rs`): "hunted down by predators" only when the agent took bite damage in that tick, otherwise the starvation/senescence message. Test `test_death_without_bite_is_not_reported_as_predation`. Before P6 every death in the damage pass was logged as predation (P5 check: 101 "hunted" deaths with 0 bites in 90 s).
- Trainer: a creature at 0 energy is starved for the rest of the trial (no brain step, no movement, no eating) and recovers on the next trial reset ([[wiki/simulation/trainer]], TCK-178).

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

- One additive, activity-based energy function for Ocean and trainer (body 85 %, senses 7.5 %, brain 7.5 % of the median resting cost; full thrust ≈ resting cost); the multiplicative BMR is gone (TCK-151, RV-S03 resolved).
- Lamarckian offsets follow synapse keys (TCK-179) and are inherited per plastic synapse with probability 0.25, the same rule in Ocean and trainer (TCK-152).
- Ocean offspring are not simulated, so the Ocean does not evolve (RV-B01); trainer data is the evolution signal.
- A bite moves energy from victim to attacker (capped by damage and victim energy), at most one bite per attacker and tick with a per-pair cooldown (TCK-145, RV-B07 resolved); species = genome string.

Sources: `.agents/skills/biology-dna/SKILL.md`, TCK-119, TCK-128, TCK-129, TCK-145, TCK-149, TCK-151, TCK-152, TCK-178, TCK-179, TCK-182, TCK-185, `src/biology/metabolism.rs`, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`.
