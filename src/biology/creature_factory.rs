use crate::biology::dna::{parse_genome, CTRNNSynapse, CreaturePhenotype};
use crate::shared::brain::BrainRuntime;
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

/// Lamarckian inheritance by synapse key (TCK-179): each child synapse gets its genome weight plus `scale` × the
/// learned offset (learned − genome weight) of the parent synapse with the same key, clamped to [-4, 4] like Hebbian
/// learning; child synapses without a homologue keep their genome weight. `parent_weights` are the parent's learned
/// weights in `parent_synapses` order; on a length mismatch nothing is inherited. With `scale` 1 a child with the
/// parent's genome gets exactly the parent's learned weights.
pub fn inherit_learned_weights(parent_synapses: &[CTRNNSynapse], parent_weights: &[f32], child_synapses: &[CTRNNSynapse], scale: f32) -> Vec<f32> {
    let learned: std::collections::HashMap<u64, (f32, f32)> = if parent_weights.len() == parent_synapses.len() {
        parent_synapses.iter().zip(parent_weights).map(|(s, &w)| (s.key, (s.weight, w))).collect()
    } else {
        Default::default()
    };
    child_synapses
        .iter()
        .map(|c| match learned.get(&c.key) {
            Some(&(genome_weight, learned_weight)) => (learned_weight * scale + (c.weight - genome_weight * scale)).clamp(-4.0, 4.0),
            None => c.weight,
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
}
