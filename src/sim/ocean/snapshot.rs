//! `OceanSnapshot` (TCK-156): the persisted Ocean state (`simulation_state.current_state`, JSON). Typed instead of a
//! `json!` value; reads snapshots of earlier versions (the fields added in TCK-156 default when missing).

use serde::{Deserialize, Serialize};

use crate::shared::dto::{CreatureAgent, FoodSpore};

use super::spore_field::FIRST_SPORE_ID;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OceanSnapshot {
    pub creatures: Vec<CreatureAgent>,
    #[serde(default)]
    pub food_pellets: Vec<FoodSpore>,
    #[serde(default = "one")]
    pub highest_generation: u32,
    #[serde(default = "one")]
    pub next_creature_id: u32,
    #[serde(default = "first_spore_id")]
    pub next_spore_id: u32,
    /// Nutrient centres of the spore field (not persisted before TCK-156: a restart drew new ones)
    #[serde(default)]
    pub nutrient_centers: Vec<(f32, f32)>,
    /// Ocean seed (`OceanSim::seed`); missing in older snapshots
    #[serde(default)]
    pub seed: Option<u64>,
    /// Ocean tick, the index of the per-tick RNG streams
    #[serde(default)]
    pub tick: u64,
    /// Count of releases (index of the action RNG streams)
    #[serde(default)]
    pub actions: u64,
}

fn one() -> u32 {
    1
}

fn first_spore_id() -> u32 {
    FIRST_SPORE_ID
}
