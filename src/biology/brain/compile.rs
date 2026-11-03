//! Brain compiler (Brain v2, TCK-179/182/144/153): genes and organs -> `BrainTopology`.

use super::{
    Activation, BrainTopology, CTRNNNeuron, CTRNNSynapse, Motor, NeuronRole, NeuronType, SYSTEMIC_BASE_INPUTS_COUNT,
    SystemicInput, TAU_MAX, TAU_MIN, ThalamusKind,
};
use crate::biology::genome::alphabet::char_to_value;
use crate::biology::genome::payload::{Payload, get_payload_linear_value_offset};
use crate::biology::genome::promoters::{self, GeneRole};
use crate::biology::genome::table::{GeneRef, GeneTable};
use crate::biology::organs::{self, CHANNELS_PER_ORGANELLE, Organ, OrganGene};
use crate::shared::brain::BrainConfig;

/// Hidden neurons: at most this many `NEU` genes are expressed.
const MAX_HIDDEN_NEURONS: usize = 12;
/// Explicit `SY` genes: at most this many are expressed.
const MAX_EXPLICIT_SYNAPSES: usize = 30;
/// Synapse keys keep 53 bits so they stay exact as JavaScript numbers.
const KEY_MASK: u64 = (1 << 53) - 1;

/// A hidden neuron decoded from its `NEU` gene.
struct HiddenGene<'a> {
    gene: GeneRef,
    owner: usize,
    payload: &'a str,
    bias: f32,
    tau: f32,
    activation: Activation,
    depth: f32,
}

pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Stable synapse identity from the source and target roles (independent of neuron indices and genome positions).
fn synapse_key(src: &NeuronRole, dst: &NeuronRole) -> u64 {
    let mut bytes = Vec::with_capacity(32);
    src.encode(false, &mut bytes);
    bytes.push(0xFF);
    dst.encode(false, &mut bytes);
    fnv1a(&bytes) & KEY_MASK
}

/// Initial weight of a synapse owned by `owner`: one entry of the owner's payload tape, letter A..Z mapped to [-0.5, 0.5].
/// The entry is picked by hashing both roles with the owner's own gene identity left out, so it depends neither on
/// the owner's identity nor on other genes. `None` for an empty payload (the synapse is not wired).
fn tape_weight(tape: &str, owner: &GeneRef, src: &NeuronRole, dst: &NeuronRole) -> Option<f32> {
    if tape.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity(32);
    src.encode(src.gene() == Some(owner), &mut bytes);
    bytes.push(0xFF);
    dst.encode(dst.gene() == Some(owner), &mut bytes);
    let slot = (fnv1a(&bytes) % tape.len() as u64) as usize;
    Some(char_to_value(tape.as_bytes()[slot] as char) as f32 / 25.0 - 0.5)
}

/// Brain v2 (TCK-179/182/144/153). Neuron order Inputs (sensors, systemic) → Thalamus → Hidden (by depth) → Motors,
/// so one ascending pass integrates in evaluation order (sensor → motor latency 1 tick). Targeted wiring of live
/// structure only; every synapse has a role key and exactly one owner gene:
/// - sensor → thalamus (fixed ±1) and thalamus → motor: the organ gene;
/// - thalamus/systemic → hidden and hidden → motor: the hidden neuron's `NEU` gene;
/// - systemic → motor: the `OUT` brain-base gene (not wired without one);
/// - explicit `SY` genes override or add one synapse each (first `SY` gene per pair wins).
pub(crate) fn compile_brain(
    genes: &GeneTable,
    organelles: &[Organ],
    organ_genes: &[OrganGene],
    brain_config: &BrainConfig,
) -> BrainTopology {
    let input_count = organelles.len() * CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT;
    let neuron = |id: usize,
                  neuron_type: NeuronType,
                  role: NeuronRole,
                  tau: f32,
                  bias: f32,
                  activation: Option<Activation>,
                  x: f32,
                  y: f32| CTRNNNeuron {
        id,
        neuron_type,
        role,
        tau,
        bias,
        activation_type: activation,
        x: Some(x),
        y: Some(y),
    };
    let input_y = |id: usize| (id as f32 + 1.0) / (input_count as f32 + 1.0);
    let mut neurons: Vec<CTRNNNeuron> = Vec::new();

    // A. Sensors: CHANNELS_PER_ORGANELLE input slots per organ in organ order (layout of `compute_sensory_inputs`)
    for og in organ_genes {
        for patch in &organelles[og.organs.clone()] {
            for channel in 0..CHANNELS_PER_ORGANELLE {
                let id = neurons.len();
                let role = NeuronRole::Sensor {
                    gene: og.gene.clone(),
                    side: patch.side,
                    channel,
                    kind: patch.kind,
                };
                neurons.push(neuron(id, NeuronType::Input, role, 1.0, 0.0, None, 0.0, input_y(id)));
            }
        }
    }

    // B. Systemic inputs
    for input in SystemicInput::ALL {
        let id = neurons.len();
        neurons.push(neuron(
            id,
            NeuronType::Input,
            NeuronRole::Systemic { input },
            1.0,
            0.0,
            None,
            0.0,
            input_y(id),
        ));
    }
    let systemic: Vec<usize> = (input_count - SYSTEMIC_BASE_INPUTS_COUNT..input_count).collect();

    // C. Thalamus relays (tau TAU_MIN: near pass-through): per organ gene and live channel a Sum node, plus a Diff node for HOX pairs
    let paired = |og: &OrganGene| og.organs.len() == 2;
    let live: Vec<[bool; CHANNELS_PER_ORGANELLE]> = organ_genes
        .iter()
        .map(|og| organs::live_channels(&organelles[og.organs.start]))
        .collect();
    let thalamus_count: usize = organ_genes
        .iter()
        .zip(&live)
        .map(|(og, l)| l.iter().filter(|&&x| x).count() * if paired(og) { 2 } else { 1 })
        .sum();
    let mut thalamus: Vec<(usize, usize, ThalamusKind, usize)> = Vec::new(); // (organ gene, channel, node, neuron id)
    for (g, og) in organ_genes.iter().enumerate() {
        let nodes: &[ThalamusKind] = if paired(og) {
            &[ThalamusKind::Sum, ThalamusKind::Diff]
        } else {
            &[ThalamusKind::Sum]
        };
        for &node in nodes {
            for channel in (0..CHANNELS_PER_ORGANELLE).filter(|&c| live[g][c]) {
                let id = neurons.len();
                let activation = match node {
                    ThalamusKind::Sum => brain_config.thalamus_sum_activation.activation(),
                    ThalamusKind::Diff => Activation::Tanh,
                };
                let y = (thalamus.len() + 1) as f32 / (thalamus_count as f32 + 1.0);
                let role = NeuronRole::Thalamus {
                    gene: og.gene.clone(),
                    channel,
                    node,
                };
                neurons.push(neuron(
                    id,
                    NeuronType::Hidden,
                    role,
                    TAU_MIN,
                    0.0,
                    Some(activation),
                    0.20,
                    y,
                ));
                thalamus.push((g, channel, node, id));
            }
        }
    }
    let thalamus_ids: Vec<usize> = thalamus.iter().map(|t| t.3).collect();

    // D. Hidden neurons: one per NEU gene (first MAX_HIDDEN_NEURONS); lower depth integrates first
    let hidden: Vec<HiddenGene> = genes
        .genes(promoters::motif_of(GeneRole::Hidden))
        .iter()
        .take(MAX_HIDDEN_NEURONS)
        .map(|gene| {
            let p = gene.payload;
            let owner = genes.owner(promoters::motif_of(GeneRole::Hidden), gene.start);
            HiddenGene {
                gene: genes.gene_ref(owner),
                owner,
                payload: p,
                bias: get_payload_linear_value_offset(p, 1) * 2.0 - 1.0,
                tau: (0.2 + get_payload_linear_value_offset(p, 4) * 1.8).clamp(TAU_MIN, TAU_MAX),
                activation: match (get_payload_linear_value_offset(p, 7) * 100.0) as usize % 4 {
                    1 => Activation::Relu,
                    2 => Activation::Sigmoid,
                    3 => Activation::Sin,
                    _ => Activation::Tanh,
                },
                depth: 0.55 + get_payload_linear_value_offset(p, 10) * 0.30,
            }
        })
        .collect();
    let mut by_depth: Vec<usize> = (0..hidden.len()).collect();
    by_depth.sort_by(|&a, &b| hidden[a].depth.total_cmp(&hidden[b].depth).then(a.cmp(&b)));
    let mut hidden_ids = vec![0; hidden.len()]; // by NEU gene order
    for h in by_depth {
        let hg = &hidden[h];
        let id = neurons.len();
        hidden_ids[h] = id;
        let role = NeuronRole::Hidden { gene: hg.gene.clone() };
        neurons.push(neuron(
            id,
            NeuronType::Hidden,
            role,
            hg.tau,
            hg.bias,
            Some(hg.activation),
            0.5,
            hg.depth,
        ));
    }

    // E. Motors: bias and tau from the OUT brain-base gene (without one: empty payload, bias 0, tau 1.1)
    let out_gene = genes.genes(promoters::motif_of(GeneRole::BrainBase)).first();
    let out_payload = out_gene.map(|g| g.payload).unwrap_or("");
    let mut motor_ids = Vec::new();
    for (i, motor) in Motor::ALL.into_iter().enumerate() {
        let bias = get_payload_linear_value_offset(out_payload, i + 1) * 2.0 - 1.0;
        let tau = (0.2 + get_payload_linear_value_offset(out_payload, i + 10) * 1.8).clamp(TAU_MIN, TAU_MAX);
        let id = neurons.len();
        motor_ids.push(id);
        neurons.push(neuron(
            id,
            NeuronType::Output,
            NeuronRole::Motor { motor },
            tau,
            bias,
            None,
            1.0,
            (i + 1) as f32 / 3.0,
        ));
    }

    // Synapses
    let mut synapses: Vec<CTRNNSynapse> = Vec::new();
    let wire = |synapses: &mut Vec<CTRNNSynapse>, from: usize, to: usize, weight: f32, owner: usize| {
        let (source, target) = (&neurons[from].role, &neurons[to].role);
        let key = synapse_key(source, target);
        // Sensor → thalamus reflexes (fixed ±1 sum/difference relays) do not learn (TCK-152)
        let plastic = !matches!(
            (source, target),
            (NeuronRole::Sensor { .. }, NeuronRole::Thalamus { .. })
        );
        synapses.push(CTRNNSynapse {
            from_node: from,
            to_node: to,
            weight,
            key,
            owner,
            plastic,
        });
    };
    let tape = |payload: &str, owner: &GeneRef, from: usize, to: usize| {
        tape_weight(payload, owner, &neurons[from].role, &neurons[to].role)
    };

    // 1. Reflex sensor → thalamus (fixed ±1, organ gene); a pair shares one weight set through its Sum/Diff nodes
    for &(g, channel, node, id) in &thalamus {
        let og = &organ_genes[g];
        let sensor = |o: usize| o * CHANNELS_PER_ORGANELLE + channel;
        let first = og.organs.start;
        match (node, paired(og)) {
            (ThalamusKind::Sum, false) => wire(&mut synapses, sensor(first), id, 1.0, og.owner),
            (ThalamusKind::Sum, true) => {
                wire(&mut synapses, sensor(first), id, 1.0, og.owner);
                wire(&mut synapses, sensor(first + 1), id, 1.0, og.owner);
            }
            (ThalamusKind::Diff, _) => {
                wire(&mut synapses, sensor(first), id, 1.0, og.owner);
                wire(&mut synapses, sensor(first + 1), id, -1.0, og.owner);
            }
        }
    }

    // 2. Thalamus → motors (organ gene tape)
    for &(g, _, _, id) in &thalamus {
        let og = &organ_genes[g];
        for &m in &motor_ids {
            if let Some(w) = tape(og.payload, &og.gene, id, m) {
                wire(&mut synapses, id, m, w, og.owner);
            }
        }
    }

    // 3. Into and out of hidden neurons (NEU gene tape): thalamus → hidden, systemic → hidden, hidden → motors
    for (hg, &hid) in hidden.iter().zip(&hidden_ids) {
        for &src in thalamus_ids.iter().chain(&systemic) {
            if let Some(w) = tape(hg.payload, &hg.gene, src, hid) {
                wire(&mut synapses, src, hid, w, hg.owner);
            }
        }
        for &m in &motor_ids {
            if let Some(w) = tape(hg.payload, &hg.gene, hid, m) {
                wire(&mut synapses, hid, m, w, hg.owner);
            }
        }
    }

    // 4. Systemic → motors (OUT gene tape)
    if let Some(out) = out_gene {
        let owner = genes.owner(promoters::motif_of(GeneRole::BrainBase), out.start);
        let out_ref = genes.gene_ref(owner);
        for &s in &systemic {
            for &m in &motor_ids {
                if let Some(w) = tape(out.payload, &out_ref, s, m) {
                    wire(&mut synapses, s, m, w, owner);
                }
            }
        }
    }

    // 5. Explicit SY genes address roles: (source class, index, target class, index), index modulo the class size;
    //    weight in [-2, 2]. Sources: systemic, thalamus, hidden; targets: hidden, motors (hidden in NEU gene order).
    let sources: [&[usize]; 3] = [&systemic, &thalamus_ids, &hidden_ids];
    let targets: [&[usize]; 2] = [&hidden_ids, &motor_ids];
    let mut explicit = vec![false; synapses.len()];
    for gene in genes
        .genes(promoters::motif_of(GeneRole::Synapse))
        .iter()
        .take(MAX_EXPLICIT_SYNAPSES)
    {
        let p = gene.payload;
        let (src_class, dst_class) = (sources[Payload(p).index(1) % 3], targets[Payload(p).index(3) % 2]);
        if src_class.is_empty() || dst_class.is_empty() {
            continue;
        }
        let from = src_class[Payload(p).index(2) % src_class.len()];
        let to = dst_class[Payload(p).index(4) % dst_class.len()];
        if from == to {
            continue;
        }
        let weight = get_payload_linear_value_offset(p, 7) * 4.0 - 2.0;
        let owner = genes.owner(promoters::motif_of(GeneRole::Synapse), gene.start);
        match synapses.iter().position(|s| s.from_node == from && s.to_node == to) {
            Some(i) if !explicit[i] => {
                synapses[i].weight = weight;
                synapses[i].owner = owner;
                explicit[i] = true;
            }
            Some(_) => {} // the first SY gene of a pair wins
            None => {
                wire(&mut synapses, from, to, weight, owner);
                explicit.push(true);
            }
        }
    }

    // Sorted by target for the single linear pass in execute_brain_with_learning
    synapses.sort_by_key(|s| s.to_node);
    BrainTopology { neurons, synapses }
}
