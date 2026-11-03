//! Reproduction system (asexual mitosis, TCK-152): a mature, fed creature above its stomach threshold splits; the
//! child is a fresh agent at the parent's place that joins the population (`sim::ocean::OceanSim::step`, TCK-143).

use rand::Rng;

use crate::biology::creature_factory::{SpawnOptions, inherit_learned_weights, spawn_from_phenotype};
use crate::biology::genome::{GenomeConfig, mutate_genome};
use crate::biology::metabolism::MetabolismConfig;
use crate::biology::phenotype::parse_genome;
use crate::config::OceanConfig;
use crate::shared::brain::BrainConfig;
use crate::shared::dto::CreatureAgent;
use crate::shared::fmath;

/// Probability that a mitosis child's genome mutates.
pub const MITOSIS_MUTATION_CHANCE: f64 = 0.35;

/// Mitosis rules (config section `ocean`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MitosisRule {
    pub mature_age_floor: u32,
    pub stomach_threshold_floor: f32,
    pub max_creatures: usize,
    pub split_loss_ratio: f32,
    pub recoil_velocity_scale: f32,
    /// Parents younger than this pass no learned weights
    pub lamarckian_age_floor: u32,
}

impl MitosisRule {
    pub fn from_ocean(ocean: &OceanConfig) -> Self {
        Self {
            mature_age_floor: ocean.creature_mature_age_floor,
            stomach_threshold_floor: ocean.reproduction_stomach_threshold_floor,
            max_creatures: ocean.max_creatures,
            split_loss_ratio: ocean.reproduction_split_loss_ratio,
            recoil_velocity_scale: ocean.reproduction_recoil_velocity_scale,
            lamarckian_age_floor: ocean.lamarckian_assimilation_age_floor,
        }
    }
}

/// Whether `agent` splits this tick: mature (`max(mature_age, floor)`), fed since its last split, room below
/// `max_creatures` and energy at least `stomach_capacity · max(repro_threshold, floor)`.
pub fn ready(agent: &CreatureAgent, rule: &MitosisRule, population: usize) -> bool {
    let mature_age = agent.phenotype.mature_age.max(rule.mature_age_floor);
    let threshold =
        agent.phenotype.stomach_capacity * agent.phenotype.repro_threshold.max(rule.stomach_threshold_floor);
    agent.age >= mature_age && agent.has_eaten && population < rule.max_creatures && agent.energy >= threshold
}

/// The parent's side of the split: energy times `split_loss_ratio`, hunger flag reset, recoil against the heading.
pub fn split(agent: &mut CreatureAgent, rule: &MitosisRule) {
    agent.energy *= rule.split_loss_ratio;
    agent.has_eaten = false;
    let recoil = agent.phenotype.split_loss * rule.recoil_velocity_scale;
    agent.vx += recoil * fmath::cos(agent.heading_angle + std::f32::consts::PI);
    agent.vy += recoil * fmath::sin(agent.heading_angle + std::f32::consts::PI);
}

/// The child of a split parent (call after `split`): with `MITOSIS_MUTATION_CHANCE` a mutated genome, Lamarckian
/// learned weights from parents at least `lamarckian_age_floor` old, the parent's post-split energy and the parent's
/// recoil along the heading (parent and child drift apart). Returns the child
/// and whether its genome mutated. Draws the mutation roll, the mutation and the inheritance from `rng` in this order.
pub fn child<R: Rng + ?Sized>(
    parent: &CreatureAgent,
    id: u32,
    rule: &MitosisRule,
    genome: &GenomeConfig,
    brain: &BrainConfig,
    metabolism: &MetabolismConfig,
    rng: &mut R,
) -> (CreatureAgent, bool) {
    let is_mutation = rng.gen_range(0.0..1.0) < MITOSIS_MUTATION_CHANCE;
    let (child_genome, child_phenotype) = if is_mutation {
        let g = mutate_genome(&parent.genome, &genome.mutation_operators, rng)
            .map_or_else(|| parent.genome.clone(), |m| m.genome);
        let p = parse_genome(&g, None, None, brain);
        (g, p)
    } else {
        (parent.genome.clone(), parent.phenotype.clone())
    };
    let assimilate = parent.age >= rule.lamarckian_age_floor;
    let (parent_synapses, parent_weights): (&[_], &[_]) = if assimilate {
        (&parent.phenotype.brain.synapses, &parent.synapse_weights)
    } else {
        (&[], &[])
    };
    let weights = inherit_learned_weights(
        parent_synapses,
        parent_weights,
        &child_phenotype.brain.synapses,
        genome.lamarckian_assimilation_chance,
        rng,
    );
    let mut child = spawn_from_phenotype(
        &child_genome,
        child_phenotype,
        SpawnOptions {
            id,
            px: parent.px,
            py: parent.py,
            heading_angle: parent.heading_angle,
            generation: parent.generation + 1,
            methylations: None,
            synapse_weights: Some(weights),
        },
        metabolism,
    );
    child.energy = parent.energy;
    let recoil = parent.phenotype.split_loss * rule.recoil_velocity_scale;
    child.vx = recoil * fmath::cos(parent.heading_angle);
    child.vy = recoil * fmath::sin(parent.heading_angle);
    (child, is_mutation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::creature_factory::{SpawnOptions, spawn_creature};
    use crate::biology::genome::WILDTYPE_GENOME;
    use crate::shared::rng::seeded;

    #[test]
    fn test_split_parent_and_child_share_the_post_split_energy() {
        let config = crate::config::test_config();
        let rule = MitosisRule::from_ocean(&config.ocean);
        let mut parent = spawn_creature(
            WILDTYPE_GENOME,
            SpawnOptions {
                id: 1,
                px: 10.0,
                py: 20.0,
                heading_angle: 0.0,
                generation: 3,
                methylations: None,
                synapse_weights: None,
            },
            config,
        );
        assert!(!ready(&parent, &rule, 1), "newborn is not mature and not fed");
        parent.age = parent.phenotype.mature_age.max(rule.mature_age_floor);
        parent.has_eaten = true;
        parent.energy = parent.phenotype.stomach_capacity;
        assert!(ready(&parent, &rule, 1));
        assert!(!ready(&parent, &rule, rule.max_creatures), "population cap");
        let before = parent.energy;
        split(&mut parent, &rule);
        assert_eq!(parent.energy, before * rule.split_loss_ratio);
        assert!(!parent.has_eaten);
        let (child, _) = child(
            &parent,
            2,
            &rule,
            &config.genome,
            &config.brain,
            &config.metabolism,
            &mut seeded(4),
        );
        assert_eq!(
            (child.id, child.generation, child.energy, child.px, child.py),
            (2, 4, parent.energy, 10.0, 20.0)
        );
    }
}
