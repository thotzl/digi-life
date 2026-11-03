//! `OceanSpawner` (TCK-156): the one founder spawn of the Ocean (fresh start, `INJECT_URZELLE`, `SPAWN_CLONE`,
//! `RESET_EVOLUTION`, restock). `engine.rs` held six copies of it; mitosis children come from `systems::reproduce`.

use rand::Rng;

use crate::biology::creature_factory::{SpawnOptions, spawn_from_phenotype};
use crate::biology::genome::{WILDTYPE_GENOME, generate_structured_genome, mutate_genome};
use crate::biology::phenotype::parse_genome;
use crate::config::AppConfig;
use crate::shared::dto::CreatureAgent;

/// Probability that a fresh-start founder carries a mutated wildtype genome.
pub const WILDTYPE_MUTATION_CHANCE: f64 = 0.60;
/// Distance of a founder's spawn point from the world edge (px).
pub const SPAWN_MARGIN: f32 = 500.0;
/// Start velocity bound of a founder (px/tick, each axis).
pub const SPAWN_DRIFT: f32 = 0.4;

/// A catalogue specimen as the Ocean uses it (mapped from the store's catalogue rows by the engine).
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogueFounder {
    pub name: String,
    pub genome: String,
    pub methylations: Vec<f32>,
    pub synapse_weights: Vec<f32>,
}

/// Where a founder's genome comes from.
#[derive(Debug, Clone, PartialEq)]
pub enum Founder<'a> {
    /// A structured random genome (`INJECT_URZELLE`, empty-catalogue reset and restock)
    Random,
    /// The wildtype genome, mutated with `WILDTYPE_MUTATION_CHANCE` (fresh start)
    Wildtype,
    /// A given genome with optional parent methylations and learned weights (catalogue clone, `SPAWN_CLONE`); the
    /// factory falls back to the genome weights on a length mismatch
    Genome {
        genome: &'a str,
        methylations: Option<&'a [f32]>,
        synapse_weights: Option<Vec<f32>>,
    },
}

impl<'a> Founder<'a> {
    /// A clone of a catalogue specimen with its methylations and learned weights.
    pub fn catalogue(entry: &'a CatalogueFounder) -> Self {
        let methylations = if entry.methylations.is_empty() {
            None
        } else {
            Some(&entry.methylations[..])
        };
        Founder::Genome {
            genome: &entry.genome,
            methylations,
            synapse_weights: Some(entry.synapse_weights.clone()),
        }
    }
}

/// Spawns founders (generation 1) uniformly `SPAWN_MARGIN` inside a world of `width × height`.
pub struct OceanSpawner<'a> {
    pub config: &'a AppConfig,
    pub width: f32,
    pub height: f32,
}

impl<'a> OceanSpawner<'a> {
    /// A fresh founder with id `id`. Draws from `rng` in this order: the genome (random or wildtype mutation), the
    /// position, the heading, the start velocity.
    pub fn founder<R: Rng + ?Sized>(&self, founder: Founder, id: u32, rng: &mut R) -> CreatureAgent {
        let (genome, methylations, synapse_weights) = match founder {
            Founder::Random => (
                generate_structured_genome(&self.config.genome.genome_composition, rng),
                None,
                None,
            ),
            Founder::Wildtype => {
                let mut genome = WILDTYPE_GENOME.to_string();
                if rng.gen_range(0.0..1.0) < WILDTYPE_MUTATION_CHANCE {
                    if let Some(mutation) = mutate_genome(WILDTYPE_GENOME, &self.config.genome.mutation_operators, rng)
                    {
                        genome = mutation.genome;
                    }
                }
                (genome, None, None)
            }
            Founder::Genome {
                genome,
                methylations,
                synapse_weights,
            } => (genome.to_string(), methylations, synapse_weights),
        };
        let phenotype = parse_genome(&genome, None, methylations, &self.config.brain);
        let px = rng.gen_range(SPAWN_MARGIN..self.width - SPAWN_MARGIN);
        let py = rng.gen_range(SPAWN_MARGIN..self.height - SPAWN_MARGIN);
        let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let mut agent = spawn_from_phenotype(
            &genome,
            phenotype,
            SpawnOptions {
                id,
                px,
                py,
                heading_angle,
                generation: 1,
                methylations,
                synapse_weights,
            },
            &self.config.metabolism,
        );
        agent.vx = rng.gen_range(-SPAWN_DRIFT..SPAWN_DRIFT);
        agent.vy = rng.gen_range(-SPAWN_DRIFT..SPAWN_DRIFT);
        agent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::rng::seeded;

    #[test]
    fn test_founders_spawn_inside_with_their_genome_source() {
        let config = crate::config::test_config();
        let spawner = OceanSpawner {
            config,
            width: 4000.0,
            height: 3000.0,
        };
        let entry = CatalogueFounder {
            name: "Testus".into(),
            genome: WILDTYPE_GENOME.into(),
            methylations: vec![],
            synapse_weights: vec![0.5],
        };
        let clone = spawner.founder(Founder::catalogue(&entry), 7, &mut seeded(1));
        assert_eq!(
            (clone.id, clone.generation, clone.genome.as_str()),
            (7, 1, WILDTYPE_GENOME)
        );
        assert!((500.0..3500.0).contains(&clone.px) && (500.0..2500.0).contains(&clone.py));
        assert!(clone.vx.abs() < SPAWN_DRIFT && clone.vy.abs() < SPAWN_DRIFT);
        // Same stream, same founder
        let a = spawner.founder(Founder::Random, 1, &mut seeded(9));
        let b = spawner.founder(Founder::Random, 1, &mut seeded(9));
        assert_eq!((a.genome, a.px, a.heading_angle), (b.genome, b.px, b.heading_angle));
        let wild: Vec<bool> = (0..20)
            .map(|s| spawner.founder(Founder::Wildtype, 1, &mut seeded(s)).genome == WILDTYPE_GENOME)
            .collect();
        assert!(
            wild.contains(&true) && wild.contains(&false),
            "wildtype founders mutate with a chance"
        );
    }
}
