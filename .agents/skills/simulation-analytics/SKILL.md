---
name: simulation-analytics
description: Procedures for analyzing evolutionary drift, species lineage tracking, population dynamics, and ecosystem balancing in Pixel DNA Life.
---
# simulation-analytics: Ecosystem Balancing & Evolutionary Telemetry

## I. Data Sources (SQLite `pixel_life_local.db` in the Tauri app data dir)
Open read-only (e.g. `sqlite3 -readonly`) while the app runs; the engine writes concurrently.
| Table | Content | Useful columns |
| :--- | :--- | :--- |
| `simulation_state` | Ocean snapshot (`key = 'current_state'`, JSON, every 10 s) | `state_json` -> `creatures[]` (`energy`, `age`, `generation`, `adrenaline`, `neuronStates`), `foodPellets`, `highestGeneration` |
| `simulation_history` | Ocean time series (every 10 s) | `generation`, `population_count`, `average_fitness`/`max_fitness` (actually avg/max energy) |
| `species_records` | One row per distinct genome | `latin_name`, `parent_name`, `status`, `peak_population`, `generation`, `carnivory`, `birth_time`, `extinction_time` |
| `trainer_genomes` | Trainer populations per run/generation | `run_id`, `generation`, `fitness`, `active`, `genome`, `methylations`, `synapse_weights` |
| `creature_catalogue` | User-saved specimens | `source`, `fitness`, `carnivory` |

## II. Caveats Before Drawing Conclusions
- **No Ocean offspring (RV-B01, deliberately deferred):** mitosis children are never simulated. `highestGeneration` and new `species_records` rows grow, but the living population comes only from restocking. Ocean lineage analysis is meaningless until RV-B01 is fixed; use `trainer_genomes` for evolutionary trends.
- **Species = genome string:** every mutant is a "species" (RV-B10). Species counts measure mutation volume, not cladogenesis.
- **Dead config keys:** several `config.json` rules are never read (RV-S01). Verify a key is used in `src/` before tuning it.
- `src/bin/cli_trainer.rs` uses an outdated model (RV-S11); do not use it for balancing.

## III. Balancing Heuristics
1. **Herbivore/predator ratio:** aim for roughly 75% with `carnivory < bitingCarnivoryThreshold` and at most 25% predators; if predators dominate, lower biting gain or raise carnivore metabolic cost.
2. **BMR vs. food supply:** compare `bmrBaseScale` drain against spore availability (`foodSporeCount`) and average time-to-food.
3. **Mitosis cost:** `reproductionSplitLossRatio` must keep growth bounded. Restocking refills to `targetPopulation`; `maxCreatures` exists in config but is not read yet (cap planned with RV-B01).
4. **Measure first:** change one constant at a time, record before/after series from `simulation_history` / `trainer_genomes` in `docs/`.

## IV. Lineage & Diversity
- Build ancestry via `species_records.parent_name -> latin_name`.
- Track letter distribution and length of genomes in `trainer_genomes`: high uniformity = stagnation; high randomness = non-viable mutants.
