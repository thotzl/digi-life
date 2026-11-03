//! Curriculum (TCK-176): the success gate on the difficulty level, rehearsal trials and the metabolic ramp. The
//! trainer applies them at generation ends (`TrainerSim`, module doc of `trainer::sim`).

use rand::Rng;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::biology::trainer_config::TrainerCurriculumConfig;
use crate::shared::rng::derive;

use super::streams;

/// Curriculum settings of a run (TCK-176; defaults from config `trainerCurriculum`, the runner overrides them).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurriculumParams {
    pub upper: f32,
    pub lower: f32,
    pub step: f32,
    pub start_level: f32,
    pub rehearsal_share: f32,
    pub rehearsal_level: f32,
    pub metabolic_ramp_start: f32,
}

impl CurriculumParams {
    pub fn from_config(c: &TrainerCurriculumConfig) -> Self {
        Self {
            upper: c.upper,
            lower: c.lower,
            step: c.step,
            start_level: c.start_level,
            rehearsal_share: c.rehearsal_share,
            rehearsal_level: c.rehearsal_level,
            metabolic_ramp_start: c.metabolic_ramp_start,
        }
    }
}

/// Decision of the curriculum gate at a generation end (`GenerationStats::level_move`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum LevelMove {
    Up,
    Down,
    /// Success between the thresholds, or the level already at the bound it would move past
    Hold,
}

/// Level clamped to [0, 1] and rounded to 1e-6, so repeated steps of 0.1 land on the same values both ways and
/// reach the ends exactly.
pub fn snap_level(level: f32) -> f32 {
    if !level.is_finite() {
        return 0.0;
    }
    (level.clamp(0.0, 1.0) * 1e6).round() / 1e6
}

/// Curriculum gate (TCK-176): success above `upper` → `level + step`, below `lower` → `level − step`, else unchanged;
/// clamped to [0, 1]. Returns the new level and whether it moved.
pub fn gate_level(level: f32, success_rate: f32, c: &CurriculumParams) -> (f32, LevelMove) {
    let target = if success_rate > c.upper {
        level + c.step
    } else if success_rate < c.lower {
        level - c.step
    } else {
        level
    };
    let next = snap_level(target);
    let movement = if next > level {
        LevelMove::Up
    } else if next < level {
        LevelMove::Down
    } else {
        LevelMove::Hold
    };
    (next, movement)
}

/// Metabolic pressure ramp of the curriculum: factor `ramp_start + (1 − ramp_start) · level` on the trainer
/// metabolic scale (1 at level 1; constant 1 for `ramp_start` = 1).
pub fn metabolic_ramp(level: f32, ramp_start: f32) -> f32 {
    ramp_start + (1.0 - ramp_start) * level
}

/// Rehearsal trials of a layout (sorted): `round(trials · share)` of them, at most `trials − 1` so at least one trial
/// runs at the current level; drawn without replacement from the layout's stream `derive(seed, [LAYOUT, layout_id,
/// REHEARSAL_PICK])`, so every sandbox and every generation on that layout has the same choice.
pub fn rehearsal_trials(seed: u64, layout_id: u64, trials: u32, share: f32) -> Vec<u32> {
    let count = if share > 0.0 {
        ((trials as f32 * share).round() as u32).min(trials.saturating_sub(1))
    } else {
        0
    };
    if count == 0 {
        return Vec::new();
    }
    let mut rng = derive(seed, &[streams::LAYOUT, layout_id, streams::REHEARSAL_PICK]);
    let mut order: Vec<u32> = (0..trials).collect();
    for i in 0..count as usize {
        let j = rng.gen_range(i..order.len());
        order.swap(i, j);
    }
    let mut picked = order[..count as usize].to_vec();
    picked.sort_unstable();
    picked
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Curriculum params independent of the calibrated config defaults: no rehearsal, no ramp.
    pub(crate) fn plain_curriculum() -> CurriculumParams {
        CurriculumParams {
            upper: 0.6,
            lower: 0.3,
            step: 0.1,
            start_level: 0.0,
            rehearsal_share: 0.0,
            rehearsal_level: 0.0,
            metabolic_ramp_start: 1.0,
        }
    }

    #[test]
    fn test_gate_moves_up_down_and_clamps() {
        let c = plain_curriculum();
        assert_eq!(gate_level(0.5, 0.7, &c), (0.6, LevelMove::Up));
        assert_eq!(gate_level(0.5, 0.2, &c), (0.4, LevelMove::Down));
        assert_eq!(gate_level(0.5, 0.45, &c), (0.5, LevelMove::Hold));
        assert_eq!(
            gate_level(0.5, 0.6, &c),
            (0.5, LevelMove::Hold),
            "the thresholds themselves hold"
        );
        assert_eq!(gate_level(0.5, 0.3, &c), (0.5, LevelMove::Hold));
        assert_eq!(gate_level(1.0, 0.9, &c), (1.0, LevelMove::Hold), "clamped at 1");
        assert_eq!(gate_level(0.0, 0.0, &c), (0.0, LevelMove::Hold), "clamped at 0");
        // Ten steps reach both ends exactly and retrace the same levels
        let mut up = vec![0.0f32];
        for _ in 0..10 {
            up.push(gate_level(*up.last().unwrap(), 1.0, &c).0);
        }
        assert_eq!(*up.last().unwrap(), 1.0);
        let mut down = vec![1.0f32];
        for _ in 0..10 {
            down.push(gate_level(*down.last().unwrap(), 0.0, &c).0);
        }
        down.reverse();
        assert_eq!(up, down);
    }

    #[test]
    fn test_rehearsal_trial_selection_is_deterministic() {
        assert_eq!(rehearsal_trials(5, 0, 3, 0.0), Vec::<u32>::new());
        assert_eq!(
            rehearsal_trials(5, 0, 1, 1.0),
            Vec::<u32>::new(),
            "a single trial never rehearses"
        );
        assert_eq!(
            rehearsal_trials(5, 0, 3, 1.0).len(),
            2,
            "at least one trial stays at the level"
        );
        let mut picks = std::collections::BTreeSet::new();
        for layout in 0..20 {
            let picked = rehearsal_trials(5, layout, 3, 1.0 / 3.0);
            assert_eq!(picked, rehearsal_trials(5, layout, 3, 1.0 / 3.0));
            assert_eq!(picked.len(), 1);
            assert!(picked[0] < 3);
            picks.insert(picked[0]);
        }
        assert!(picks.len() > 1, "the choice varies between layouts: {:?}", picks);
    }

    #[test]
    fn test_metabolic_ramp_values() {
        assert_eq!(metabolic_ramp(0.0, 0.5), 0.5);
        assert_eq!(metabolic_ramp(1.0, 0.0), 1.0);
        assert_eq!(metabolic_ramp(0.3, 1.0), 1.0, "ramp start 1 = no ramp");
    }
}
