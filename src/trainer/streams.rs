//! Named RNG streams of the trainer (TCK-201): every random draw of a run comes from `derive(seed, path)` with one
//! of these stream ids at a fixed place of the path, so streams never overlap and results do not depend on
//! `parallel` or the thread count. Stream ids count down from `u64::MAX`; generation, layout and trial indices never
//! reach them.

/// Breeding slot `s` of generation `g`: `derive(seed, [g, BREED, s])`.
pub const BREED: u64 = u64::MAX;
/// Shared layout of trial `t` of layout `l`: `derive(seed, [LAYOUT, l, t])`.
pub const LAYOUT: u64 = u64::MAX - 1;
/// Rehearsal choice of layout `l`: `derive(seed, [LAYOUT, l, REHEARSAL_PICK])` (trial indices never reach it).
pub const REHEARSAL_PICK: u64 = u64::MAX;
/// Evaluation layout of trial `t` (`TrainerSim::evaluate_population`): `derive(seed, [EVAL, t])`.
pub const EVAL: u64 = u64::MAX - 2;
/// Reserved for an evaluator that pairs slots into shared chambers (opponent pairing, arena):
/// `derive(seed, [g, PAIRING, trial])`. Unused by `SoloTrials`.
pub const PAIRING: u64 = u64::MAX - 3;
/// Reserved for births during a trial (a reproduction room): `derive(respawn_seed, [BIRTHS, k])` for birth `k`.
/// Unused by the level rooms.
pub const BIRTHS: u64 = u64::MAX - 4;

/// Respawns inside a trial need no stream id: catch `k` of a chamber draws from `derive(respawn_seed, [k])`
/// (`Chamber::step`), the respawn seed itself coming from the trial's layout stream.
pub fn layout(seed: u64, layout_id: u64, trial: u32) -> crate::shared::rng::SimRng {
    crate::shared::rng::derive(seed, &[LAYOUT, layout_id, trial as u64])
}
