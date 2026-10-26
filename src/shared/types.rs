use serde::{Deserialize, Serialize};
use crate::biology::dna::{CreaturePhenotype, GenomeComposition, MutationOperators};
use crate::biology::metabolism::MetabolismConfig;
use crate::shared::brain::BrainRuntime;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FoodSpore {
    pub id: u32,
    #[serde(rename = "typeId")]
    #[ts(rename = "typeId")]
    pub type_id: u32, // 1 = Plant/Algae, 2 = Meat/Carcass
    pub x: f32,
    pub y: f32,
    pub amount: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreatureAgent {
    pub id: u32,
    #[serde(rename = "speciesId")]
    #[ts(rename = "speciesId")]
    pub species_id: String,
    
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    
    #[serde(rename = "headingAngle")]
    #[ts(rename = "headingAngle")]
    pub heading_angle: f32,
    #[serde(rename = "bendAngle")]
    #[ts(rename = "bendAngle")]
    pub bend_angle: f32,
    #[serde(rename = "omegaRot")]
    #[ts(rename = "omegaRot")]
    pub omega_rot: f32,
    
    pub energy: f32,
    pub age: u32,
    pub generation: u32,
    #[serde(rename = "hasEaten")]
    #[ts(rename = "hasEaten")]
    pub has_eaten: bool,
    
    pub genome: String,
    pub antisense: String,
    pub phenotype: CreaturePhenotype,

    #[serde(rename = "neuronStates")]
    #[ts(rename = "neuronStates")]
    pub neuron_states: Vec<f32>,
    #[serde(rename = "neuronActivations")]
    #[ts(rename = "neuronActivations")]
    pub neuron_activations: Vec<f32>,

    #[serde(rename = "synapseWeights")]
    #[ts(rename = "synapseWeights")]
    pub synapse_weights: Vec<f32>,

    /// Derived brain data (runtime only; reset when phenotype or weights are replaced in place)
    #[serde(skip)]
    #[ts(skip)]
    pub brain_runtime: BrainRuntime,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TelemetryCreature {
    pub id: u32,
    #[serde(rename = "speciesId")]
    #[ts(rename = "speciesId")]
    pub species_id: String,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    #[serde(rename = "headingAngle")]
    #[ts(rename = "headingAngle")]
    pub heading_angle: f32,
    #[serde(rename = "omegaRot")]
    #[ts(rename = "omegaRot")]
    pub omega_rot: f32,
    pub energy: f32,
    pub age: u32,
    pub generation: u32,
    #[serde(rename = "hasEaten")]
    #[ts(rename = "hasEaten")]
    pub has_eaten: bool,
}

impl From<&CreatureAgent> for TelemetryCreature {
    fn from(c: &CreatureAgent) -> Self {
        Self {
            id: c.id,
            species_id: c.species_id.clone(),
            px: c.px,
            py: c.py,
            vx: c.vx,
            vy: c.vy,
            heading_angle: c.heading_angle,
            omega_rot: c.omega_rot,
            energy: c.energy,
            age: c.age,
            generation: c.generation,
            has_eaten: c.has_eaten,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SimulationRules {
    pub creature_max_age_ticks: u32,
    pub creature_lifespan_mature_age_multiplier: u32,
    pub creature_mature_age_floor: u32,
    pub lamarckian_assimilation_age_floor: u32,
    pub lamarckian_assimilation_chance: f32,
    pub thermal_stress_penalty_scale: f32,
    pub photosynthesis_energy_gain: f32,
    pub grazing_radius_multiplier: f32,
    pub grazing_radius_offset: f32,
    pub grazing_efficiency_herbivore_scale: f32,
    pub biting_carnivory_threshold: f32,
    pub biting_radius_multiplier: f32,
    pub biting_radius_offset: f32,
    pub biting_energy_damage: f32,
    pub biting_base_energy_gain: f32,
    pub biting_efficiency_carnivore_scale: f32,
    /// Ticks before the same attacker can bite the same victim again (TCK-145)
    pub bite_cooldown_ticks: u32,
    /// Energy of one food spore before diet efficiency and scale; the one source of food energy for Ocean and
    /// trainer (`metabolism::food_energy`, TCK-178)
    pub food_spore_energy: f32,
    pub reproduction_stomach_threshold_floor: f32,
    pub reproduction_split_loss_ratio: f32,
    pub reproduction_recoil_velocity_scale: f32,
    /// Terminal speed at full thrust of the slowest / fastest genome (px/tick); thrust is derived from it (TCK-150)
    pub locomotion_speed_min: f32,
    pub locomotion_speed_max: f32,
    /// Heading change per tick per unit bend independent of speed (rad), allows turning at rest (TCK-150)
    pub turn_rate_at_rest: f32,
    pub drag_forward_coefficient: f32,
    pub drag_forward_stiffness_decay: f32,
    pub receptor_ballast_scale: f32,
    pub elastic_wall_restitution: f32,
    /// η of the Oja rule of plastic synapses (TCK-152)
    pub hebbian_learning_rate_base: f32,
    pub decomposition_spore_min: i32,
    pub decomposition_spore_max: i32,
    pub decomposition_size_ratio: f32,
    pub creature_start_stomach_ratio: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppConfig {
    pub seed: String,
    pub target_population: usize,
    pub food_spore_count: usize,
    pub max_creatures: usize,
    /// Trainer genomes kept per run in the DB and in the in-memory hall of fame (best by fitness)
    pub trainer_genome_retention_per_run: usize,
    /// Random genomes of trainer generation 1, immigrants and Ocean founders (TCK-184)
    pub genome_composition: GenomeComposition,
    /// Operator weights of `mutate_genome` (TCK-184)
    pub mutation_operators: MutationOperators,
    /// Constants of the shared energy function (`biology::metabolism`, TCK-151)
    pub metabolism: MetabolismConfig,
    pub rules: SimulationRules,
}

use std::sync::OnceLock;
static GLOBAL_CONFIG: OnceLock<AppConfig> = OnceLock::new();
const EMBEDDED_CONFIG: &str = include_str!("../../config.json");

impl AppConfig {
    pub fn global() -> &'static Self {
        GLOBAL_CONFIG.get_or_init(|| Self::load())
    }

    /// Loads the config compiled into the binary. A `config.json` in the working directory
    /// overrides it for development; an invalid override is reported and ignored.
    pub fn load() -> Self {
        if let Ok(file_content) = std::fs::read_to_string("config.json") {
            match serde_json::from_str::<Self>(&file_content) {
                Ok(parsed) => {
                    eprintln!("[CONFIG] Loaded config override from working directory config.json.");
                    return parsed;
                }
                Err(e) => eprintln!("[CONFIG] ERROR: working directory config.json is invalid ({}). Using embedded config.", e),
            }
        }
        eprintln!("[CONFIG] Using embedded config.json.");
        Self::embedded()
    }

    pub fn embedded() -> Self {
        serde_json::from_str(EMBEDDED_CONFIG).expect("embedded config.json must match AppConfig")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_config_matches_config_json() {
        let embedded = AppConfig::embedded();
        let on_disk: serde_json::Value = serde_json::from_str(include_str!("../../config.json")).unwrap();

        assert_eq!(embedded.food_spore_count as u64, on_disk["foodSporeCount"].as_u64().unwrap());
        assert_eq!(embedded.target_population as u64, on_disk["targetPopulation"].as_u64().unwrap());
        assert_eq!(embedded.rules.creature_max_age_ticks as u64, on_disk["rules"]["creatureMaxAgeTicks"].as_u64().unwrap());
        assert!((embedded.rules.locomotion_speed_max as f64 - on_disk["rules"]["locomotionSpeedMax"].as_f64().unwrap()).abs() < 1e-6);
    }

    fn camel_to_snake(key: &str) -> String {
        let mut out = String::new();
        for c in key.chars() {
            if c.is_ascii_uppercase() {
                out.push('_');
                out.push(c.to_ascii_lowercase());
            } else {
                out.push(c);
            }
        }
        out
    }

    fn collect_rust_sources(dir: &std::path::Path, out: &mut String) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_rust_sources(&path, out);
            } else if path.extension().map_or(false, |e| e == "rs") && !path.ends_with("shared/types.rs") {
                out.push_str(&std::fs::read_to_string(&path).unwrap());
            }
        }
    }

    /// Every config key must be read somewhere in `src/` (no exemptions since TCK-145 wired `bitingBaseEnergyGain`).
    #[test]
    fn test_every_config_key_has_read_site() {
        let on_disk: serde_json::Value = serde_json::from_str(include_str!("../../config.json")).unwrap();
        // Top-level keys plus the keys of every nested block (rules, genomeComposition, ...)
        let mut keys: Vec<String> = Vec::new();
        for (key, value) in on_disk.as_object().unwrap() {
            match value.as_object() {
                Some(block) => {
                    keys.push(key.clone());
                    keys.extend(block.keys().cloned());
                }
                None => keys.push(key.clone()),
            }
        }

        let mut sources = String::new();
        collect_rust_sources(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut sources);

        let unread: Vec<String> = keys
            .iter()
            .map(|k| camel_to_snake(k))
            .filter(|field| !sources.contains(&format!(".{}", field)))
            .collect();
        assert!(unread.is_empty(), "config keys without a read site in src/: {:?}", unread);
    }

    #[test]
    fn test_unknown_config_key_is_rejected() {
        let mut on_disk: serde_json::Value = serde_json::from_str(include_str!("../../config.json")).unwrap();
        on_disk["rules"]["ventsBaseCount"] = serde_json::json!(6);
        assert!(serde_json::from_value::<AppConfig>(on_disk).is_err());
    }
}
