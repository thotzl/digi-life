//! Room registry (TCK-201): the one place a new room is registered. The runner flags (`--scenario`, `--schedule`,
//! `--eval-final`), the trainer's fixed modes and the probes find rooms here by id.

use std::sync::Arc;

use crate::config::AppConfig;

use super::Room;
use super::level;

/// One registered room: its id and its constructor (rooms read their parameters from the config).
pub struct RoomEntry {
    pub id: &'static str,
    pub build: fn(&AppConfig) -> Arc<dyn Room>,
    /// One line for `trainer_runner` usage and the docs
    pub summary: &'static str,
}

/// All rooms, in the order the runner lists them.
pub const ROOMS: &[RoomEntry] = &[
    RoomEntry {
        id: "standard",
        build: level::standard,
        summary: "small room: level 0 of the level room (1 plant + 1 meat)",
    },
    RoomEntry {
        id: "exploration",
        build: level::exploration,
        summary: "large room: level 1 of the level room (3 plants + 3 meat)",
    },
];

pub fn entry(id: &str) -> Option<&'static RoomEntry> {
    ROOMS.iter().find(|e| e.id == id)
}

/// The room registered under `id`, built from `config`.
pub fn build(id: &str, config: &AppConfig) -> Option<Arc<dyn Room>> {
    entry(id).map(|e| (e.build)(config))
}

/// The static form of a registered id.
pub fn static_id(id: &str) -> Option<&'static str> {
    entry(id).map(|e| e.id)
}

pub fn ids() -> impl Iterator<Item = &'static str> {
    ROOMS.iter().map(|e| e.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_registered_room_builds_under_its_id_and_ids_are_unique() {
        let config = crate::config::test_config();
        let mut seen = std::collections::BTreeSet::new();
        for e in ROOMS {
            assert!(seen.insert(e.id), "duplicate room id {}", e.id);
            let room = (e.build)(config);
            assert_eq!(room.id(), e.id);
            assert!(
                room.chamber_size() > 0.0 && room.episode_ticks() > 0 && room.entities() >= 1,
                "{}",
                e.id
            );
        }
        assert!(build("no-such-room", config).is_none());
    }
}
