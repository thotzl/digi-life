//! Config section `ocean` (`OceanConfig`): population, food, lifespan, grazing, biting, mitosis and decomposition
//! rules of the Ocean (`server::engine`). Food and bite energy live in section `metabolism` (shared with the trainer).

use serde::{Deserialize, Serialize};

/// Section `ocean` of `config.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OceanConfig {
    /// Restock target: below it the Ocean spawns catalogue clones or structured random founders
    pub target_population: usize,
    pub food_spore_count: usize,
    /// Cap for `INJECT_URZELLE` / `SPAWN_CLONE` and mitosis
    pub max_creatures: usize,
    /// Lifespan `min(mature_age · creatureLifespanMatureAgeMultiplier, creatureMaxAgeTicks)`
    pub creature_max_age_ticks: u32,
    pub creature_lifespan_mature_age_multiplier: u32,
    /// Mitosis floors on the phenotype's mature age and stomach threshold
    pub creature_mature_age_floor: u32,
    /// Parents younger than this pass no learned weights to mitosis children
    pub lamarckian_assimilation_age_floor: u32,
    pub thermal_stress_penalty_scale: f32,
    pub photosynthesis_energy_gain: f32,
    /// Eat radius `mean_radius · grazingRadiusMultiplier + grazingRadiusOffset`
    pub grazing_radius_multiplier: f32,
    pub grazing_radius_offset: f32,
    /// Carnivory at or above which a creature bites (and below which green creatures photosynthesise)
    pub biting_carnivory_threshold: f32,
    /// Bite range `mean_radius · bitingRadiusMultiplier + bitingRadiusOffset`
    pub biting_radius_multiplier: f32,
    pub biting_radius_offset: f32,
    /// Ticks before the same attacker can bite the same victim again (TCK-145)
    pub bite_cooldown_ticks: u32,
    pub reproduction_stomach_threshold_floor: f32,
    pub reproduction_split_loss_ratio: f32,
    pub reproduction_recoil_velocity_scale: f32,
    /// Meat spores of a corpse: `floor(length · radius / decompositionSizeRatio)` clamped to [min, max]
    pub decomposition_spore_min: i32,
    pub decomposition_spore_max: i32,
    pub decomposition_size_ratio: f32,
}
