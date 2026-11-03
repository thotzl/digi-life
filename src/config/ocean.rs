//! Config section `ocean` (`OceanConfig`): world size, population, food, lifespan, biting, mitosis and
//! decomposition rules of the Ocean (`sim::ocean::OceanSim`, TCK-156). Food and bite energy live in section `metabolism`
//! (shared with the trainer).

use serde::{Deserialize, Serialize};

/// Section `ocean` of `config.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OceanConfig {
    /// World width in px (RV-A05, TCK-156); sent to the UI as `INIT_STATE.world.width`
    pub world_width: f32,
    /// World height in px; sent to the UI as `INIT_STATE.world.height`
    pub world_height: f32,
    /// Restock target: below it the Ocean spawns catalogue clones or structured random founders
    pub target_population: usize,
    pub food_spore_count: usize,
    /// Cap for `INJECT_URZELLE` / `SPAWN_CLONE` and mitosis
    pub max_creatures: usize,
    /// Lifespan `mature_age · creatureLifespanMatureAgeMultiplier`, the one lifespan source (TCK-202/TCK-146: the
    /// former cap `creatureMaxAgeTicks` 36000 lay above the longest lifespan 32,400 and never bound)
    pub creature_lifespan_mature_age_multiplier: u32,
    /// Mitosis floors on the phenotype's mature age and stomach threshold
    pub creature_mature_age_floor: u32,
    /// Parents younger than this pass no learned weights to mitosis children
    pub lamarckian_assimilation_age_floor: u32,
    pub thermal_stress_penalty_scale: f32,
    pub photosynthesis_energy_gain: f32,
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
    /// Retention of `species_records` (TCK-159): extinct species kept, newest extinction first; alive ones always stay.
    /// Pruned with the 10 s Ocean snapshot.
    pub species_record_retention: usize,
}
