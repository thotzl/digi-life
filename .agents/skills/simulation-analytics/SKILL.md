---
name: simulation-analytics
description: Procedures for analyzing evolutionary drift, species lineage tracking, population dynamics, and ecosystem balancing in Pixel DNA Life.
---
# simulation-analytics: Ecosystem Balancing & Evolutionary Telemetry

## I. Data Architecture
- **`simulation_state.json`**: Current frame snapshot containing all active `CreatureAgent` instances.
  - Useful parameters: `energy`, `age`, `generation`, `adrenaline`, `neuronStates`.
- **`species_db.json`**: Historical record of all emerged species.
  - Useful parameters: `parentSpeciesId`, `peakPopulation`, `carnivory` (predatory index).

## II. Ecosystem Balancing Heuristics
When adjusting variables or suggesting parameters, use the following balancing ratios to prevent instant extinction or monoculture:
1. **Autotroph to Heterotroph Ratio:**
   - Ideally, at least **75%** of the population should have a `carnivory` index < 0.55 (Herbivores/Plants), and at most **25%** should be Predators (`carnivory` >= 0.55).
   - If predator count is too high, decrease predation success rate or increase basic metabolic rate (BMR) for carnivores.
2. **BMR (Basal Metabolic Rate) Calibration:**
   - Must be balanced against food spore generation frequency.
   - If creatures die too quickly, check if `basalMetabolicRate` is draining energy faster than the average search-and-consume time.
3. **Mitosis Energy Cost:**
   - `splitLoss` must remain high enough ([0.1 to 0.4]) to prevent exponential population explosions that crash the browser or server.

## III. Evolutionary Drift & Lineage Tracking
- **Cladogenesis Analysis:**
  - Build ancestral trees by linking `speciesId` back to `parentSpeciesId`.
  - Track "peak generation" to see how far evolution has progressed from the first seeded ancestors.
- **Genome Entropy:**
  - Watch for letters `A-Z` distribution in genomes. High uniformity means evolutionary stagnation. High randomness means non-viable mutants. Maintain mutation variance to keep selection active.
