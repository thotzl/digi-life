//! Typed Ocean IPC (TCK-157): the Ocean actions of `handle_client_action` and the Ocean events on
//! `simulation-state`. Tagged by `type`; exported to `bindings/` by ts-rs. Contract: `wiki/contracts/ipc-message.md`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::shared::dto::{CreatureAgent, FoodSpore, TelemetryCreature};
use crate::shared::map_generator::ProceduralWorld;

/// Ocean actions from the UI (tag `type`).
#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum OceanAction {
    /// Run or pause the Ocean; answered with `SIM_STATE`
    ToggleSimulation { running: bool },
    /// Release one structured random founder (rejected with a log line at the population cap)
    InjectUrzelle,
    /// Release a clone of a given genome (sent by the backend's `spawn_catalogue_creature_to_ocean`)
    SpawnClone(SpawnClone),
    /// Clear species, snapshot and creatures and repopulate (clears the newborn queue, invariant 3)
    ResetEvolution,
    /// Creature whose live activations `TELEMETRY_TICK.selectedBrain` carries; absent / null clears the selection
    SelectAgent {
        #[serde(default)]
        #[ts(optional)]
        id: Option<u32>,
    },
    /// Ocean steps per frame (clamped to 35 by the engine, invariant 2; answered with `SIM_STATE` carrying the clamped
    /// value)
    SetSimulationSpeed { speed: u32 },
}

/// A clone to release into the Ocean (field names in snake case like `AssignSandboxCreature`).
#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[ts(export)]
pub struct SpawnClone {
    pub genome: String,
    #[serde(default)]
    #[ts(optional)]
    pub methylations: Option<Vec<f32>>,
    /// Used only with `load_learned_synapses`; the factory falls back to the genome weights on a length mismatch
    #[serde(default)]
    #[ts(optional)]
    pub synapse_weights: Option<Vec<f32>>,
    #[serde(default)]
    #[ts(optional)]
    pub load_learned_synapses: Option<bool>,
}

/// Ocean events sent to the UI (payload of `simulation-state`, tag `type`).
#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum OceanUiEvent {
    /// Full state on `CLIENT_READY` and after `RESET_EVOLUTION`
    InitState(OceanInitState),
    /// Answer to `TOGGLE_SIMULATION` and `SET_SIMULATION_SPEED`: run state and the steps per frame after the warp cap
    /// (`speed` ≤ 35, invariant 2; TCK-156)
    SimState { running: bool, speed: u32 },
    /// 25 Hz while the Ocean runs
    TelemetryTick(OceanTelemetryTick),
    /// 2 Hz: all food spores
    FoodTick {
        #[serde(rename = "foodPellets")]
        food_pellets: Vec<FoodSpore>,
    },
    /// A spore was eaten at (x, y) (position before it respawned)
    EatEvent { x: f32, y: f32 },
    /// A bite at the victim's position
    BiteEvent {
        #[serde(rename = "attackerId")]
        attacker_id: u32,
        #[serde(rename = "victimId")]
        victim_id: u32,
        x: f32,
        y: f32,
    },
}

/// Everything the Ocean view needs to (re)build its state: all creatures with phenotype and brain, all spores and
/// the procedural world.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OceanInitState {
    pub highest_generation: u32,
    pub running: bool,
    pub seed: String,
    pub world: ProceduralWorld,
    pub rules: OceanRules,
    pub food_pellets: Vec<FoodSpore>,
    pub creatures: Vec<CreatureAgent>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OceanRules {
    pub spore_energy: f32,
}

/// Live Ocean frame: light creatures, the creatures born since the last frame (full, with phenotype) and the
/// selected creature's activations.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OceanTelemetryTick {
    pub highest_generation: u32,
    pub creatures: Vec<TelemetryCreature>,
    pub new_creatures: Vec<CreatureAgent>,
    pub selected_brain: Option<SelectedBrain>,
}

/// Live neuron activations of the selected Ocean creature (brain view).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct SelectedBrain {
    pub id: u32,
    pub activations: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_ocean_events_keep_type_tag_and_ui_field_names() {
        let tick = OceanUiEvent::TelemetryTick(OceanTelemetryTick {
            highest_generation: 3,
            creatures: vec![],
            new_creatures: vec![],
            selected_brain: Some(SelectedBrain {
                id: 2,
                activations: vec![0.5],
            }),
        });
        assert_eq!(
            serde_json::to_value(&tick).unwrap(),
            json!({ "type": "TELEMETRY_TICK", "highestGeneration": 3, "creatures": [], "newCreatures": [], "selectedBrain": { "id": 2, "activations": [0.5] } })
        );
        assert_eq!(
            serde_json::to_value(OceanUiEvent::SimState {
                running: true,
                speed: 35
            })
            .unwrap(),
            json!({ "type": "SIM_STATE", "running": true, "speed": 35 })
        );
        assert_eq!(
            serde_json::to_value(OceanUiEvent::FoodTick { food_pellets: vec![] }).unwrap(),
            json!({ "type": "FOOD_TICK", "foodPellets": [] })
        );
        assert_eq!(
            serde_json::to_value(OceanUiEvent::EatEvent { x: 1.5, y: 2.0 }).unwrap(),
            json!({ "type": "EAT_EVENT", "x": 1.5, "y": 2.0 })
        );
        assert_eq!(
            serde_json::to_value(OceanUiEvent::BiteEvent {
                attacker_id: 1,
                victim_id: 2,
                x: 0.5,
                y: 0.25
            })
            .unwrap(),
            json!({ "type": "BITE_EVENT", "attackerId": 1, "victimId": 2, "x": 0.5, "y": 0.25 })
        );
        let init = serde_json::to_value(OceanUiEvent::InitState(OceanInitState {
            highest_generation: 1,
            running: false,
            seed: "s".to_string(),
            world: crate::shared::map_generator::generate_world("s", 1920.0, 1080.0),
            rules: OceanRules { spore_energy: 15.0 },
            food_pellets: vec![],
            creatures: vec![],
        }))
        .unwrap();
        assert_eq!(init["type"], "INIT_STATE");
        assert_eq!(init["rules"]["sporeEnergy"], 15.0);
        for key in [
            "highestGeneration",
            "running",
            "seed",
            "world",
            "foodPellets",
            "creatures",
        ] {
            assert!(init.get(key).is_some(), "{}", key);
        }
    }
}
