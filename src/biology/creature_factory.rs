use rand::Rng;

use crate::biology::dna::{parse_genome, CTRNNSynapse, CreaturePhenotype};
use crate::shared::brain::{BrainRuntime, WEIGHT_CLAMP};
use crate::shared::types::{AppConfig, CreatureAgent};

/// What differs between spawn sites (Ocean seed/inject/clone/reset/restock, trainer sandbox).
/// Everything else about a fresh agent is defined once in `spawn_from_phenotype`.
pub struct SpawnOptions<'a> {
    pub id: u32,
    pub px: f32,
    pub py: f32,
    pub heading_angle: f32,
    pub generation: u32,
    /// Parent methylations passed to `parse_genome` (only used by `spawn_creature`).
    pub methylations: Option<&'a [f32]>,
    /// Learned weights; ignored unless the length matches the genome's synapse count.
    pub synapse_weights: Option<Vec<f32>>,
}

/// Lamarckian inheritance by synapse key (TCK-179), as a probability (TCK-152): each plastic child synapse whose key
/// matches a plastic parent synapse takes, with probability `chance`, the parent's full learned offset
/// (child genome weight + parent learned − parent genome weight, clamped to the learning clamp); otherwise, and for
/// child synapses without a homologue or fixed ones, it keeps its genome weight. `parent_weights` are the parent's
/// learned weights in `parent_synapses` order; on a length mismatch nothing is inherited. With `chance` 1 a child with
/// the parent's genome gets exactly the parent's learned weights. One draw from `rng` per matched synapse.
pub fn inherit_learned_weights<R: Rng + ?Sized>(
    parent_synapses: &[CTRNNSynapse],
    parent_weights: &[f32],
    child_synapses: &[CTRNNSynapse],
    chance: f32,
    rng: &mut R,
) -> Vec<f32> {
    let learned: std::collections::HashMap<u64, (f32, f32)> = if parent_weights.len() == parent_synapses.len() {
        parent_synapses.iter().zip(parent_weights).filter(|(s, _)| s.plastic).map(|(s, &w)| (s.key, (s.weight, w))).collect()
    } else {
        Default::default()
    };
    child_synapses
        .iter()
        .map(|c| match learned.get(&c.key) {
            Some(&(genome_weight, learned_weight)) if c.plastic && rng.gen_range(0.0f32..1.0) < chance => {
                (learned_weight + (c.weight - genome_weight)).clamp(-WEIGHT_CLAMP, WEIGHT_CLAMP)
            }
            _ => c.weight,
        })
        .collect()
}

/// Start energy of a fresh agent: a fixed share of its stomach capacity (config `creatureStartStomachRatio`).
pub fn start_energy(phenotype: &CreaturePhenotype) -> f32 {
    phenotype.stomach_capacity * AppConfig::global().rules.creature_start_stomach_ratio
}

/// Parses the genome (with optional parent methylations) and spawns a fresh agent.
pub fn spawn_creature(genome: &str, opts: SpawnOptions) -> CreatureAgent {
    let phenotype = parse_genome(genome, None, opts.methylations);
    spawn_from_phenotype(genome, phenotype, opts)
}

/// Spawns a fresh agent from an already parsed phenotype (`opts.methylations` is not re-applied).
pub fn spawn_from_phenotype(genome: &str, phenotype: CreaturePhenotype, opts: SpawnOptions) -> CreatureAgent {
    let num_neurons = phenotype.brain.neurons.len();
    let synapse_weights = match opts.synapse_weights {
        Some(learned) if learned.len() == phenotype.brain.synapses.len() => learned,
        _ => phenotype.brain.synapses.iter().map(|s| s.weight).collect(),
    };

    CreatureAgent {
        id: opts.id,
        species_id: genome.to_string(),
        px: opts.px,
        py: opts.py,
        vx: 0.0,
        vy: 0.0,
        heading_angle: opts.heading_angle,
        bend_angle: 0.0,
        omega_rot: 0.0,
        energy: start_energy(&phenotype),
        age: 0,
        generation: opts.generation,
        has_eaten: false,
        genome: genome.to_string(),
        antisense: phenotype.antisense_strand.clone(),
        phenotype,
        neuron_states: vec![0.0; num_neurons],
        neuron_activations: vec![0.0; num_neurons],
        synapse_weights,
        brain_runtime: BrainRuntime::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_DNA: &str = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";

    fn opts(id: u32, px: f32, py: f32, heading_angle: f32, generation: u32) -> SpawnOptions<'static> {
        SpawnOptions { id, px, py, heading_angle, generation, methylations: None, synapse_weights: None }
    }

    #[test]
    fn test_trainer_and_ocean_spawn_identical_except_placement() {
        // Trainer: chamber centre, generation counter; Ocean: random position, generation 1
        let trainer = spawn_creature(BASE_DNA, opts(3, 500.0, 500.0, 1.2, 7));
        let ocean = spawn_creature(BASE_DNA, opts(42, 8123.0, 4210.0, 4.4, 1));

        let normalize = |agent: &CreatureAgent| {
            let mut value = serde_json::to_value(agent).unwrap();
            for key in ["id", "px", "py", "headingAngle", "generation"] {
                value.as_object_mut().unwrap().remove(key);
            }
            value
        };
        assert_eq!(normalize(&trainer), normalize(&ocean));

        let num_neurons = trainer.phenotype.brain.neurons.len();
        assert_eq!(trainer.energy, trainer.phenotype.stomach_capacity * AppConfig::global().rules.creature_start_stomach_ratio);
        assert_eq!(trainer.antisense, trainer.phenotype.antisense_strand);
        assert_eq!(trainer.neuron_states, vec![0.0; num_neurons]);
        assert_eq!(trainer.neuron_activations, vec![0.0; num_neurons]);
    }

    #[test]
    fn test_mismatching_synapse_weights_fall_back_to_genome() {
        let genome_weights: Vec<f32> = parse_genome(BASE_DNA, None, None).brain.synapses.iter().map(|s| s.weight).collect();
        assert!(!genome_weights.is_empty());

        let mut wrong_len = opts(1, 0.0, 0.0, 0.0, 1);
        wrong_len.synapse_weights = Some(vec![0.5; genome_weights.len() + 1]);
        assert_eq!(spawn_creature(BASE_DNA, wrong_len).synapse_weights, genome_weights);

        let learned = vec![0.25; genome_weights.len()];
        let mut matching = opts(1, 0.0, 0.0, 0.0, 1);
        matching.synapse_weights = Some(learned.clone());
        assert_eq!(spawn_creature(BASE_DNA, matching).synapse_weights, learned);
    }

    /// Structured genome, its synapses and learned weights = genome weight + a distinct non-zero offset per synapse.
    fn learned_parent(seed: u64) -> (Vec<CTRNNSynapse>, Vec<f32>) {
        let genome = crate::biology::dna::generate_structured_genome(&AppConfig::global().genome_composition, &mut crate::shared::rng::seeded(seed));
        let synapses = parse_genome(&genome, None, None).brain.synapses;
        let learned = synapses.iter().enumerate().map(|(i, s)| s.weight + 0.01 * (i + 1) as f32).collect();
        (synapses, learned)
    }

    #[test]
    fn test_lamarck_chance_is_a_probability_per_synapse() {
        let mut rng = crate::shared::rng::seeded(152);
        let (mut plastic, mut taken) = (0usize, 0usize);
        for seed in 0..60 {
            let (synapses, learned) = learned_parent(seed);
            let genome: Vec<f32> = synapses.iter().map(|s| s.weight).collect();
            assert!(synapses.iter().any(|s| !s.plastic) && synapses.iter().any(|s| s.plastic));

            // Chance 1: the learned parent on every plastic synapse; fixed reflexes keep their genome weight
            let all = inherit_learned_weights(&synapses, &learned, &synapses, 1.0, &mut rng);
            for ((s, &w), (&l, &g)) in synapses.iter().zip(&all).zip(learned.iter().zip(&genome)) {
                assert_eq!(w, if s.plastic { l } else { g }, "{:?}", s);
            }
            // Chance 0: the genome weights
            assert_eq!(inherit_learned_weights(&synapses, &learned, &synapses, 0.0, &mut rng), genome);

            // Chance 0.25: each plastic synapse takes the full learned weight or its genome weight
            let some = inherit_learned_weights(&synapses, &learned, &synapses, 0.25, &mut rng);
            for ((s, &w), (&l, &g)) in synapses.iter().zip(&some).zip(learned.iter().zip(&genome)) {
                assert!(w == g || (s.plastic && w == l), "{:?}: {}", s, w);
                plastic += s.plastic as usize;
                taken += (w != g) as usize;
            }
        }
        // Binomial: n > 3000 plastic synapses, sd of the share < 0.008
        let share = taken as f32 / plastic as f32;
        assert!(plastic > 3000, "{} plastic synapses", plastic);
        assert!((share - 0.25).abs() < 0.03, "inherited share {}", share);
    }

    #[test]
    fn test_lamarck_draws_are_seeded() {
        let (synapses, learned) = learned_parent(7);
        let draw = |seed| inherit_learned_weights(&synapses, &learned, &synapses, 0.5, &mut crate::shared::rng::seeded(seed));
        assert_eq!(draw(1), draw(1));
        assert_ne!(draw(1), draw(2));
    }
}
