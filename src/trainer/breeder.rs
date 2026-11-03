//! Selection and breeding (moved out of `TrainerSim`, TCK-201): parents and hall of fame, the slot plan, the breeding
//! of a generation, the lineage of every slot (mutation channels, TCK-181), elite selection and the input of the 1/5
//! success rule (TCK-174). One implementation; `TrainerSim` only orchestrates it.

use std::sync::Arc;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biology::brain::CTRNNSynapse;
use crate::biology::creature_factory::inherit_learned_weights;
use crate::biology::genome::{MutationChannel, MutationTouch, WILDTYPE_GENOME};
use crate::biology::phenotype::parse_genome;
use crate::config::AppConfig;
use crate::shared::rng::derive;

use super::genomes::breed_genome_traced;
use super::room::Room;
use super::sandbox::{SandboxOrigin, TrainerSandbox};
use super::sim::TrainerParams;
use super::stats::MutationChannelStats;
use super::streams;

/// A scored genome (elite of a generation, parent or hall-of-fame entry).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenomeRecord {
    pub genome: String,
    pub name: String,
    pub fitness: f32,
    pub generation: u32,
    pub methylations: Vec<f32>,
    pub synapse_weights: Vec<f32>,
}

/// Breeding record of one slot (TCK-181): its origin, the channels of its own mutations (mutated origins) and the
/// channel of the last mutation in its lineage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotLineage {
    pub origin: SandboxOrigin,
    pub own: Option<MutationTouch>,
    pub last: Option<MutationChannel>,
}

/// One slot of the plan: source genome, origin, index of the source parent.
type SlotPlan<'a> = (&'a str, SandboxOrigin, Option<usize>);

/// 1/5 success rule on the max ops per child `k` (TCK-174): `better` of `total` mutants beat their parent. Share
/// above 1/5 → `k + 1`, below 1/5 → `k - 1`, exactly 1/5 or no mutants → unchanged; the result is clamped to
/// `[lo, max(hi, lo)]`. Integer comparison (`5 · better` vs `total`), so the rule is exact and deterministic.
pub fn one_fifth_update(k: u32, better: usize, total: usize, lo: u32, hi: u32) -> u32 {
    let next = if total == 0 || 5 * better == total {
        k
    } else if 5 * better > total {
        k.saturating_add(1)
    } else {
        k.saturating_sub(1)
    };
    next.clamp(lo, hi.max(lo))
}

/// Layout advance rule (TCK-174): the layout is solved when every elite (the first `elite_count` entries of the
/// fitness ranking, at least one) caught food in every trial.
pub fn elites_solved_layout(ranked_solved: &[bool], elite_count: usize) -> bool {
    !ranked_solved.is_empty() && ranked_solved.iter().take(elite_count.max(1)).all(|&s| s)
}

/// Elites of a population of `n`: `clamp(round(n · ratio), 1, n)`.
pub fn elite_count(n: usize, ratio: f32) -> usize {
    (((n as f32) * ratio).round() as usize).clamp(1, n)
}

fn sort_desc_by_fitness(records: &mut [GenomeRecord]) {
    records.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
}

/// Outcome of the selection at a generation end (`Breeder::select`).
#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    /// The new parents, fitness descending
    pub elites: Vec<GenomeRecord>,
    pub elite_count: usize,
    /// Solved flags (caught food in every trial) in rank order
    pub ranked_solved: Vec<bool>,
    /// Mutants with a parent that beat it, of `mutants_with_parent` (input of the 1/5 rule)
    pub mutants_better: usize,
    pub mutants_with_parent: usize,
    pub mutation_channels: MutationChannelStats,
    pub elite_last_mutation: Vec<Option<MutationChannel>>,
}

impl Selection {
    /// Share of mutants that beat their parent; `None` without such mutants.
    pub fn child_success_share(&self) -> Option<f32> {
        (self.mutants_with_parent > 0).then(|| self.mutants_better as f32 / self.mutants_with_parent as f32)
    }

    /// `k` after the 1/5 success rule, clamped to `[lo, hi]`.
    pub fn adapt_mutation_ops(&self, k: u32, lo: u32, hi: u32) -> u32 {
        one_fifth_update(k, self.mutants_better, self.mutants_with_parent, lo, hi)
    }
}

/// Parents, hall of fame and the lineage of the current generation's slots.
#[derive(Debug, Clone)]
pub struct Breeder {
    /// Elites of the previous generation, fitness descending
    parents: Vec<GenomeRecord>,
    /// Channel of the last mutation in each parent's lineage, aligned with `parents`
    parent_last_mutation: Vec<Option<MutationChannel>>,
    /// Best genomes of the run, fitness descending, at most `hof_retention`
    hof: Vec<GenomeRecord>,
    hof_retention: usize,
    /// Parent index per breeding slot (elites: their own parent, mutants: their source parent, hof/random: none);
    /// aligned with the sandboxes until they are ranked at generation end
    slot_parents: Vec<Option<usize>>,
    /// Breeding record per slot, aligned like `slot_parents`
    slot_lineage: Vec<SlotLineage>,
}

impl Breeder {
    /// Continues from `parents` (fitness descending) and `hof` (sorted and cut to `hof_retention` here).
    pub fn new(parents: Vec<GenomeRecord>, mut hof: Vec<GenomeRecord>, hof_retention: usize) -> Self {
        sort_desc_by_fitness(&mut hof);
        hof.truncate(hof_retention);
        Self { parent_last_mutation: vec![None; parents.len()], parents, hof, hof_retention, slot_parents: Vec::new(), slot_lineage: Vec::new() }
    }

    pub fn parents(&self) -> &[GenomeRecord] {
        &self.parents
    }

    pub fn hof(&self) -> &[GenomeRecord] {
        &self.hof
    }

    /// Lineage of slot `slot` of the current generation.
    pub fn slot_lineage(&self, slot: usize) -> Option<(Option<usize>, SlotLineage)> {
        Some((*self.slot_parents.get(slot)?, *self.slot_lineage.get(slot)?))
    }

    /// Slot plan: elites (unmodified) | hof (mutated like mutants, TCK-174) | random immigrants | mutants of the
    /// parents round-robin. Missing parents or hof entries fall back to mutants of parent 0; without parents every slot
    /// is a structured random genome (generation 1).
    fn plan(&self, p: &TrainerParams) -> Vec<SlotPlan<'_>> {
        let n = p.population;
        let elites = elite_count(n, p.elite_ratio);
        let hof_count = ((n as f32) * p.hof_rate).round() as usize;
        let inflow_count = ((n as f32) * p.inflow_rate).round() as usize;
        (0..n)
            .map(|idx| -> SlotPlan<'_> {
                if self.parents.is_empty() {
                    (WILDTYPE_GENOME, SandboxOrigin::Random, None)
                } else if idx < elites {
                    if idx < self.parents.len() {
                        (&self.parents[idx].genome, SandboxOrigin::Elite, Some(idx))
                    } else {
                        (&self.parents[0].genome, SandboxOrigin::Mutant, Some(0))
                    }
                } else if idx < elites + hof_count {
                    let hof_idx = idx - elites;
                    if hof_idx < self.hof.len() {
                        (&self.hof[hof_idx].genome, SandboxOrigin::Hof, None)
                    } else {
                        (&self.parents[0].genome, SandboxOrigin::Mutant, Some(0))
                    }
                } else if idx < elites + hof_count + inflow_count {
                    (WILDTYPE_GENOME, SandboxOrigin::Random, None)
                } else {
                    let source_idx = (idx - elites - hof_count - inflow_count) % self.parents.len();
                    (&self.parents[source_idx].genome, SandboxOrigin::Mutant, Some(source_idx))
                }
            })
            .collect()
    }

    /// Breeds the sandboxes of `generation` in `room`. Every slot breeds with its own stream
    /// `derive(seed, [generation, BREED, slot])` and starts on the shared layout of trial 0 (`streams::layout`), so
    /// `parallel` (rayon) never changes the result.
    #[allow(clippy::too_many_arguments)]
    pub fn breed(&mut self, p: &TrainerParams, seed: u64, generation: u32, layout_id: u64, room: &Arc<dyn Room>, config: &AppConfig, parallel: bool) -> Vec<TrainerSandbox> {
        let plan = self.plan(p);
        let generation = generation as u64;
        let (parents, lamarckian) = (&self.parents, p.lamarckian);
        let (ops_min, ops_max, minimal_criterion) = (p.mutation_ops_min, p.mutation_ops_max, p.minimal_criterion);
        let (channel_mode, parent_last_mutation) = (p.mutation_channels, &self.parent_last_mutation);
        let brain_config = &config.brain;
        let lamarck_chance = config.genome.lamarckian_assimilation_chance;
        // Lamarckian sources: each parent's synapses (keys and genome weights) for matching by key
        let parent_synapses: Vec<Vec<CTRNNSynapse>> = if lamarckian { parents.iter().map(|r| parse_genome(&r.genome, None, None, brain_config).brain.synapses).collect() } else { Vec::new() };
        let build_slot = |(idx, &(source, origin, parent_idx)): (usize, &SlotPlan<'_>)| {
            let mut breed_rng = derive(seed, &[generation, streams::BREED, idx as u64]);
            let (genome, own) = breed_genome_traced(source, origin, ops_min, ops_max, minimal_criterion, channel_mode, &mut breed_rng, config);
            let last = match own {
                Some(touch) => Some(touch.channel()),
                None if origin == SandboxOrigin::Elite => parent_idx.and_then(|i| parent_last_mutation.get(i).copied().flatten()),
                None => None,
            };
            let lineage = SlotLineage { origin, own, last };

            // Lamarckian inheritance: the source parent's learned methylations, and per plastic synapse with the same key
            // in this sandbox's (possibly mutated) genome, with probability `lamarckianAssimilationChance`, its learned
            // weight offset (TCK-152; drawn from the slot's breed stream)
            let inherited = if lamarckian { parent_idx } else { None };
            let methylations = inherited.map(|i| &parents[i].methylations[..]).filter(|m| !m.is_empty());
            let synapse_weights = inherited.filter(|&i| !parents[i].synapse_weights.is_empty()).map(|i| {
                let child = parse_genome(&genome, None, None, brain_config).brain.synapses;
                inherit_learned_weights(&parent_synapses[i], &parents[i].synapse_weights, &child, lamarck_chance, &mut breed_rng)
            });

            // Shared layout of trial 0: heading, food and respawn seed identical for every slot
            let sandbox = TrainerSandbox::spawn((idx + 1) as u32, &genome, generation as u32, origin, room.clone(), methylations, synapse_weights, &mut streams::layout(seed, layout_id, 0), config);
            (sandbox, lineage)
        };
        let (sandboxes, lineage): (Vec<TrainerSandbox>, Vec<SlotLineage>) = if parallel {
            plan.par_iter().enumerate().map(build_slot).unzip()
        } else {
            plan.iter().enumerate().map(build_slot).unzip()
        };
        self.slot_parents = plan.iter().map(|&(_, _, parent)| parent).collect();
        self.slot_lineage = lineage;
        sandboxes
    }

    /// Per slot: `Some(beat its parent)` for mutants with a parent, else `None`. The reference of a parent is the
    /// fitness of its unmodified elite slot in this generation (same layout), else its recorded fitness.
    fn child_successes(&self, sandboxes: &[TrainerSandbox]) -> Vec<Option<bool>> {
        let slots = || sandboxes.iter().zip(&self.slot_parents);
        let parent_fitness = |p: usize| {
            slots()
                .find(|(sb, parent)| sb.origin_type == SandboxOrigin::Elite && **parent == Some(p))
                .map(|(sb, _)| sb.current_fitness)
                .unwrap_or(self.parents[p].fitness)
        };
        slots()
            .map(|(sb, parent)| if sb.origin_type == SandboxOrigin::Mutant { parent.map(|p| sb.current_fitness > parent_fitness(p)) } else { None })
            .collect()
    }

    /// Generation end: ranks `sandboxes` by their final fitness (stable, descending; `solved` per slot follows), takes
    /// the first `elite_count` as the new parents and adds them to the hall of fame. Returns the elites and the
    /// breeding statistics (child success, mutation channels, elite lineage).
    pub fn select(&mut self, sandboxes: &mut Vec<TrainerSandbox>, solved: Vec<bool>, elite_ratio: f32, generation: u32) -> Selection {
        let n = sandboxes.len();
        let successes = self.child_successes(sandboxes);
        let (better, total) = successes.iter().flatten().fold((0, 0), |(better, total), &b| (better + b as usize, total + 1));

        let mut ranked: Vec<(TrainerSandbox, bool, usize)> = std::mem::take(sandboxes).into_iter().zip(solved).enumerate().map(|(slot, (sb, solved))| (sb, solved, slot)).collect();
        ranked.sort_by(|a, b| b.0.current_fitness.partial_cmp(&a.0.current_fitness).unwrap_or(std::cmp::Ordering::Equal));
        let mut ranked_solved = Vec::with_capacity(n);
        let mut ranked_slots = Vec::with_capacity(n);
        for (sb, solved, slot) in ranked {
            sandboxes.push(sb);
            ranked_solved.push(solved);
            ranked_slots.push(slot);
        }
        self.slot_parents.clear();

        let elite_count = elite_count(n, elite_ratio);
        // Mutation channels (TCK-181): mutant slots by channel, and the lineage channel of every elite
        let lineage = std::mem::take(&mut self.slot_lineage);
        let mut is_elite = vec![false; n];
        ranked_slots.iter().take(elite_count).for_each(|&slot| is_elite[slot] = true);
        let mutants: Vec<(MutationTouch, bool, bool)> = lineage
            .iter()
            .enumerate()
            .filter(|(_, l)| l.origin == SandboxOrigin::Mutant)
            .filter_map(|(slot, l)| l.own.map(|touch| (touch, successes.get(slot).copied().flatten().unwrap_or(false), is_elite[slot])))
            .collect();
        let elite_last_mutation: Vec<Option<MutationChannel>> = ranked_slots.iter().take(elite_count).map(|&slot| lineage.get(slot).and_then(|l| l.last)).collect();
        let elites: Vec<GenomeRecord> = sandboxes
            .iter()
            .take(elite_count)
            .map(|sb| GenomeRecord {
                genome: sb.agent().genome.clone(),
                name: sb.agent().phenotype.latin_name.clone(),
                fitness: sb.current_fitness,
                generation,
                methylations: sb.agent().phenotype.methylations.clone(),
                synapse_weights: sb.agent().synapse_weights.clone(),
            })
            .collect();

        self.parents = elites.clone();
        self.parent_last_mutation = elite_last_mutation.clone();
        self.hof.extend(elites.iter().cloned());
        sort_desc_by_fitness(&mut self.hof);
        self.hof.truncate(self.hof_retention);

        Selection {
            elites,
            elite_count,
            ranked_solved,
            mutants_better: better,
            mutants_with_parent: total,
            mutation_channels: MutationChannelStats::compute(&mutants),
            elite_last_mutation,
        }
    }

    /// A slot got a creature from outside the plan (catalogue assignment): it has no parent and no lineage any more,
    /// so it does not enter the 1/5 rule, the mutation channel statistics or a parent's reference fitness.
    pub fn forget_slot(&mut self, slot: usize) {
        if let Some(parent) = self.slot_parents.get_mut(slot) {
            *parent = None;
        }
        if let Some(lineage) = self.slot_lineage.get_mut(slot) {
            lineage.own = None;
            lineage.last = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elites_solved_layout_rule() {
        assert!(elites_solved_layout(&[true, true, false, false], 2));
        assert!(!elites_solved_layout(&[true, false, true, true], 2));
        assert!(elites_solved_layout(&[true, false], 0), "at least one elite");
        assert!(!elites_solved_layout(&[], 1));
    }

    #[test]
    fn test_one_fifth_rule_direction_and_bounds() {
        assert_eq!(one_fifth_update(3, 3, 10, 1, 8), 4); // 30 % > 1/5
        assert_eq!(one_fifth_update(3, 1, 10, 1, 8), 2); // 10 % < 1/5
        assert_eq!(one_fifth_update(3, 2, 10, 1, 8), 3); // exactly 1/5
        assert_eq!(one_fifth_update(3, 0, 0, 1, 8), 3); // no mutants
        assert_eq!(one_fifth_update(8, 10, 10, 1, 8), 8);
        assert_eq!(one_fifth_update(1, 0, 10, 1, 8), 1);
        assert_eq!(one_fifth_update(12, 0, 10, 1, 8), 8);
        assert_eq!(one_fifth_update(0, 10, 10, 1, 8), 1);
    }

    #[test]
    fn test_elite_count_is_at_least_one_and_at_most_n() {
        assert_eq!(elite_count(100, 0.25), 25);
        assert_eq!(elite_count(2, 0.0), 1);
        assert_eq!(elite_count(4, 2.0), 4);
    }
}
