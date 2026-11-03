//! Training modes: the UI mode `Scenario` (bindings) and the trainer's `TrainingMode` (a registry room or the
//! curriculum), TCK-201. Contract: `wiki/contracts/room.md`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

use super::room::registry;

/// UI training mode (`SET_TRAINER_MODE`, `TrainerState::training_mode`, stored as `trainer_run_state.scenario`).
/// `Standard` and `Exploration` select the registry rooms of the same id; `Curriculum` runs the room of the trainer's
/// current level, moved by the success gate (TCK-176). A room without a UI mode is selected through
/// `TrainingMode::Room` (headless runner).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Scenario {
    #[default]
    Standard,
    Exploration,
    Curriculum,
}

impl Scenario {
    /// UI mode string (`SET_TRAINER_MODE`); anything but "exploration" or "curriculum" is the standard room.
    pub fn from_mode(mode: &str) -> Self {
        match mode {
            "exploration" => Scenario::Exploration,
            "curriculum" => Scenario::Curriculum,
            _ => Scenario::Standard,
        }
    }

    /// UI mode string; inverse of `from_mode` (also the stored form in `trainer_run_state.scenario`, TCK-191).
    pub fn mode(self) -> &'static str {
        match self {
            Scenario::Standard => "standard",
            Scenario::Exploration => "exploration",
            Scenario::Curriculum => "curriculum",
        }
    }

    /// Registry id of the room of a fixed mode (`None` for the curriculum, whose room is trainer state).
    pub fn room_id(self) -> Option<&'static str> {
        match self {
            Scenario::Standard => Some("standard"),
            Scenario::Exploration => Some("exploration"),
            Scenario::Curriculum => None,
        }
    }

    /// Difficulty level the UI shows for a fixed mode (`None` for the curriculum, whose level is trainer state).
    pub fn fixed_level(self) -> Option<f32> {
        match self {
            Scenario::Standard => Some(0.0),
            Scenario::Exploration => Some(1.0),
            Scenario::Curriculum => None,
        }
    }
}

/// What the trainer trains in: one fixed room of the registry (by its static id) or the curriculum. Serialized as
/// the id (`"standard"`, `"exploration"`, `"curriculum"`, or any other registry id), the form of the runner flags
/// `--scenario` / `--schedule` and of the report field `scenario`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingMode {
    Room(&'static str),
    Curriculum,
}

impl TrainingMode {
    /// `"curriculum"` or a registry room id.
    pub fn parse(id: &str) -> Result<Self, String> {
        if id == "curriculum" {
            return Ok(TrainingMode::Curriculum);
        }
        registry::static_id(id).map(TrainingMode::Room).ok_or_else(|| format!("unknown room '{}' (known: {}, curriculum)", id, registry::ids().collect::<Vec<_>>().join(", ")))
    }

    pub fn id(self) -> &'static str {
        match self {
            TrainingMode::Room(id) => id,
            TrainingMode::Curriculum => "curriculum",
        }
    }

    /// UI mode of this training mode; a room without a UI mode shows as `Standard`.
    pub fn scenario(self) -> Scenario {
        match self {
            TrainingMode::Curriculum => Scenario::Curriculum,
            TrainingMode::Room(id) => Scenario::from_mode(id),
        }
    }
}

impl From<Scenario> for TrainingMode {
    fn from(scenario: Scenario) -> Self {
        scenario.room_id().map_or(TrainingMode::Curriculum, TrainingMode::Room)
    }
}

impl Serialize for TrainingMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for TrainingMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        TrainingMode::parse(&id).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_modes_map_to_registry_rooms_and_back() {
        for scenario in [Scenario::Standard, Scenario::Exploration, Scenario::Curriculum] {
            let mode = TrainingMode::from(scenario);
            assert_eq!(mode.id(), scenario.mode());
            assert_eq!(mode.scenario(), scenario);
            assert_eq!(TrainingMode::parse(scenario.mode()), Ok(mode));
            assert_eq!(serde_json::to_string(&mode).unwrap(), serde_json::to_string(&scenario).unwrap(), "same report form");
        }
        assert!(TrainingMode::parse("arena-that-does-not-exist").is_err());
        assert_eq!(serde_json::from_str::<TrainingMode>("\"exploration\"").unwrap(), TrainingMode::Room("exploration"));
    }
}
