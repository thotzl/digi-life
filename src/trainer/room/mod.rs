//! Training rooms as plugins (TCK-201). A room is one `Room` implementation plus one entry in `registry::ROOMS`.
//! Contract: `wiki/contracts/room.md`, guide `wiki/guides/add-room-or-mode.md`.
//!
//! A room owns the rules of one trial: chamber size, episode length, the rules of its world for the shared creature
//! tick (`WorldSettings`, obstacles), the start layout of creatures and food, the end of an episode and the score of
//! an entity. It holds no per-trial state: everything that changes during a trial lives in the `Chamber`, which the
//! trainer owns and lends to the room's hooks.

pub mod level;
pub mod registry;

use crate::config::AppConfig;
use crate::shared::map_generator::ProceduralObstacle;
use crate::shared::rng::SimRng;
use crate::sim::WorldSettings;

use super::chamber::Chamber;

pub use level::{calculate_fitness, LevelRoom};

/// Index of a creature in its chamber (`Chamber::creatures`); entity 0 is the creature of the sandbox's breeding slot.
pub type EntityId = usize;

/// Per-chamber numbers of a room for the UI telemetry and the report.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomTelemetry {
    /// Difficulty level of the room (curriculum level; 0 for rooms without levels)
    pub level: f32,
}

/// A training room. Object-safe; the trainer holds rooms as `Arc<dyn Room>` (shared by all sandboxes of a trial).
/// Every method is a pure function of the room, the chamber and the explicit RNG: no IO, no global state.
pub trait Room: Send + Sync + std::fmt::Debug {
    /// Registry id (`registry::ROOMS`); a room built outside the registry (curriculum level) names its kind
    fn id(&self) -> &str;

    /// Edge length of the square chamber in px
    fn chamber_size(&self) -> f32;

    /// Ticks of one trial (episode)
    fn episode_ticks(&self) -> u32;

    /// Creatures per chamber. Entity 0 is the slot's creature; a room with more entities adds them in `layout`.
    fn entities(&self) -> usize {
        1
    }

    /// Rules of the room's world for the shared creature tick (`sim::step_creature`): systems, sensing, drain,
    /// death, food drift, eat radius, respawn. `config` is the config of the step (probes pass modified ones).
    fn world_settings(&self, config: &AppConfig) -> WorldSettings;

    /// Static obstacles of the chamber (none in the level rooms)
    fn obstacles(&self) -> &[ProceduralObstacle] {
        &[]
    }

    /// Start layout of a trial: places every creature (position, heading) and the food. Called on a chamber whose
    /// creatures and trial records were just reset (`Chamber::start_trial`); draws only from `rng`, which is the
    /// trial's shared layout stream in the trainer (the same task for every sandbox of a generation).
    fn layout(&self, chamber: &mut Chamber, rng: &mut SimRng);

    /// The episode is over (default: after `episode_ticks` ticks)
    fn is_done(&self, chamber: &Chamber) -> bool {
        chamber.tick >= self.episode_ticks()
    }

    /// Fitness of `entity` in the current trial (called per tick for the UI telemetry and at the end of the trial)
    fn score(&self, chamber: &Chamber, entity: EntityId) -> f32;

    /// Difficulty level of the room (curriculum level; the report's `level` of a fixed room). Default 0.
    fn level(&self) -> f32 {
        0.0
    }

    /// Per-chamber numbers for the UI telemetry (default: the room's level)
    fn telemetry(&self, _chamber: &Chamber) -> RoomTelemetry {
        RoomTelemetry { level: self.level() }
    }
}
