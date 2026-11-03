//! Generation summary of the trainer (the report's `generations[]` and the UI metrics, TCK-173/177/178/181).

use serde::{Deserialize, Serialize};

use crate::biology::genome::{MutationChannel, MutationTouch};
use crate::biology::organs::OrganKind;
use crate::shared::stats::{quantile, sort_ascending};

use super::curriculum::LevelMove;
use super::mode::TrainingMode;
use super::sandbox::TrainerSandbox;

/// A trial counts as immobile if the creature travelled less than this in total (px over the trial).
pub const IMMOBILE_DISTANCE_PX: f32 = 5.0;

/// Outcome of one slot in one trial.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrialRecord {
    /// Breeding slot (index into the generation's sandboxes before ranking)
    pub slot: usize,
    pub finish_tick: Option<u32>,
    pub distance_traveled: f32,
    /// Energy at trial end
    pub end_energy: f32,
    /// The creature ran out of energy during the trial (TCK-178)
    pub starved: bool,
    /// Rehearsal trial of the curriculum (excluded from the gate, TCK-176)
    pub rehearsal: bool,
}

/// Generation summary, computed at epoch end over the final fitness, before breeding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationStats {
    pub generation: u32,
    /// Share of sandbox-trials with at least one catch
    pub success_rate: f32,
    /// Over the `finish_tick` of successful trials; `None` without any catch
    pub mean_time_to_first_catch: Option<f32>,
    pub median_time_to_first_catch: Option<f32>,
    pub best_fitness: f32,
    pub median_fitness: f32,
    pub mean_fitness: f32,
    /// p10, p25, p50, p75, p90 (linear interpolation)
    pub fitness_quantiles: [f32; 5],
    pub mean_genome_length: f32,
    /// Share of sandbox-trials below `IMMOBILE_DISTANCE_PX` travelled
    pub immobile_share: f32,
    /// Mean energy at trial end over the sandbox-trials (TCK-178)
    pub mean_end_energy: f32,
    /// Share of sandbox-trials in which the creature ran out of energy (TCK-178)
    pub starved_share: f32,
    /// Mean organ count of the generation's sandboxes (sensor equipment, TCK-178)
    pub mean_organ_count: f32,
    /// Share of the generation's sandboxes with at least one eye or nose (TCK-178)
    pub food_organ_share: f32,
    /// Shared layout this generation was evaluated on (TCK-174)
    pub layout_id: u64,
    /// Max mutation ops per child used to breed this generation (TCK-174; adapted by the 1/5 rule)
    pub mutation_ops_max: u32,
    /// Share of mutants with a higher fitness than their parent's unmodified elite slot of the same generation;
    /// `None` without such mutants (TCK-174, input of the 1/5 rule)
    pub child_success_share: Option<f32>,
    /// Mutant children of this generation per mutation channel (TCK-181, `trainer::breeder`)
    pub mutation_channels: MutationChannelStats,
    /// Channel of the last mutation in each elite's lineage, elites in rank order (an unmodified elite copy keeps its
    /// parent's); `None` for a lineage without any mutation (random immigrant or generation 1) or a restored parent
    pub elite_last_mutation: Vec<Option<MutationChannel>>,
    /// Training mode of this generation (TCK-176): a registry room id or "curriculum"
    pub scenario: TrainingMode,
    /// Difficulty level of the generation's non-rehearsal trials (fixed rooms: their room level, 0 / 1)
    pub level: f32,
    /// Success rate of the non-rehearsal trials (the gate input); equals `success_rate` without rehearsal
    pub level_success_rate: f32,
    /// Trials of each sandbox run at the rehearsal level
    pub rehearsal_trials: u32,
    /// Success rate of the rehearsal trials; `None` without rehearsal
    pub rehearsal_success_rate: Option<f32>,
    /// Gate decision after this generation; `None` when the gate is off (fixed room, manual level, or a pending
    /// manual switch)
    pub level_move: Option<LevelMove>,
}

/// Mutant children (origin `Mutant`) of one mutation channel in one generation (TCK-181).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChannelStats {
    pub mutants: u32,
    /// Share with a higher fitness than their parent (same reference as `child_success_share`); `None` without mutants
    pub beat_parent_share: Option<f32>,
    /// Share ranked among the elites of this generation (the parents of the next one); `None` without mutants
    pub elite_share: Option<f32>,
}

/// `ChannelStats` per channel of a child's mutations (union of its operator calls, `genome::classify_mutation`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MutationChannelStats {
    pub body: ChannelStats,
    pub brain: ChannelStats,
    pub both: ChannelStats,
    pub neutral: ChannelStats,
    /// Mutants with at least one call in an organ gene (counted as body; organ payloads also carry thalamus -> motor weights)
    pub organ_mutants: u32,
}

impl MutationChannelStats {
    /// From the mutant slots: (channel set, beat its parent, ranked among the elites).
    pub fn compute(mutants: &[(MutationTouch, bool, bool)]) -> Self {
        let of = |channel: MutationChannel| {
            let members: Vec<_> = mutants.iter().filter(|m| m.0.channel() == channel).collect();
            let share = |f: fn(&(MutationTouch, bool, bool)) -> bool| (!members.is_empty()).then(|| members.iter().filter(|m| f(m)).count() as f32 / members.len() as f32);
            ChannelStats { mutants: members.len() as u32, beat_parent_share: share(|m| m.1), elite_share: share(|m| m.2) }
        };
        Self {
            body: of(MutationChannel::Body),
            brain: of(MutationChannel::Brain),
            both: of(MutationChannel::Both),
            neutral: of(MutationChannel::Neutral),
            organ_mutants: mutants.iter().filter(|m| m.0.organ).count() as u32,
        }
    }
}

impl GenerationStats {
    /// Trial and population figures; the breeding, layout and curriculum fields are filled in by the trainer.
    pub fn compute(generation: u32, trials: &[TrialRecord], sandboxes: &[TrainerSandbox], scenario: TrainingMode) -> Self {
        let trial_count = trials.len().max(1) as f32;
        let success_of = |rehearsal: bool| {
            let selected: Vec<&TrialRecord> = trials.iter().filter(|t| t.rehearsal == rehearsal).collect();
            (!selected.is_empty()).then(|| selected.iter().filter(|t| t.finish_tick.is_some()).count() as f32 / selected.len() as f32)
        };
        let (level_success, rehearsal_success) = (success_of(false), success_of(true));
        let rehearsal_trials = trials.iter().filter(|t| t.rehearsal).count() as u32 / (sandboxes.len().max(1) as u32);
        let mut catch_ticks: Vec<f32> = trials.iter().filter_map(|t| t.finish_tick).map(|t| t as f32).collect();
        sort_ascending(&mut catch_ticks);
        let (mean_catch, median_catch) = if catch_ticks.is_empty() {
            (None, None)
        } else {
            (Some(catch_ticks.iter().sum::<f32>() / catch_ticks.len() as f32), Some(quantile(&catch_ticks, 0.5)))
        };

        let mut fitness: Vec<f32> = sandboxes.iter().map(|s| s.current_fitness).collect();
        sort_ascending(&mut fitness);
        let n = fitness.len().max(1) as f32;
        let quantiles = [0.1, 0.25, 0.5, 0.75, 0.9].map(|q| if fitness.is_empty() { 0.0 } else { quantile(&fitness, q) });

        Self {
            generation,
            success_rate: catch_ticks.len() as f32 / trial_count,
            mean_time_to_first_catch: mean_catch,
            median_time_to_first_catch: median_catch,
            best_fitness: fitness.last().copied().unwrap_or(0.0),
            median_fitness: quantiles[2],
            mean_fitness: fitness.iter().sum::<f32>() / n,
            fitness_quantiles: quantiles,
            mean_genome_length: sandboxes.iter().map(|s| s.agent().genome.len() as f32).sum::<f32>() / n,
            immobile_share: trials.iter().filter(|t| t.distance_traveled < IMMOBILE_DISTANCE_PX).count() as f32 / trial_count,
            mean_end_energy: trials.iter().map(|t| t.end_energy).sum::<f32>() / trial_count,
            starved_share: trials.iter().filter(|t| t.starved).count() as f32 / trial_count,
            mean_organ_count: sandboxes.iter().map(|s| s.agent().phenotype.organelles.len() as f32).sum::<f32>() / n,
            food_organ_share: sandboxes
                .iter()
                .filter(|s| s.agent().phenotype.organelles.iter().any(|o| matches!(o.kind, OrganKind::Eye | OrganKind::Nose)))
                .count() as f32
                / n,
            layout_id: 0,
            mutation_ops_max: 0,
            child_success_share: None,
            mutation_channels: MutationChannelStats::default(),
            elite_last_mutation: Vec::new(),
            scenario,
            level: 0.0,
            level_success_rate: level_success.unwrap_or(catch_ticks.len() as f32 / trial_count),
            rehearsal_trials,
            rehearsal_success_rate: rehearsal_success,
            level_move: None,
        }
    }
}
