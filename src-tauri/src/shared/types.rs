use serde::{Deserialize, Serialize};
use crate::biology::dna::CreaturePhenotype;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodSpore {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub amount: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureAgent {
    pub id: u32,
    #[serde(rename = "speciesId")]
    pub species_id: String,
    
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    
    #[serde(rename = "headingAngle")]
    pub heading_angle: f32,
    #[serde(rename = "bendAngle")]
    pub bend_angle: f32,
    #[serde(rename = "omegaRot")]
    pub omega_rot: f32,
    
    pub energy: f32,
    pub adrenaline: f32,
    pub age: u32,
    pub generation: u32,
    #[serde(rename = "hasEaten")]
    pub has_eaten: bool,
    
    pub genome: String,
    pub antisense: String,
    pub phenotype: CreaturePhenotype,

    #[serde(rename = "neuronStates")]
    pub neuron_states: Vec<f32>,
    #[serde(rename = "neuronActivations")]
    pub neuron_activations: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryCreature {
    pub id: u32,
    #[serde(rename = "speciesId")]
    pub species_id: String,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    #[serde(rename = "headingAngle")]
    pub heading_angle: f32,
    #[serde(rename = "omegaRot")]
    pub omega_rot: f32,
    pub energy: f32,
    pub adrenaline: f32,
    pub age: u32,
    pub generation: u32,
    #[serde(rename = "hasEaten")]
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
            adrenaline: c.adrenaline,
            age: c.age,
            generation: c.generation,
            has_eaten: c.has_eaten,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimulationRules {
    pub creature_max_age_ticks: u32,
    pub creature_mature_age_floor: u32,
    pub lamarckian_assimilation_age_floor: u32,
    pub lamarckian_assimilation_chance: f32,
    pub bmr_base_scale: f32,
    pub adrenaline_max_multiplier: f32,
    pub adrenaline_increase_rate: f32,
    pub adrenaline_decay_rate: f32,
    pub adrenaline_metabolic_surcharge_scale: f32,
    pub thermal_stress_penalty_scale: f32,
    pub bioluminescence_flash_cost: f32,
    pub photosynthesis_energy_gain: f32,
    pub biome_hazard_damage_scale: f32,
    pub biome_algae_spore_spawn_threshold: f32,
    pub grazing_radius_multiplier: f32,
    pub grazing_radius_offset: f32,
    pub grazing_efficiency_herbivore_scale: f32,
    pub biting_carnivory_threshold: f32,
    pub biting_radius_multiplier: f32,
    pub biting_radius_offset: f32,
    pub biting_energy_damage: f32,
    pub biting_base_energy_gain: f32,
    pub biting_efficiency_carnivore_scale: f32,
    pub reproduction_stomach_threshold_floor: f32,
    pub reproduction_split_loss_ratio: f32,
    pub reproduction_recoil_velocity_scale: f32,
    pub thrust_base_multiplier: f32,
    pub steer_torque_base_multiplier: f32,
    pub predator_savage_thrust_threshold: f32,
    pub predator_savage_thrust_multiplier: f32,
    pub drag_forward_coefficient: f32,
    pub drag_forward_stiffness_decay: f32,
    pub drag_lateral_coefficient: f32,
    pub receptor_ballast_scale: f32,
    pub elastic_wall_restitution: f32,
    pub hebbian_learning_rate_base: f32,
    pub hebbian_learning_stiffness_decay: f32,
    pub hebbian_forgetting_decay: f32,
    pub decomposition_spore_min: i32,
    pub decomposition_spore_max: i32,
    pub decomposition_size_ratio: f32,
    pub restock_founder_gene_inherit_chance: f32,
    pub restock_initial_stomach_ratio: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub seed: String,
    pub target_population: usize,
    pub food_spore_count: usize,
    pub max_creatures: usize,
    pub basal_metabolic_rate_multiplier: f32,
    pub rules: SimulationRules,
}

impl AppConfig {
    pub fn load() -> Self {
        let paths = ["config.json", "../config.json", "src-tauri/config.json"];
        for path in &paths {
            if let Ok(file_content) = std::fs::read_to_string(path) {
                if let Ok(parsed) = serde_json::from_str::<Self>(&file_content) {
                    println!("[CONFIG] Successfully loaded application config from '{}'!", path);
                    return parsed;
                }
            }
        }
        
        println!("[CONFIG] Warning: Failed to load config.json, using compiled fallback defaults!");
        Self {
            seed: "ALIFE_BASIN_77A".to_string(),
            target_population: 25,
            food_spore_count: 600,
            max_creatures: 45,
            basal_metabolic_rate_multiplier: 1.0,
            rules: SimulationRules {
                creature_max_age_ticks: 5400,
                creature_mature_age_floor: 600,
                lamarckian_assimilation_age_floor: 1200,
                lamarckian_assimilation_chance: 0.25,
                bmr_base_scale: 0.002,
                adrenaline_max_multiplier: 1.8,
                adrenaline_increase_rate: 0.06,
                adrenaline_decay_rate: 0.015,
                adrenaline_metabolic_surcharge_scale: 1.5,
                thermal_stress_penalty_scale: 0.0012,
                bioluminescence_flash_cost: 0.05,
                photosynthesis_energy_gain: 0.20,
                biome_hazard_damage_scale: 0.40,
                biome_algae_spore_spawn_threshold: 1.0,
                grazing_radius_multiplier: 1.5,
                grazing_radius_offset: 4.0,
                grazing_efficiency_herbivore_scale: 1.50,
                biting_carnivory_threshold: 0.35,
                biting_radius_multiplier: 1.6,
                biting_radius_offset: 5.0,
                biting_energy_damage: 35.0,
                biting_base_energy_gain: 35.0,
                biting_efficiency_carnivore_scale: 1.25,
                reproduction_stomach_threshold_floor: 0.60,
                reproduction_split_loss_ratio: 0.4,
                reproduction_recoil_velocity_scale: 15.0,
                thrust_base_multiplier: 6.8,
                steer_torque_base_multiplier: 5.8,
                predator_savage_thrust_threshold: 0.55,
                predator_savage_thrust_multiplier: 1.45,
                drag_forward_coefficient: 0.015,
                drag_forward_stiffness_decay: 0.3,
                drag_lateral_coefficient: 0.045,
                receptor_ballast_scale: 0.18,
                elastic_wall_restitution: 0.5,
                hebbian_learning_rate_base: 0.00015,
                hebbian_learning_stiffness_decay: 0.85,
                hebbian_forgetting_decay: 0.0000032,
                decomposition_spore_min: 1,
                decomposition_spore_max: 5,
                decomposition_size_ratio: 1200.0,
                restock_founder_gene_inherit_chance: 0.60,
                restock_initial_stomach_ratio: 0.60,
            }
        }
    }
}
