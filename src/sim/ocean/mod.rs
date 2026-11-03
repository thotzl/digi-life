//! `OceanSim` (TCK-156): the Ocean ecosystem as an IO-free simulation. It owns the population, the spore field, the
//! species ledger and its RNG; `tick()` runs one Ocean step on the shared creature systems (`sim::step_creature` with
//! `WorldSettings::ocean`, then bite and mitosis) and returns what happened (`OceanStep`) instead of writing SQL or
//! emitting UI events. The engine's `OceanMode` maps the step onto events and the species store. Contract:
//! `wiki/contracts/mode.md`; systems: `wiki/contracts/creature-system.md`.
//!
//! Randomness comes only from seeded `SimRng` streams of `seed` (`shared::rng::derive`), so a seed and the same
//! inputs give the same Ocean on every platform (headless Ocean runner: TCK-158).

pub mod snapshot;
pub mod spawner;
pub mod species_ledger;
pub mod spore_field;

use std::collections::HashMap;

use rand::Rng;

use crate::config::AppConfig;
use crate::shared::dto::CreatureAgent;
use crate::shared::map_generator::{ProceduralWorld, generate_world};
use crate::shared::physics::SporePusher;
use crate::shared::rng::{SimRng, derive, seed_from_str};
use crate::shared::spatial_grid::{GridLayer, SpatialGrid};
use crate::sim::systems::bite::{BiteRule, bite};
use crate::sim::systems::death::{decompose, is_dead_energy};
use crate::sim::systems::eat::respawn;
use crate::sim::systems::food_drift::drift_before_creatures;
use crate::sim::systems::reproduce::{self, MitosisRule};
use crate::sim::{CreatureEvent, CreatureStep, PeerSnapshot, TickCtx, World, WorldSettings, step_creature};

pub use snapshot::OceanSnapshot;
pub use spawner::{CatalogueFounder, Founder, OceanSpawner};
pub use species_ledger::{LedgerEntry, SpeciesChange, SpeciesLedger};
pub use spore_field::SporeField;

/// Founders of a fresh Ocean (mutated wildtype).
pub const FRESH_FOUNDERS: usize = 15;
/// Founders after `RESET_EVOLUTION` (catalogue clones or structured random genomes).
pub const RESET_FOUNDERS: usize = 25;
/// Cell size of the spatial grid (px).
pub const GRID_CELL: f32 = 80.0;
/// Characters of the attacker's name in a bite log line.
pub const BITE_NAME_CHARS: usize = 16;

/// RNG streams of `derive(seed, [stream, index])`; the index is the tick (food, mitosis, restock) or the release
/// count (action). Populate has no index: one stream per seed.
pub const STREAM_FOOD: u64 = 1;
pub const STREAM_MITOSIS: u64 = 2;
pub const STREAM_RESTOCK: u64 = 3;
pub const STREAM_ACTION: u64 = 4;
pub const STREAM_POPULATE: u64 = 5;

/// Seed of an Ocean: `seed_from_str("ocean:{config_seed}")`; a reset adds a nonce (`"ocean:{config_seed}:{nonce}"`) so
/// it starts a new Ocean instead of replaying the first one.
pub fn ocean_seed(config_seed: &str, nonce: Option<u64>) -> u64 {
    match nonce {
        Some(nonce) => seed_from_str(&format!("ocean:{}:{}", config_seed, nonce)),
        None => seed_from_str(&format!("ocean:{}", config_seed)),
    }
}

/// Something that happened in the Ocean, in order. The engine turns these into UI events and log lines.
#[derive(Debug, Clone, PartialEq)]
pub enum OceanEvent {
    /// A spore was eaten at (x, y) (its position before the respawn)
    Ate { x: f32, y: f32 },
    /// `attacker` bit `victim` at the victim's position
    Bite {
        attacker: u32,
        attacker_name: String,
        victim: u32,
        damage: f32,
        gain: f32,
        x: f32,
        y: f32,
    },
    /// A creature died and decomposed
    Died {
        id: u32,
        latin_name: String,
        cause: DeathCause,
    },
    /// `parent` split; the child's genome mutated when `mutated` (a new species unless the genome or name is known)
    Mitosis {
        parent: u32,
        child_name: String,
        generation: u32,
        mutated: bool,
    },
    /// The last member of a species died
    Extinct { latin_name: String },
    /// A founder refilled the population (`catalogue_name`: catalogue clone, else structured random)
    Restocked { id: u32, catalogue_name: Option<String> },
}

/// Why a creature died (TCK-158).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    /// No energy left (`death::is_dead_energy`): in its step, or after a tick without bite damage
    Starved,
    /// Reached its lifespan with energy left
    Aged,
    /// Took bite damage this tick and has no energy left
    Hunted,
}

/// Result of one tick or action.
#[derive(Debug, Clone, Default)]
pub struct OceanStep {
    pub events: Vec<OceanEvent>,
    /// Changes for the species store, in order
    pub species: Vec<SpeciesChange>,
    /// Creatures the UI has not seen yet (mitosis children, restocked and released founders); all of them are already
    /// in the population
    pub born: Vec<CreatureAgent>,
}

/// Why a release was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnRejected {
    /// `ocean.maxCreatures` reached
    PopulationCap(usize),
}

/// The Ocean ecosystem. Mitosis children join the population and are reported once in `OceanStep::born` (TCK-143).
pub struct OceanSim {
    config: AppConfig,
    world: ProceduralWorld,
    settings: WorldSettings,
    creatures: Vec<CreatureAgent>,
    spores: SporeField,
    ledger: SpeciesLedger,
    next_creature_id: u32,
    highest_generation: u32,
    seed: u64,
    tick: u64,
    actions: u64,
    /// Last bite tick per (attacker, victim) for the bite cooldown (TCK-145)
    bite_ticks: HashMap<(u32, u32), u64>,
    // Reused per-tick buffers (no allocation in the hot loop)
    grid: SpatialGrid,
    peers: Vec<PeerSnapshot>,
    step: CreatureStep,
    nearby: Vec<u32>,
    pushers: Vec<SporePusher>,
    damage: HashMap<u32, f32>,
    alive: Vec<bool>,
}

impl OceanSim {
    /// An empty Ocean (no creatures, no spores) of the configured world size; the procedural world comes from
    /// `config.core.seed`.
    pub fn empty(config: &AppConfig, seed: u64) -> Self {
        let (width, height) = (config.ocean.world_width, config.ocean.world_height);
        Self {
            config: config.clone(),
            world: generate_world(&config.core.seed, width, height),
            settings: WorldSettings::ocean(&config.ocean, Vec::new()),
            creatures: Vec::new(),
            spores: SporeField::default(),
            ledger: SpeciesLedger::default(),
            next_creature_id: 1,
            highest_generation: 1,
            seed,
            tick: 0,
            actions: 0,
            bite_ticks: HashMap::new(),
            grid: SpatialGrid::new(width, height, GRID_CELL),
            peers: Vec::new(),
            step: CreatureStep::default(),
            nearby: Vec::new(),
            pushers: Vec::new(),
            damage: HashMap::new(),
            alive: Vec::new(),
        }
    }

    /// A fresh Ocean: `FRESH_FOUNDERS` mutated wildtype founders and `ocean.foodSporeCount` spores.
    pub fn fresh(config: &AppConfig, seed: u64) -> (Self, OceanStep) {
        let mut sim = Self::empty(config, seed);
        let mut out = OceanStep::default();
        let mut rng = derive(seed, &[STREAM_POPULATE]);
        for _ in 0..FRESH_FOUNDERS {
            sim.add_founder(Founder::Wildtype, &mut rng, &mut out);
        }
        sim.reseed_spores(&mut rng);
        (sim, out)
    }

    /// Continues a saved Ocean. A snapshot without a seed (before TCK-156) takes `fallback_seed`; one without
    /// nutrient centres gets new ones from the seed. The ledger stays empty (the engine loads it from the store).
    pub fn restore(config: &AppConfig, snapshot: OceanSnapshot, fallback_seed: u64) -> Self {
        let mut sim = Self::empty(config, snapshot.seed.unwrap_or(fallback_seed));
        sim.creatures = snapshot.creatures;
        sim.spores = SporeField {
            spores: snapshot.food_pellets,
            nutrient_centers: snapshot.nutrient_centers,
            next_id: snapshot.next_spore_id,
        };
        if sim.spores.nutrient_centers.is_empty() {
            let (width, height) = sim.size();
            sim.spores
                .regenerate_centers(&mut derive(sim.seed, &[STREAM_POPULATE]), width, height);
        }
        sim.sync_settings();
        sim.highest_generation = snapshot.highest_generation;
        sim.next_creature_id = snapshot.next_creature_id;
        sim.tick = snapshot.tick;
        sim.actions = snapshot.actions;
        sim
    }

    pub fn snapshot(&self) -> OceanSnapshot {
        OceanSnapshot {
            creatures: self.creatures.clone(),
            food_pellets: self.spores.spores.clone(),
            highest_generation: self.highest_generation,
            next_creature_id: self.next_creature_id,
            next_spore_id: self.spores.next_id,
            nutrient_centers: self.spores.nutrient_centers.clone(),
            seed: Some(self.seed),
            tick: self.tick,
            actions: self.actions,
        }
    }

    /// `RESET_EVOLUTION`: a new Ocean under `seed` with `RESET_FOUNDERS` founders (random catalogue clones, or
    /// structured random genomes with an empty catalogue) and a new spore field. Clears the ledger and the bite
    /// cooldowns; creature ids restart at 1, spore ids continue.
    pub fn reset(&mut self, seed: u64, catalogue: &[CatalogueFounder]) -> OceanStep {
        self.seed = seed;
        self.tick = 0;
        self.actions = 0;
        self.creatures.clear();
        self.bite_ticks.clear();
        self.ledger.clear();
        self.next_creature_id = 1;
        self.highest_generation = 1;
        let mut out = OceanStep::default();
        let mut rng = derive(seed, &[STREAM_POPULATE]);
        for _ in 0..RESET_FOUNDERS {
            let founder = pick_founder(catalogue, &mut rng).0;
            self.add_founder(founder, &mut rng, &mut out);
        }
        self.reseed_spores(&mut rng);
        out
    }

    /// Releases one founder (`INJECT_URZELLE`: `Founder::Random`, `SPAWN_CLONE`: `Founder::Genome`); rejected at
    /// `ocean.maxCreatures`. The founder is the last creature and `born[0]` of the result.
    pub fn release(&mut self, founder: Founder) -> Result<OceanStep, SpawnRejected> {
        if self.creatures.len() >= self.config.ocean.max_creatures {
            return Err(SpawnRejected::PopulationCap(self.config.ocean.max_creatures));
        }
        self.actions += 1;
        let mut rng = derive(self.seed, &[STREAM_ACTION, self.actions]);
        let mut out = OceanStep::default();
        self.add_founder(founder, &mut rng, &mut out);
        out.born.push(self.creatures.last().expect("founder added").clone());
        Ok(out)
    }

    /// One Ocean step plus the restock: `catalogue` is called only when the population fell below
    /// `ocean.targetPopulation`.
    pub fn tick(&mut self, catalogue: impl FnOnce() -> Vec<CatalogueFounder>) -> OceanStep {
        let mut out = self.step();
        if self.creatures.len() < self.config.ocean.target_population {
            self.restock(&catalogue(), &mut out);
        }
        out
    }

    /// One Ocean step without the restock. Order: spatial grid, food drift (before all creatures), peer snapshot;
    /// per creature `step_creature` (sense, think, metabolize, death, move, eat) with the eat and decomposition events
    /// applied right after the creature's step; then per surviving creature age, bite (no target that died this tick)
    /// and mitosis; bite damage after all creatures (creatures without energy left, `death::is_dead_energy`,
    /// decompose); mitosis children join the population (a mutated child's species is discovered here); species
    /// census.
    pub fn step(&mut self) -> OceanStep {
        let mut out = OceanStep::default();
        self.tick += 1;
        let tick = self.tick;
        let (width, height) = self.size();
        let config = &self.config;

        self.grid.clear();
        for (i, spore) in self.spores.spores.iter().enumerate() {
            self.grid.insert_food(i as u32, spore.x, spore.y);
        }
        for (i, agent) in self.creatures.iter().enumerate() {
            self.grid.insert_creature(i as u32, agent.px, agent.py);
        }
        self.peers.clear();
        self.peers.extend(self.creatures.iter().map(PeerSnapshot::from));
        self.damage.clear();
        let cooldown = config.ocean.bite_cooldown_ticks as u64;
        self.bite_ticks.retain(|_, &mut bitten| tick - bitten < cooldown);
        let population = self.creatures.len();
        self.alive.clear();
        self.alive.resize(population, true);

        let mut food_rng = derive(self.seed, &[STREAM_FOOD, tick]);
        let mut mitosis_rng = derive(self.seed, &[STREAM_MITOSIS, tick]);
        let bite_rule = BiteRule::from_ocean(&config.ocean);
        let mitosis_rule = MitosisRule::from_ocean(&config.ocean);
        let ctx = TickCtx::new(config, tick);
        let (grid, peers, nearby, bite_ticks, damage) = (
            &self.grid,
            &self.peers,
            &mut self.nearby,
            &mut self.bite_ticks,
            &mut self.damage,
        );

        let mut world = World {
            creatures: &mut self.creatures,
            foods: &mut self.spores.spores,
            obstacles: &self.world.obstacles,
            peers,
            grid: Some(grid),
            width,
            height,
            settings: &self.settings,
        };
        drift_before_creatures(&mut world, nearby, &mut self.pushers);

        for idx in 0..population {
            let step = &mut self.step;
            step_creature(&mut world, idx, &ctx, step);
            for event in &step.events {
                match *event {
                    CreatureEvent::Ate { food, x, y, .. } => {
                        let agent = &mut world.creatures[idx];
                        agent.has_eaten = true;
                        let from = (agent.px, agent.py);
                        respawn(
                            &mut world.foods[food],
                            &world.settings.respawn,
                            from,
                            width,
                            height,
                            &mut food_rng,
                        );
                        out.events.push(OceanEvent::Ate { x, y });
                    }
                    CreatureEvent::Decomposed { x, y, spores } => decompose(world.foods, x, y, spores, &mut food_rng),
                }
            }
            let agent = &world.creatures[idx];
            if step.died {
                self.alive[idx] = false;
                let cause = if is_dead_energy(agent.energy) {
                    DeathCause::Starved
                } else {
                    DeathCause::Aged
                };
                out.events.push(OceanEvent::Died {
                    id: agent.id,
                    latin_name: agent.phenotype.latin_name.clone(),
                    cause,
                });
            }
        }

        // Bite and mitosis after every creature's step (TCK-202): a creature that died in its own step this tick is
        // no bite target (it decomposes instead of feeding an attacker)
        let alive = &self.alive;
        // Mitosis children with the parent's name for a mutated child's species
        let mut births: Vec<(CreatureAgent, Option<String>)> = Vec::new();
        for idx in 0..population {
            if !alive[idx] {
                continue;
            }
            let agent = &mut world.creatures[idx];
            agent.age += 1;

            // Bite (TCK-145): candidates from the grid, read from the snapshot of the tick start
            if BiteRule::is_biter(agent) {
                let range = bite_rule.range(agent.phenotype.spinal_harmonics.mean_radius);
                nearby.clear();
                grid.for_each_nearby(GridLayer::Creatures, agent.px, agent.py, range, |i| nearby.push(i));
                let candidates = nearby
                    .iter()
                    .filter(|&&i| alive[i as usize])
                    .filter_map(|&i| peers.get(i as usize));
                let attacker = agent.id;
                if let Some(b) = bite(agent, candidates, &bite_rule, &config.metabolism, damage, |victim| {
                    bite_ticks.contains_key(&(attacker, victim))
                }) {
                    bite_ticks.insert((b.attacker, b.victim), tick);
                    let attacker_name = agent.phenotype.latin_name.chars().take(BITE_NAME_CHARS).collect();
                    out.events.push(OceanEvent::Bite {
                        attacker,
                        attacker_name,
                        victim: b.victim,
                        damage: config.metabolism.biting_energy_damage,
                        gain: b.gain,
                        x: b.x,
                        y: b.y,
                    });
                }
            }

            // Mitosis (TCK-152): the child joins the population after the damage pass (TCK-143); the cap counts the
            // children of this tick
            if reproduce::ready(agent, &mitosis_rule, population + births.len()) {
                reproduce::split(agent, &mitosis_rule);
                let (child, mutated) = reproduce::child(
                    agent,
                    self.next_creature_id,
                    &mitosis_rule,
                    &config.genome,
                    &config.brain,
                    &config.metabolism,
                    &mut mitosis_rng,
                );
                out.events.push(OceanEvent::Mitosis {
                    parent: agent.id,
                    child_name: child.phenotype.latin_name.clone(),
                    generation: child.generation,
                    mutated,
                });
                self.highest_generation = self.highest_generation.max(child.generation);
                self.next_creature_id += 1;
                births.push((child, mutated.then(|| agent.phenotype.latin_name.clone())));
            }
        }

        // Bite damage after all creatures; creatures without energy left decompose
        let (alive, spores, events) = (&self.alive, &mut self.spores.spores, &mut out.events);
        let mut i = 0;
        self.creatures.retain_mut(|agent| {
            let in_step_alive = alive[i];
            i += 1;
            if !in_step_alive {
                return false;
            }
            let bite_damage = damage.get(&agent.id).copied();
            if let Some(d) = bite_damage {
                agent.energy -= d;
            }
            if !is_dead_energy(agent.energy) {
                return true;
            }
            let spore_count = self.settings.death.spores(agent);
            decompose(spores, agent.px, agent.py, spore_count, &mut food_rng);
            let cause = if bite_damage.is_some_and(|d| d > 0.0) {
                DeathCause::Hunted
            } else {
                DeathCause::Starved
            };
            events.push(OceanEvent::Died {
                id: agent.id,
                latin_name: agent.phenotype.latin_name.clone(),
                cause,
            });
            false
        });

        // Children join the population (simulated from the next tick) before the census, so a mutated child's
        // species is recorded when it has a living member (no phantom species, RV-B01); the UI gets them once
        for (child, parent_name) in births {
            if let Some(parent_name) = parent_name {
                self.ledger.discover(&child, Some(&parent_name), &mut out.species);
            }
            out.born.push(child.clone());
            self.creatures.push(child);
        }

        let extinct_from = out.species.len();
        self.ledger.census(&self.creatures, &mut out.species);
        for change in &out.species[extinct_from..] {
            if let SpeciesChange::Extinct { latin_name, .. } = change {
                out.events.push(OceanEvent::Extinct {
                    latin_name: latin_name.clone(),
                });
            }
        }
        out
    }

    /// Refills the population to `ocean.targetPopulation` with random catalogue clones, or structured random founders
    /// with an empty catalogue.
    pub fn restock(&mut self, catalogue: &[CatalogueFounder], out: &mut OceanStep) {
        let mut rng = derive(self.seed, &[STREAM_RESTOCK, self.tick]);
        while self.creatures.len() < self.config.ocean.target_population {
            let (founder, catalogue_name) = pick_founder(catalogue, &mut rng);
            let id = self.next_creature_id;
            self.add_founder(founder, &mut rng, out);
            out.born.push(self.creatures.last().expect("founder added").clone());
            out.events.push(OceanEvent::Restocked { id, catalogue_name });
        }
    }

    /// Spawns a founder with the next id, records its species and adds it to the population.
    fn add_founder(&mut self, founder: Founder, rng: &mut SimRng, out: &mut OceanStep) {
        let (width, height) = self.size();
        let agent = OceanSpawner {
            config: &self.config,
            width,
            height,
        }
        .founder(founder, self.next_creature_id, rng);
        self.next_creature_id += 1;
        self.ledger.discover(&agent, None, &mut out.species);
        self.creatures.push(agent);
    }

    fn reseed_spores(&mut self, rng: &mut SimRng) {
        let (width, height) = self.size();
        self.spores.reseed(
            self.config.ocean.food_spore_count,
            self.config.metabolism.food_spore_energy,
            rng,
            width,
            height,
        );
        self.sync_settings();
    }

    /// The world settings carry the nutrient centres of the respawn policy.
    fn sync_settings(&mut self) {
        self.settings = WorldSettings::ocean(&self.config.ocean, self.spores.nutrient_centers.clone());
    }

    pub fn size(&self) -> (f32, f32) {
        (self.config.ocean.world_width, self.config.ocean.world_height)
    }

    pub fn world(&self) -> &ProceduralWorld {
        &self.world
    }

    pub fn creatures(&self) -> &[CreatureAgent] {
        &self.creatures
    }

    /// Test and tool access to the population (placing creatures for a scenario).
    pub fn creatures_mut(&mut self) -> &mut Vec<CreatureAgent> {
        &mut self.creatures
    }

    pub fn spores(&self) -> &SporeField {
        &self.spores
    }

    pub fn spores_mut(&mut self) -> &mut SporeField {
        &mut self.spores
    }

    pub fn ledger(&self) -> &SpeciesLedger {
        &self.ledger
    }

    pub fn ledger_mut(&mut self) -> &mut SpeciesLedger {
        &mut self.ledger
    }

    pub fn highest_generation(&self) -> u32 {
        self.highest_generation
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

/// A random catalogue specimen (with its name), or a structured random founder for an empty catalogue.
fn pick_founder<'a>(catalogue: &'a [CatalogueFounder], rng: &mut SimRng) -> (Founder<'a>, Option<String>) {
    if catalogue.is_empty() {
        return (Founder::Random, None);
    }
    let entry = &catalogue[rng.gen_range(0..catalogue.len())];
    (Founder::catalogue(entry), Some(entry.name.clone()))
}

#[cfg(test)]
mod tests;
