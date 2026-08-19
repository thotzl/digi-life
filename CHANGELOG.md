# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-08-19

### Added
- **SQLite Storage Integration:** Migrated from raw JSON file storage (`species_db.json` and `simulation_state.json`) to a fast, transactional, multi-connection-safe SQLite database (`digilife.db`) running in WAL (Write-Ahead Logging) mode.
- **Testing Architecture:** Added full Vitest integration with `@vitest/coverage-v8` and configured Stryker Mutator (`stryker.config.json`) for mutation testing.
- **Unit and Integration Tests (29 passing tests):**
  - Covered 100% of CTRNN Euler-integration bounds, genetic base-pairing decoding, and mutation guard rails.
  - Covered 100% of API endpoints, SQLite queries, and concurrent transactions.
  - Covered physical boundary reflections, fluid drag, nutrient spore absorption, and asexual mitosis.
- **Luminosity Flash Handling:** Headless server now parses the 4th CTRNN output node ("Biolum Flash"). Active flash ($>0.5$ intensity) triggers a metabolic energy tax of $0.05 \times \text{intensity}$ and broadcasts a `"FLASH_EVENT"` to all visual web clients.
- **Newborn Species Ledger:** Centralized a new `registerSpeciesIfNew` helper that tracks and records all newly evolved genomes (mitosis mutations, random founders, and manually injected specimens) to SQLite with strict lineage parent IDs.

### Fixed
- **Synaptic Index Mapping Bottleneck:** Fixed the genomic mapping bottleneck where synapses in brains with $>26$ total nodes could never choose presynaptic nodes $\ge 26$, leaving hidden layers isolated.
- **Mutation Engine Crash:** Patched `mutateGenome` to prevent runtime division-by-zero / NaN index crashes when passed an empty string.

---

## [1.0.0] - 2026-08-19

### Added
- Initial workspace repository setup (ref TCK-101).
- Scaffolded Vite v5 client monitor and tsx WebSocket simulation server.
- Built AI local skills structure under `.agents/skills/`.
