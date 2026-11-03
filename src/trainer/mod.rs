//! Trainer (TCK-173, rooms/evaluator/breeder as seams since TCK-201): neuroevolution in training rooms.
//!
//! - `room`: the `Room` trait, the room registry (`room::registry::ROOMS`) and the level room
//! - `chamber`: the owned world of a sandbox (N creatures, food, trial records) and its tick on the shared creature
//!   systems of `crate::sim`
//! - `sandbox`: one breeding slot with its chamber and the room of its trial
//! - `evaluator`: the `Evaluator` trait and `SoloTrials` (scoring, trial aggregation, batch evaluation)
//! - `breeder`: parents, hall of fame, slot plan, breeding, selection, 1/5 rule input, lineage statistics
//! - `curriculum`: success gate, rehearsal, metabolic ramp
//! - `genomes`: genomes of new slots, random genomes, minimal criterion
//! - `mode`: the UI mode `Scenario` and the `TrainingMode` (registry room or curriculum)
//! - `stats`: the generation summary; `streams`: the named RNG streams
//! - `sim`: `TrainerSim`, which only orchestrates the above (IO-free)
//!
//! Contracts: `wiki/contracts/room.md`, `wiki/contracts/evaluator.md`; guide `wiki/guides/add-room-or-mode.md`.

pub mod breeder;
pub mod chamber;
pub mod curriculum;
pub mod evaluator;
pub mod genomes;
pub mod mode;
pub mod room;
pub mod sandbox;
pub mod sim;
pub mod stats;
pub mod streams;
