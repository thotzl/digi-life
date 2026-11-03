//! Evaluation schemes (TCK-201): how a slot is scored in a trial and how its trials combine into the generation
//! fitness. `SoloTrials` (one slot per chamber, every slot faces the same shared layout) is today's trainer and is
//! used by `TrainerSim` (live score, trial end, generation end), `TrainerSim::evaluate_population` and the diagnosis
//! probes. A new scheme (e.g. a tournament that pairs slots into shared chambers, stream `streams::PAIRING`) is one
//! more implementation. Contract: `wiki/contracts/evaluator.md`.

use std::sync::Arc;

use rayon::prelude::*;

use crate::biology::trainer_config::TrialAggregation;
use crate::config::AppConfig;
use crate::shared::rng::SimRng;
use crate::shared::stats::{quantile, sort_ascending};

use super::room::Room;
use super::sandbox::TrainerSandbox;

/// Result of complete trials of one slot (`Evaluator::evaluate`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SlotEval {
    /// Fitness of every trial, in trial order
    pub trial_fitness: Vec<f32>,
    /// Trials with at least one catch
    pub successes: u32,
    /// Catches over all trials
    pub catches: u32,
}

/// An evaluation scheme. Object-safe; pure functions of their arguments (deterministic, no IO).
pub trait Evaluator: Send + Sync + std::fmt::Debug {
    fn id(&self) -> &str;

    /// Fitness of a slot in the trial running now: per tick for the UI telemetry, at the trial end for selection.
    fn score(&self, sb: &TrainerSandbox) -> f32;

    /// Generation fitness of a slot from its trial fitness (trial order).
    fn aggregate(&self, trial_fitness: &[f32], aggregation: TrialAggregation) -> f32;

    /// Evaluates `sandboxes` (clones; the originals are not touched) for `trials` complete trials in `room`, trial `t`
    /// on the layout `layout(t)`, learned weights carried across the trials as in training. Results in sandbox order;
    /// `parallel` never changes them.
    fn evaluate(&self, room: &Arc<dyn Room>, sandboxes: &[TrainerSandbox], trials: u32, layout: &(dyn Fn(u32) -> SimRng + Sync), config: &AppConfig, parallel: bool) -> Vec<SlotEval>;
}

/// Combines the per-trial fitness of one sandbox: mean, or median (mean of the two middle values for an even
/// count); 0 without trials.
pub fn aggregate_trials(fitness: &[f32], aggregation: TrialAggregation) -> f32 {
    if fitness.is_empty() {
        return 0.0;
    }
    match aggregation {
        TrialAggregation::Mean => fitness.iter().sum::<f32>() / fitness.len() as f32,
        TrialAggregation::Median => {
            let mut sorted = fitness.to_vec();
            sort_ascending(&mut sorted);
            quantile(&sorted, 0.5)
        }
    }
}

/// Today's trainer evaluation (TCK-174/175): every slot alone in its chamber, scored by the room
/// (`Room::score` of entity 0), the generation fitness the mean or median of its trial fitness.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SoloTrials;

impl SoloTrials {
    /// Runs `trials` complete trials of `sb` in its room. Trial `t` starts with `sb.reset_trial(layout(t))`, except
    /// trial 0 when `fresh` (the sandbox was just spawned on `layout(0)` and runs that layout as is).
    pub fn run(&self, sb: &mut TrainerSandbox, trials: u32, fresh: bool, layout: impl Fn(u32) -> SimRng, config: &AppConfig) -> SlotEval {
        let mut eval = SlotEval { trial_fitness: Vec::with_capacity(trials as usize), ..SlotEval::default() };
        for trial in 0..trials {
            if trial > 0 || !fresh {
                sb.reset_trial(&mut layout(trial), config);
            }
            while !sb.is_done() {
                sb.step(config);
            }
            eval.trial_fitness.push(self.score(sb));
            eval.successes += sb.trial().finished as u32;
            eval.catches += sb.trial().consumed_count;
        }
        eval
    }
}

impl Evaluator for SoloTrials {
    fn id(&self) -> &str {
        "solo"
    }

    fn score(&self, sb: &TrainerSandbox) -> f32 {
        sb.score()
    }

    fn aggregate(&self, trial_fitness: &[f32], aggregation: TrialAggregation) -> f32 {
        aggregate_trials(trial_fitness, aggregation)
    }

    fn evaluate(&self, room: &Arc<dyn Room>, sandboxes: &[TrainerSandbox], trials: u32, layout: &(dyn Fn(u32) -> SimRng + Sync), config: &AppConfig, parallel: bool) -> Vec<SlotEval> {
        let one = |sb: &TrainerSandbox| {
            let mut sb = sb.clone();
            sb.room = room.clone();
            self.run(&mut sb, trials, false, layout, config)
        };
        if parallel { sandboxes.par_iter().map(one).collect() } else { sandboxes.iter().map(one).collect() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::genome::WILDTYPE_GENOME;
    use crate::shared::rng::{derive, seeded};
    use crate::trainer::room::registry;
    use crate::trainer::sandbox::SandboxOrigin;

    #[test]
    fn test_trial_aggregation_mean_and_median() {
        assert_eq!(aggregate_trials(&[1.0, 2.0, 9.0], TrialAggregation::Mean), 4.0);
        assert_eq!(aggregate_trials(&[9.0, 1.0, 2.0], TrialAggregation::Median), 2.0);
        assert_eq!(aggregate_trials(&[4.0, 1.0, 2.0, 9.0], TrialAggregation::Median), 3.0);
        assert_eq!(aggregate_trials(&[], TrialAggregation::Median), 0.0);
    }

    #[test]
    fn test_solo_run_equals_a_hand_written_trial_loop() {
        let config = crate::config::test_config();
        let room = registry::build("standard", config).unwrap();
        let layout = |t: u32| derive(7, &[t as u64]);
        let spawn = || TrainerSandbox::spawn(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, room.clone(), None, None, &mut layout(0), config);
        let mut a = spawn();
        let eval = SoloTrials.run(&mut a, 3, true, layout, config);

        let mut b = spawn();
        let mut fitness = Vec::new();
        for t in 0..3 {
            if t > 0 {
                b.reset_trial(&mut layout(t), config);
            }
            for _ in 0..room.episode_ticks() {
                b.step(config);
            }
            fitness.push(b.score());
        }
        assert_eq!(eval.trial_fitness, fitness);
        assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
        // Batch form: clones, sandbox order, parallel == sequential
        let pop: Vec<TrainerSandbox> = (0..4).map(|i| TrainerSandbox::spawn(i, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, room.clone(), None, None, &mut seeded(i as u64), config)).collect();
        let seq = SoloTrials.evaluate(&room, &pop, 2, &layout, config, false);
        assert_eq!(seq, SoloTrials.evaluate(&room, &pop, 2, &layout, config, true));
        assert!(seq.iter().all(|e| e.trial_fitness.len() == 2));
    }
}
