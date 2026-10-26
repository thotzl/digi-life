//! Energy v2 (TCK-151, TCK-178, TCK-145): one activity-based energy function for Ocean and trainer.
//!
//! `cost per tick = body + senses + brain + movement`
//! - body     = `bodyCostPerMass · body_mass` (the physics mass `r^1.5 · L / 25`)
//! - senses   = `organCostPerLoad · Σ scale² · (1.1 − bandwidth)` over the organs (shape of the former organ tax)
//! - brain    = `neuronCost · neurons + activityCost · Σ |w · a_source|`; neurons = thalamus + hidden + motors
//!   (Herculano-Houzel 2011: cost per neuron roughly constant), activity = the signal sum of the active synapses
//!   in the brain step (Attwell & Laughlin 2001: signalling dominates), so pruned and zero weights cost nothing
//! - movement = `movementCostPerMass · body_mass · |out_thrust|`
//!
//! Thermal stress stays an Ocean-only environment cost on top. The former multiplicative BMR
//! (`brain_complexity_multiplier`, RV-S03) and the Ocean `* 0.45` literal are gone.
//!
//! ## Derivation of the constants (config `metabolism`, no free tuning)
//! Inputs: `trainer_runner probe metabolism --seed tck151` over 1000 structured genomes (config `genomeComposition`),
//! medians: body mass M = 580.355, organ load O = 1.74513, neurons N = 14, activity A = 9.60623 and |thrust| T = 0.7029
//! (mean Σ|w·a| and mean |out_thrust| per tick over one standard-room trial, learning on; the "typical" brain).
//! Old Ocean drain: `basal_metabolic_rate · bmrBaseScale (0.002) · basalMetabolicRateMultiplier (1.0) · 0.45`, median
//! over the same genomes 27 · 0.0009 = D = 0.0243 per tick.
//! 1. Shares (relative constants, unit scale D): at rest, for the median phenotype, senses 7.5 % and brain 7.5 %
//!    (together 15 %, target band 10–20 % of TCK-171), body 85 %. The brain share is split 20 % per-neuron base /
//!    80 % activity (Attwell & Laughlin: ~80 % of the signalling budget is action potentials and synaptic
//!    transmission). Movement: full thrust costs the resting cost of the median body (active ≈ 2× resting), so speed
//!    is not free; for any body full thrust costs `body / 0.85`, about its resting cost.
//!    body 0.85·D/M = 3.559e-5, organ 0.075·D/O = 1.044e-3, neuron 0.2·0.075·D/N = 2.604e-5,
//!    activity 0.8·0.075·D/A = 1.518e-4, movement D/M = 4.187e-5.
//! 2. Absolute scale: the median TYPICAL TOTAL cost (resting at the genome's trial activity + movement at its trial
//!    mean |thrust|) equals D, so active creatures pay what the old model charged (matching only the resting cost
//!    doubled the Ocean death turnover, P6 Ocean measurement). With the step-1 constants that median is 0.039834, so
//!    all five constants are multiplied by f = 0.0243 / 0.039834 = 0.6100; shares and the movement ratio are unchanged:
//!   - `bodyCostPerMass`     = 2.171e-5
//!   - `organCostPerLoad`    = 6.369e-4
//!   - `neuronCost`          = 1.589e-5
//!   - `activityCost`        = 9.26e-5
//!   - `movementCostPerMass` = 2.554e-5
//!    Resulting median resting cost ≈ 0.0152 per tick.
//! - `trainerMetabolicScale` multiplies the trainer drain only (pressure ramp of TCK-176); the Ocean uses 1.

use serde::{Deserialize, Serialize};

use crate::biology::dna::{CreaturePhenotype, NeuronRole};
use crate::shared::types::{CreatureAgent, SimulationRules};

/// Config block `metabolism` (derivation in the module docs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetabolismConfig {
    pub body_cost_per_mass: f32,
    pub organ_cost_per_load: f32,
    pub neuron_cost: f32,
    pub activity_cost: f32,
    pub movement_cost_per_mass: f32,
    /// Factor on the trainer drain (1 = full Ocean cost); ramped by the curriculum (TCK-176)
    pub trainer_metabolic_scale: f32,
}

/// Body mass used by the physics (`r^1.5 · L / 25`).
pub fn body_mass(phenotype: &CreaturePhenotype) -> f32 {
    crate::shared::physics::body_mass(phenotype)
}

/// Organ load `Σ scale² · (1.1 − bandwidth)`: big, sharply tuned organs cost most (shape of the former vestigial organ tax).
pub fn organ_load(phenotype: &CreaturePhenotype) -> f32 {
    phenotype.organelles.iter().map(|o| o.scale * o.scale * (1.1 - o.bandwidth)).sum()
}

/// Neurons that integrate (thalamus, hidden, motors). Sensor and systemic slots are inputs written by the
/// sensing code, not neurons; their cost is part of the organ term.
pub fn brain_neurons(phenotype: &CreaturePhenotype) -> usize {
    phenotype.brain.neurons.iter().filter(|n| !matches!(n.role, NeuronRole::Sensor { .. } | NeuronRole::Systemic { .. })).count()
}

/// Energy cost of one tick, per term.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyCost {
    pub body: f32,
    pub senses: f32,
    pub neurons: f32,
    pub activity: f32,
    pub movement: f32,
}

impl EnergyCost {
    pub fn brain(&self) -> f32 {
        self.neurons + self.activity
    }

    /// Cost without movement
    pub fn resting(&self) -> f32 {
        self.body + self.senses + self.brain()
    }

    pub fn total(&self) -> f32 {
        self.resting() + self.movement
    }
}

/// Cost of one tick for `phenotype` with synaptic `activity` (`BrainRuntime::activity`) and motor output `out_thrust`.
pub fn tick_cost(phenotype: &CreaturePhenotype, activity: f32, out_thrust: f32, m: &MetabolismConfig) -> EnergyCost {
    let mass = body_mass(phenotype);
    EnergyCost {
        body: m.body_cost_per_mass * mass,
        senses: m.organ_cost_per_load * organ_load(phenotype),
        neurons: m.neuron_cost * brain_neurons(phenotype) as f32,
        activity: m.activity_cost * activity,
        movement: m.movement_cost_per_mass * mass * out_thrust.abs(),
    }
}

/// Drains one tick of metabolism (`tick_cost` of the agent's last brain step, times `scale`) from the agent's energy
/// and returns the drain. Called right after the brain step by the Ocean (scale 1) and the trainer
/// (`trainerMetabolicScale`).
pub fn drain_energy(agent: &mut CreatureAgent, out_thrust: f32, m: &MetabolismConfig, scale: f32) -> f32 {
    let drain = tick_cost(&agent.phenotype, agent.brain_runtime.activity(), out_thrust, m).total() * scale;
    agent.energy -= drain;
    drain
}

/// Energy of eating one food spore: `foodSporeEnergy · diet efficiency · scale` (meat: carnivory · biting scale,
/// plant: (1 − carnivory) · grazing scale). The one food energy rule of Ocean and trainer.
pub fn food_energy(carnivory: f32, is_meat: bool, rules: &SimulationRules) -> f32 {
    if is_meat {
        rules.food_spore_energy * carnivory * rules.biting_efficiency_carnivore_scale
    } else {
        rules.food_spore_energy * (1.0 - carnivory) * rules.grazing_efficiency_herbivore_scale
    }
}

/// Energy an attacker gains from one bite (TCK-145): `bitingBaseEnergyGain · carnivory · bitingEfficiencyCarnivoreScale`,
/// capped by the damage dealt (`bitingEnergyDamage`) and by the victim's energy: a bite moves energy, it never creates it.
pub fn bite_energy_gain(carnivory: f32, victim_energy: f32, rules: &SimulationRules) -> f32 {
    (rules.biting_base_energy_gain * carnivory * rules.biting_efficiency_carnivore_scale)
        .min(rules.biting_energy_damage)
        .min(victim_energy.max(0.0))
}

/// Typical brain behaviour of `genome` over one trial of `scenario` (fresh trainer sandbox, learning per `config`,
/// no metabolic drain so the brain runs the whole trial): mean synaptic activity `Σ|w·a|` per tick and mean
/// `|out_thrust|` per tick. The typical activity and thrust of the cost calibration.
pub fn mean_trial_activity_and_thrust(genome: &str, scenario: crate::biology::scenarios::Scenario, config: &crate::shared::types::AppConfig, rng: &mut crate::shared::rng::SimRng) -> (f32, f32) {
    use crate::biology::dna::Motor;
    use crate::biology::trainer_engine::{init_rust_sandbox, step_trainer_sandbox_physics_with_config, SandboxOrigin};
    let mut config = config.clone();
    config.metabolism.trainer_metabolic_scale = 0.0;
    let mut sb = init_rust_sandbox(1, genome, 1, SandboxOrigin::Random, scenario, None, None, rng);
    let thrust_neuron = sb.agent.phenotype.brain.neurons.iter().position(|n| n.role == NeuronRole::Motor { motor: Motor::Thrust });
    let ticks = scenario.plugin().epoch_ticks();
    let (mut activity, mut thrust) = (0.0, 0.0);
    for _ in 0..ticks {
        step_trainer_sandbox_physics_with_config(&mut sb, rng, &config);
        activity += sb.agent.brain_runtime.activity();
        thrust += thrust_neuron.map_or(0.0, |i| sb.agent.neuron_activations[i].abs());
    }
    (activity / ticks as f32, thrust / ticks as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::dna::parse_genome;
    use crate::biology::scenarios::Scenario;
    use crate::shared::rng::derive;
    use crate::shared::types::AppConfig;

    /// Old median Ocean drain of structured genomes (module docs), the absolute scale of the calibration.
    const PRE_P6_MEDIAN_OCEAN_DRAIN: f32 = 0.0243;
    /// Typical activity and |thrust| of the derivation (medians over one standard-room trial, module docs).
    const TYPICAL_ACTIVITY: f32 = 9.60623;
    const TYPICAL_THRUST: f32 = 0.7029;

    fn median(mut v: Vec<f32>) -> f32 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = v.len();
        if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) }
    }

    fn structured_phenotypes(n: usize) -> Vec<CreaturePhenotype> {
        let composition = &AppConfig::global().genome_composition;
        (0..n)
            .map(|i| parse_genome(&crate::biology::dna::generate_structured_genome(composition, &mut derive(151, &[i as u64])), None, None))
            .collect()
    }

    #[test]
    fn test_brain_and_senses_share_of_resting_cost_is_in_band() {
        let m = &AppConfig::global().metabolism;
        let phenotypes = structured_phenotypes(1000);
        let costs: Vec<EnergyCost> = phenotypes.iter().map(|p| tick_cost(p, TYPICAL_ACTIVITY, 0.0, m)).collect();

        // Median phenotype (median of every term): the derivation target 15 %
        let term = |f: fn(&EnergyCost) -> f32| median(costs.iter().map(f).collect());
        let (body, senses, brain) = (term(|c| c.body), term(|c| c.senses), term(|c| c.brain()));
        let share = (senses + brain) / (body + senses + brain);
        assert!((0.10..=0.20).contains(&share), "median-phenotype brain + senses share {}", share);

        // Per-genome shares: the median creature also lies in the band
        let per_genome = median(costs.iter().map(|c| (c.senses + c.brain()) / c.resting()).collect());
        assert!((0.10..=0.20).contains(&per_genome), "median per-genome brain + senses share {}", per_genome);
    }

    #[test]
    fn test_typical_total_cost_matches_the_old_ocean_drain() {
        // Real structured brains over a standard trial (200 genomes): the documented typical activity and |thrust| still
        // hold (±15 %), and the median typical total cost (resting at the own activity + movement at the own mean
        // |thrust|) equals the pre-P6 median Ocean drain (±10 %), the absolute scale of the calibration.
        let config = AppConfig::global();
        let composition = &config.genome_composition;
        let rows: Vec<(f32, f32, f32)> = (0..200)
            .map(|i| {
                let genome = crate::biology::dna::generate_structured_genome(composition, &mut derive(151, &[i as u64]));
                let (activity, thrust) = mean_trial_activity_and_thrust(&genome, Scenario::Standard, config, &mut derive(152, &[i as u64]));
                let total = tick_cost(&parse_genome(&genome, None, None), activity, thrust, &config.metabolism).total();
                (activity, thrust, total)
            })
            .collect();
        let activity = median(rows.iter().map(|r| r.0).collect());
        let thrust = median(rows.iter().map(|r| r.1).collect());
        let total = median(rows.iter().map(|r| r.2).collect());
        assert!((activity / TYPICAL_ACTIVITY - 1.0).abs() < 0.15, "median trial activity {}", activity);
        assert!((thrust / TYPICAL_THRUST - 1.0).abs() < 0.15, "median trial |thrust| {}", thrust);
        assert!((total / PRE_P6_MEDIAN_OCEAN_DRAIN - 1.0).abs() < 0.10, "median typical total cost {}", total);
    }

    #[test]
    fn test_full_thrust_costs_about_the_resting_cost() {
        let m = &AppConfig::global().metabolism;
        let phenotypes = structured_phenotypes(300);
        let ratio = median(
            phenotypes
                .iter()
                .map(|p| {
                    let cost = tick_cost(p, TYPICAL_ACTIVITY, 1.0, m);
                    cost.movement / cost.resting()
                })
                .collect(),
        );
        assert!((0.8..=1.25).contains(&ratio), "full-thrust movement / resting cost {}", ratio);
        // Backward thrust costs the same, no thrust nothing
        let p = &phenotypes[0];
        assert_eq!(tick_cost(p, 1.0, -0.5, m).movement, tick_cost(p, 1.0, 0.5, m).movement);
        assert_eq!(tick_cost(p, 1.0, 0.0, m).movement, 0.0);
    }

    #[test]
    fn test_pruned_and_silent_synapses_cost_nothing() {
        let m = &AppConfig::global().metabolism;
        let p = &structured_phenotypes(1)[0];
        let silent = tick_cost(p, 0.0, 0.0, m);
        assert_eq!(silent.activity, 0.0);
        assert!(tick_cost(p, 5.0, 0.0, m).activity > 0.0);
        // More organs, more neurons: more cost, linear and additive (no multiplicative brain term)
        assert!(silent.body > 0.0 && silent.neurons > 0.0);
        assert_eq!(silent.total(), silent.body + silent.senses + silent.neurons);
    }

    #[test]
    fn test_food_and_bite_energy() {
        let rules = &AppConfig::global().rules;
        assert_eq!(food_energy(0.0, false, rules), rules.food_spore_energy * rules.grazing_efficiency_herbivore_scale);
        assert_eq!(food_energy(1.0, true, rules), rules.food_spore_energy * rules.biting_efficiency_carnivore_scale);
        assert_eq!(food_energy(0.0, true, rules), 0.0);
        let full = (rules.biting_base_energy_gain * 0.8 * rules.biting_efficiency_carnivore_scale).min(rules.biting_energy_damage);
        assert_eq!(bite_energy_gain(0.8, 1000.0, rules), full);
        // Never more than the damage dealt
        assert!(bite_energy_gain(1.0, 1000.0, rules) <= rules.biting_energy_damage);
        assert_eq!(bite_energy_gain(0.8, 3.0, rules), 3.0);
        assert_eq!(bite_energy_gain(0.8, -1.0, rules), 0.0);
    }
}
