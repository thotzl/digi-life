//! Sensing system: picks the targets the sensors see (`TargetSelector`) and fills the sensory inputs with the shared
//! sensory compiler (`shared::brain::compute_sensory_inputs_into`, organ kinds: `wiki/contracts/organ-kind.md`).

use crate::shared::brain::compute_sensory_inputs_into;
use crate::shared::dto::{CreatureAgent, FoodSpore};
use crate::shared::{fmath, geom};

use crate::sim::{CreatureStep, PeerSnapshot, World};

/// Position of an absent target: far outside every world, so no organ registers it.
pub const ABSENT_POS: f32 = -99999.0;

/// Placeholder for a target that does not exist (sensor layout keeps `foods[0]` plant, `foods[1]` meat).
pub fn absent_target(id: u32, type_id: u32) -> FoodSpore {
    FoodSpore {
        id,
        type_id,
        x: ABSENT_POS,
        y: ABSENT_POS,
        amount: 0.0,
        vx: 0.0,
        vy: 0.0,
    }
}

/// `sin` used by the clock input `0.5 + 0.5 · sin(age · 0.1)`. The trainer uses the platform-independent
/// `fmath::sin`, the Ocean the std `sin` (divergence kept until TCK-202).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockVariant {
    Fmath,
    Std,
}

impl ClockVariant {
    pub fn clock_input(self, age: u32) -> f32 {
        let phase = (age as f32) * 0.1;
        match self {
            ClockVariant::Fmath => 0.5 + 0.5 * fmath::sin(phase),
            ClockVariant::Std => 0.5 + 0.5 * phase.sin(),
        }
    }
}

/// Which food the sensors see. Every selector but `All` yields exactly two targets: `[0]` the nearest plant,
/// `[1]` the nearest meat (absent targets at `ABSENT_POS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetSelector {
    /// All food as it is (one food per type, or a fixed probe layout)
    All,
    /// Nearest plant (`type_id` 1) and nearest meat (`type_id` 2) (trainer rooms with several food per type)
    NearestPerType,
    /// Nearest plant (every non-meat spore) and the nearer of the nearest meat spore and the nearest prey peer
    /// (another creature of another species) (Ocean)
    NearestPerTypeWithPrey,
}

impl TargetSelector {
    /// Targets of `agent`; `scratch` is a reused buffer for targets built per tick.
    pub fn select<'a>(
        self,
        agent: &CreatureAgent,
        foods: &'a [FoodSpore],
        peers: &[PeerSnapshot],
        scratch: &'a mut Vec<FoodSpore>,
    ) -> &'a [FoodSpore] {
        match self {
            TargetSelector::All => foods,
            TargetSelector::NearestPerType => {
                let plant = nearest(agent, foods.iter().filter(|f| f.type_id == 1));
                let meat = nearest(agent, foods.iter().filter(|f| f.type_id == 2));
                scratch.clear();
                scratch.push(plant.map(|(f, _)| f.clone()).unwrap_or_else(|| absent_target(1, 1)));
                scratch.push(meat.map(|(f, _)| f.clone()).unwrap_or_else(|| absent_target(2, 2)));
                scratch
            }
            TargetSelector::NearestPerTypeWithPrey => {
                let plant = nearest(agent, foods.iter().filter(|f| f.type_id != 2));
                let meat = nearest(agent, foods.iter().filter(|f| f.type_id == 2));
                let mut prey: Option<(&PeerSnapshot, f32)> = None;
                for peer in peers {
                    if peer.id == agent.id || peer.species_id == agent.species_id {
                        continue;
                    }
                    let d = geom::dist_sq(agent.px, agent.py, peer.px, peer.py);
                    if prey.map_or(d < f32::MAX, |(_, best)| d < best) {
                        prey = Some((peer, d));
                    }
                }
                let meat_dist = meat.map_or(f32::MAX, |(_, d)| d);
                let prey_dist = prey.map_or(f32::MAX, |(_, d)| d);
                let meat_target = if meat_dist < prey_dist {
                    meat.map(|(f, _)| f.clone()).unwrap_or_else(|| absent_target(2, 2))
                } else {
                    prey.map(|(p, _)| FoodSpore {
                        id: p.id,
                        type_id: 2,
                        x: p.px,
                        y: p.py,
                        amount: 15.0,
                        vx: p.vx,
                        vy: p.vy,
                    })
                    .unwrap_or_else(|| absent_target(2, 2))
                };
                scratch.clear();
                scratch.push(plant.map(|(f, _)| f.clone()).unwrap_or_else(|| absent_target(1, 1)));
                scratch.push(meat_target);
                scratch
            }
        }
    }
}

/// First food with the smallest squared distance (strict `<`, so ties keep the earlier food).
fn nearest<'a>(agent: &CreatureAgent, foods: impl Iterator<Item = &'a FoodSpore>) -> Option<(&'a FoodSpore, f32)> {
    let mut best: Option<(&FoodSpore, f32)> = None;
    for f in foods {
        let d = geom::dist_sq(agent.px, agent.py, f.x, f.y);
        if best.map_or(d < f32::MAX, |(_, b)| d < b) {
            best = Some((f, d));
        }
    }
    best
}

/// Fills `step.inputs` for creature `idx`: clock input, the world's targets, obstacles and bounds.
pub fn sense(world: &mut World, idx: usize, step: &mut CreatureStep) {
    let agent = &world.creatures[idx];
    let clock = world.settings.clock.clock_input(agent.age);
    let visible = world
        .settings
        .targets
        .select(agent, world.foods, world.peers, &mut step.visible);
    compute_sensory_inputs_into(
        agent,
        clock,
        visible,
        world.obstacles,
        world.width,
        world.height,
        &mut step.inputs,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::creature_factory::{SpawnOptions, spawn_creature};
    use crate::biology::genome::WILDTYPE_GENOME;

    fn agent() -> CreatureAgent {
        spawn_creature(
            WILDTYPE_GENOME,
            SpawnOptions {
                id: 1,
                px: 0.0,
                py: 0.0,
                heading_angle: 0.0,
                generation: 1,
                methylations: None,
                synapse_weights: None,
            },
            crate::config::test_config(),
        )
    }

    fn food(id: u32, type_id: u32, x: f32) -> FoodSpore {
        FoodSpore {
            id,
            type_id,
            x,
            y: 0.0,
            amount: 15.0,
            vx: 0.0,
            vy: 0.0,
        }
    }

    fn peer(id: u32, species: &str, px: f32) -> PeerSnapshot {
        PeerSnapshot {
            id,
            species_id: species.to_string(),
            px,
            py: 0.0,
            vx: 0.0,
            vy: 0.0,
            energy: 50.0,
        }
    }

    #[test]
    fn test_nearest_per_type_picks_the_nearest_of_each_type_and_absent_placeholders() {
        let a = agent();
        let foods = [food(1, 1, 50.0), food(2, 1, 10.0), food(3, 2, 30.0), food(4, 2, 20.0)];
        let mut scratch = Vec::new();
        let seen: Vec<u32> = TargetSelector::NearestPerType
            .select(&a, &foods, &[], &mut scratch)
            .iter()
            .map(|f| f.id)
            .collect();
        assert_eq!(seen, vec![2, 4]);
        let only_plants = [food(1, 1, 5.0)];
        let seen = TargetSelector::NearestPerType
            .select(&a, &only_plants, &[], &mut scratch)
            .to_vec();
        assert_eq!((seen[1].id, seen[1].type_id, seen[1].x), (2, 2, ABSENT_POS));
        assert_eq!(TargetSelector::All.select(&a, &foods, &[], &mut scratch).len(), 4);
    }

    #[test]
    fn test_prey_selector_takes_the_nearer_of_meat_spore_and_foreign_peer() {
        let a = agent();
        let foods = [food(1, 1, 50.0), food(2, 2, 30.0)];
        let peers = [
            peer(1, "other", 1.0),
            peer(7, &a.species_id, 2.0),
            peer(9, "other", 20.0),
        ];
        let mut scratch = Vec::new();
        let seen = TargetSelector::NearestPerTypeWithPrey
            .select(&a, &foods, &peers, &mut scratch)
            .to_vec();
        // Self and own species are no prey; peer 9 at 20 px beats the meat spore at 30 px
        assert_eq!((seen[0].id, seen[1].id, seen[1].x, seen[1].amount), (1, 9, 20.0, 15.0));
        let seen = TargetSelector::NearestPerTypeWithPrey
            .select(&a, &foods, &peers[..2], &mut scratch)
            .to_vec();
        assert_eq!(seen[1].id, 2);
        let seen = TargetSelector::NearestPerTypeWithPrey
            .select(&a, &[], &[], &mut scratch)
            .to_vec();
        assert_eq!((seen[0].x, seen[1].x), (ABSENT_POS, ABSENT_POS));
    }

    #[test]
    fn test_clock_variants_agree_at_age_zero() {
        assert_eq!(ClockVariant::Fmath.clock_input(0), 0.5);
        assert_eq!(ClockVariant::Std.clock_input(0), 0.5);
        assert!((ClockVariant::Fmath.clock_input(17) - ClockVariant::Std.clock_input(17)).abs() < 1e-6);
    }
}
