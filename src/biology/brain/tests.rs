//! Brain compiler and brain step robustness tests on compiled genomes.

use super::*;
use crate::biology::genome::tests::ops;
use crate::biology::genome::*;
use crate::biology::organs::CHANNELS_PER_ORGANELLE;
use crate::biology::phenotype::parse_genome;
#[test]
fn test_progenitor_parse_genome() {
    // High quality test input representing a well-formed progenitor genome
    let progenitor_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";
    let phenotype = parse_genome(progenitor_dna, None, None, &crate::config::test_config().brain);

    // Core assertions
    assert!(phenotype.genome_string.starts_with(progenitor_dna));
    assert!(phenotype.spinal_harmonics.mean_radius >= 16.0 && phenotype.spinal_harmonics.mean_radius <= 28.0);
    assert!(
        crate::biology::metabolism::tick_cost(&phenotype, 0.0, 0.0, &crate::config::test_config().metabolism).resting()
            > 0.0
    );
    assert!(phenotype.stiffness >= 0.15 && phenotype.stiffness <= 1.0);
    assert!(
        phenotype
            .antisense_strand
            .starts_with(&get_complementary_string(progenitor_dna))
    );

    // Brain topology assertions
    assert!(!phenotype.brain.neurons.is_empty());
    assert!(!phenotype.brain.synapses.is_empty());

    // Assert that we have inputs, outputs and hidden neurons
    let input_count = phenotype
        .brain
        .neurons
        .iter()
        .filter(|n| n.neuron_type == NeuronType::Input)
        .count();
    let output_count = phenotype
        .brain
        .neurons
        .iter()
        .filter(|n| n.neuron_type == NeuronType::Output)
        .count();
    let hidden_count = phenotype
        .brain
        .neurons
        .iter()
        .filter(|n| n.neuron_type == NeuronType::Hidden)
        .count();

    assert_eq!(
        input_count,
        phenotype.organelles.len() * CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT
    );
    assert_eq!(output_count, OUTPUT_MOTOR_NODES_COUNT);
    assert!(hidden_count >= 1 && hidden_count <= 35); // Includes our generic thalamus sum/diff pre-processor interneurons!

    // Verify that the resting metabolic cost is finite
    assert!(
        crate::biology::metabolism::tick_cost(&phenotype, 0.0, 0.0, &crate::config::test_config().metabolism)
            .resting()
            .is_finite()
    );
    assert!(phenotype.stomach_capacity > 0.0);
}

#[test]
fn test_dynamic_neuro_generation() {
    // DNA with 3 NEU promoters and 2 SY promoters using degenerate Option A + full wobble matching
    // Ending with Z prevents S-A padding boundary matches, and YNB prevents N-A-A matches.
    let dna = "NEUAAESNEUBBESNEUCCESYBAAESYBBBESZ";
    let phenotype = parse_genome(dna, None, None, &crate::config::test_config().brain);

    let hidden_count = phenotype
        .brain
        .neurons
        .iter()
        .filter(|n| n.neuron_type == NeuronType::Hidden)
        .count();
    // Since we have 3 NEU blocks, we should have exactly 3 hidden neurons compiled dynamically!
    assert_eq!(hidden_count, 3);

    // Synapses should be dynamically compiled from "SY" payloads
    assert!(!phenotype.brain.synapses.is_empty());
}

#[test]
fn test_hebbian_learning_and_epigenetics() {
    use crate::shared::brain::execute_brain_with_learning;

    let dna = "NEUAAESNEUBBESZZSYBCAESYBBBES"; // 3 hiddens, 1 SY
    let phenotype = parse_genome(dna, None, None, &crate::config::test_config().brain);
    let brain = phenotype.brain;

    let mut states = vec![0.0; brain.neurons.len()];
    let mut activations = vec![0.0; brain.neurons.len()];
    let mut synapse_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();

    let inputs = vec![0.5, 0.5]; // 1 sensor, 1 clock

    // Step 1: Execute brain with learning active (hebbian_rate > 0.0)
    let original_weight = synapse_weights[0];
    for _ in 0..5 {
        execute_brain_with_learning(
            &brain,
            &inputs,
            &mut states,
            &mut activations,
            &mut synapse_weights,
            &mut crate::shared::brain::BrainRuntime::default(),
            0.1, // high learning rate
        );
    }

    let learned_weight = synapse_weights[0];
    // Weight should change due to Hebbian correlation!
    assert_ne!(learned_weight, original_weight);
}

#[test]
fn test_targeted_wiring_and_active_pruning() {
    use crate::shared::brain::execute_brain_with_learning;

    let dna = "NEUAAESNEUBBESZZSYBCAESYBBBES"; // 3 NEU genes (one inside an SY payload), 2 SY genes, no organ, no OUT gene
    let phenotype = parse_genome(dna, None, None, &crate::config::test_config().brain);
    let brain = &phenotype.brain;
    let role = |i: usize| &brain.neurons[i].role;
    let of_role = |pred: fn(&NeuronRole) -> bool| {
        (0..brain.neurons.len())
            .filter(|&i| pred(role(i)))
            .collect::<Vec<usize>>()
    };
    let hidden = of_role(|r| matches!(r, NeuronRole::Hidden { .. }));
    let systemic = of_role(|r| matches!(r, NeuronRole::Systemic { .. }));
    let motors = of_role(|r| matches!(r, NeuronRole::Motor { .. }));
    let is_sy = |s: &CTRNNSynapse| phenotype.genes[s.owner].gene.promoter == "SY";
    assert_eq!(hidden.len(), 3);
    assert_eq!(motors.len(), OUTPUT_MOTOR_NODES_COUNT);

    // Targeted wiring (TCK-182): every systemic → hidden and hidden → motor pair exactly once; anything else
    // (no hidden → hidden, no systemic → motor without an OUT gene) only through an SY gene
    let count = |from: usize, to: usize| {
        brain
            .synapses
            .iter()
            .filter(|s| s.from_node == from && s.to_node == to)
            .count()
    };
    for &h in &hidden {
        assert!(systemic.iter().all(|&s| count(s, h) == 1));
        assert!(motors.iter().all(|&m| count(h, m) == 1));
    }
    let regular = |s: &CTRNNSynapse| {
        matches!(
            (role(s.from_node), role(s.to_node)),
            (NeuronRole::Systemic { .. }, NeuronRole::Hidden { .. })
                | (NeuronRole::Hidden { .. }, NeuronRole::Motor { .. })
        )
    };
    assert!(brain.synapses.iter().all(|s| regular(s) || is_sy(s)));
    assert_eq!(
        brain.synapses.iter().filter(|s| regular(s)).count(),
        hidden.len() * (SYSTEMIC_BASE_INPUTS_COUNT + OUTPUT_MOTOR_NODES_COUNT)
    );
    let keys: std::collections::BTreeSet<u64> = brain.synapses.iter().map(|s| s.key).collect();
    assert_eq!(keys.len(), brain.synapses.len(), "synapse keys are unique");

    // Gene-tape weights start weak in [-0.5, 0.5] (never exactly 0); SY weights in [-2, 2]
    assert!(brain.synapses.iter().any(|s| is_sy(s)));
    for s in &brain.synapses {
        let limit = if is_sy(s) { 2.0 } else { 0.5 };
        assert!(s.weight != 0.0 && s.weight.abs() <= limit, "{:?}", s);
    }

    // Test Active Pruning
    let mut synapse_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
    let mut states = vec![0.0; brain.neurons.len()];
    let mut activations = vec![0.0; brain.neurons.len()];
    let inputs = vec![0.0, 0.0]; // Zero inputs: Oja's −η·post²·w term alone shrinks the synapses without input

    // Run 50 steps of a high learning rate to prune weak connections
    for _ in 0..50 {
        execute_brain_with_learning(
            brain,
            &inputs,
            &mut states,
            &mut activations,
            &mut synapse_weights,
            &mut crate::shared::brain::BrainRuntime::default(),
            0.5, // high learning rate to prune quickly
        );
    }

    // Weak synapses should now be exactly 0.0 (permanently pruned!)
    let mut pruned_count = 0;
    for &w in &synapse_weights {
        if w == 0.0 {
            pruned_count += 1;
        }
    }
    assert!(pruned_count > 0);
}

#[test]
fn test_mutation_robustness_and_deadlocks() {
    use crate::shared::brain::execute_brain_with_learning;

    // 1. Generate 50 heavily mutated genomes of different lengths (fuzzing sweep)
    let mut rng = crate::shared::rng::seeded(3);
    for trial in 0..50 {
        let start_len = 128 + (trial * 7) % 380; // various lengths in [128..508]
        let mut genome = generate_random_genome(start_len, &mut rng);

        // Apply 50 random mutations, insertions, or deletions consecutively to scramble promoters and terminators
        for _ in 0..50 {
            if let Some(mutation) = mutate_genome(&genome, ops(), &mut rng) {
                genome = mutation.genome;
            }
        }

        // A. Assert compilation robustness: must NEVER panic, crash or deadlock
        let phenotype = parse_genome(&genome, None, None, &crate::config::test_config().brain);
        let resting_cost =
            crate::biology::metabolism::tick_cost(&phenotype, 0.0, 0.0, &crate::config::test_config().metabolism)
                .resting();
        let brain = phenotype.brain;

        // B. Assert state boundaries are finite
        assert!(phenotype.stiffness.is_finite());
        assert!(phenotype.stiffness >= 0.15 && phenotype.stiffness <= 1.0);
        assert!(resting_cost.is_finite());
        assert!(phenotype.stomach_capacity > 0.0);

        // C. Assert brain execution robustness: run euler integration for 10 ticks and ensure zero NaNs/Infs
        let mut states = vec![0.0; brain.neurons.len()];
        let mut activations = vec![0.0; brain.neurons.len()];
        let mut synapse_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();

        let input_count = brain
            .neurons
            .iter()
            .filter(|n| n.neuron_type == NeuronType::Input)
            .count();
        let inputs = vec![0.5; input_count];

        for _tick in 0..10 {
            let outputs = execute_brain_with_learning(
                &brain,
                &inputs,
                &mut states,
                &mut activations,
                &mut synapse_weights,
                &mut crate::shared::brain::BrainRuntime::default(),
                0.05, // active Oja learning
            );

            // Assert that all outputs, neuron states, activations and synapse weights are perfectly finite and NOT NaN
            for out in outputs.values() {
                assert!(out.is_finite(), "Output was NaN or Infinite!");
            }
            for &s in &states {
                assert!(s.is_finite(), "Neuron state was NaN or Infinite!");
            }
            for &a in &activations {
                assert!(a.is_finite(), "Neuron activation was NaN or Infinite!");
            }
            for &w in &synapse_weights {
                assert!(w.is_finite(), "Synapse weight was NaN or Infinite!");
            }
        }
    }
}
