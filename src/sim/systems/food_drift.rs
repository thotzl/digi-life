//! Food drift system: spores drift and are pushed by creatures (`shared::physics::step_food_spore_physics`).

use crate::shared::physics::{SporePusher, step_food_spore_physics};
use crate::shared::spatial_grid::{GridLayer, SpatialGrid};
use crate::sim::World;

/// Radius around a spore in which creatures push it (grid query of the Ocean).
pub const PUSH_QUERY_RADIUS: f32 = 80.0;

/// When the food drifts (divergence kept until TCK-202).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodDriftOrder {
    /// After each creature moved, pushed by that creature only (trainer: `System::FoodDrift` in the system list)
    AfterEachCreature,
    /// Once per tick before all creatures, pushed by the creatures near each spore (Ocean: `drift_before_creatures`)
    BeforeAllCreatures,
}

/// `System::FoodDrift`: every spore drifts once, pushed by creature `idx` after its move. No-op unless the world's
/// order is `AfterEachCreature`.
pub fn drift_after_creature(world: &mut World, idx: usize) {
    if world.settings.food_drift != FoodDriftOrder::AfterEachCreature {
        return;
    }
    let pusher = [SporePusher::from(&world.creatures[idx])];
    for pellet in world.foods.iter_mut() {
        step_food_spore_physics(pellet, &pusher, world.obstacles, world.width, world.height);
    }
}

/// Drift before the creature loop (`BeforeAllCreatures`): every spore drifts once, pushed by the creatures within
/// `PUSH_QUERY_RADIUS` in creature order. Uses the world's grid (creature layer indexed by position in
/// `world.creatures`) when present, else all creatures. `nearby` / `pushers` are reused buffers. No-op for other orders.
pub fn drift_before_creatures(world: &mut World, nearby: &mut Vec<u32>, pushers: &mut Vec<SporePusher>) {
    if world.settings.food_drift != FoodDriftOrder::BeforeAllCreatures {
        return;
    }
    for pellet in world.foods.iter_mut() {
        pushers.clear();
        match world.grid {
            Some(grid) => {
                nearby_creatures(grid, pellet.x, pellet.y, nearby);
                pushers.extend(nearby.iter().map(|&i| SporePusher::from(&world.creatures[i as usize])));
            }
            None => pushers.extend(world.creatures.iter().map(SporePusher::from)),
        }
        step_food_spore_physics(pellet, pushers, world.obstacles, world.width, world.height);
    }
}

/// Creature indices near (x, y), sorted so the push sequence keeps creature order.
fn nearby_creatures(grid: &SpatialGrid, x: f32, y: f32, nearby: &mut Vec<u32>) {
    nearby.clear();
    grid.for_each_nearby(GridLayer::Creatures, x, y, PUSH_QUERY_RADIUS, |i| nearby.push(i));
    nearby.sort_unstable();
}
