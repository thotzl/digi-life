---
sources: [.agents/skills/simulation-analytics/SKILL.md, docs/revisit/04-simulation-model.md, docs/revisit/07-testing-ci-tooling.md]
---

# Analytics and Balancing

## Data sources

SQLite `pixel_life_local.db` in the app data dir ([[wiki/architecture/persistence]]). Open read-only (`sqlite3 -readonly`) while the app runs.

| Table | Useful columns |
| :--- | :--- |
| `simulation_state` | `state_json` → `creatures[]` (`energy`, `age`, `generation`, `adrenaline`, `neuronStates`), `foodPellets`, `highestGeneration` |
| `simulation_history` | `generation`, `population_count`, `average_fitness`/`max_fitness` (actually avg/max energy) |
| `species_records` | `latin_name`, `parent_name`, `status`, `peak_population`, `generation`, `carnivory`, `birth_time`, `extinction_time` |
| `trainer_genomes` | `run_id`, `generation`, `fitness`, `active`, `genome`, `methylations`, `synapse_weights` |
| `creature_catalogue` | `source`, `fitness`, `carnivory` |

## Caveats

- **No Ocean offspring (RV-B01):** generation counters and species rows grow, but the population is restocked. Ocean lineage analysis is meaningless; use `trainer_genomes` for trends.
- **Species = genome string (RV-B10):** species counts measure mutation volume, not cladogenesis.
- **Keys without effect (RV-S01):** verify a key is read in `src/` before tuning it ([[wiki/architecture/configuration]]).
- **No reproducible runs:** `thread_rng()` everywhere (RV-T04). There is no headless measurement runner (RV-T03).
- **`src/bin/cli_trainer.rs`** (`npm run sim:balance`) feeds a 3-input model from before TCK-122 and writes into the production DB (RV-S11) — not usable for balancing.

## Heuristics

1. Herbivore/predator ratio: aim for ~75 % with `carnivory < bitingCarnivoryThreshold`, ≤ 25 % predators; if predators dominate, lower biting gain or raise carnivore metabolic cost.
2. BMR vs food: compare `bmrBaseScale` drain with spore availability (`foodSporeCount`) and average time-to-food.
3. Mitosis cost: `reproductionSplitLossRatio` must keep growth bounded; restocking refills to `targetPopulation`; `maxCreatures` is not read.
4. Change one constant at a time; record before/after series in this article.

## Lineage and diversity

- Ancestry via `species_records.parent_name → latin_name`.
- Genome letter distribution and length in `trainer_genomes`: high uniformity = stagnation, high randomness = non-viable mutants.

## Measurement results

_(none recorded yet)_

## Key Takeaways

- Trainer data is the only valid evolution signal today.
- Measure before tuning; record results here.

Sources: `.agents/skills/simulation-analytics/SKILL.md`, `docs/revisit/04-simulation-model.md`, `docs/revisit/07-testing-ci-tooling.md`.
