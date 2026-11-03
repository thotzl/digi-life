//! Eating system: food within the eat radius (`eat_radius`, one config rule for every world) is eaten (shared food
//! energy rule `metabolism::food_energy`, capped by the stomach); the owner respawns eaten food after the step with the
//! world's `RespawnPolicy`.

use rand::Rng;

use crate::biology::metabolism::{MetabolismConfig, food_energy};
use crate::shared::dto::FoodSpore;
use crate::shared::geom;
use crate::shared::rng::SimRng;
use crate::shared::spatial_grid::GridLayer;
use crate::sim::{CreatureEvent, CreatureStep, TickCtx, World};

/// Eat radius of a creature with body radius `mean_radius`: `mean_radius · grazingRadiusMultiplier +
/// grazingRadiusOffset` (config `metabolism`), the same for plants and meat and in every world (TCK-202; the trainer
/// formerly used a hard-coded `max(r + 10, ..)` rule, so trainer-bred creatures met another catch geometry in the
/// Ocean).
pub fn eat_radius(mean_radius: f32, metabolism: &MetabolismConfig) -> f32 {
    mean_radius * metabolism.grazing_radius_multiplier + metabolism.grazing_radius_offset
}

/// Where eaten food reappears.
#[derive(Debug, Clone, PartialEq)]
pub enum RespawnPolicy {
    /// Trainer: uniform in the square chamber (`margin` px from the walls, config `trainerSpawnMargin`) at least
    /// `min_distance` from the eater, at rest (`spawn_at_min_distance`); the owner passes the catch's stream
    /// (`derive(respawn_seed, [catch])`).
    MinDistance { min_distance: f32, margin: f32 },
    /// Ocean: patchy around nutrient centres (`patchy_spore_position`) with a slow random drift; meat turns back into
    /// a plant spore when `recycle_meat`.
    Patchy {
        nutrient_centers: Vec<(f32, f32)>,
        recycle_meat: bool,
    },
}

/// Upper bound for spawn rejection sampling; afterwards the farthest candidate is used.
pub const SPAWN_MAX_TRIES: u32 = 1000;

/// Random point in the square chamber (`margin` px from the walls) at least `min_dist` away from `from`.
/// After `max_tries` rejected candidates the farthest one found is returned.
pub fn spawn_at_min_distance(
    rng: &mut SimRng,
    chamber_size: f32,
    margin: f32,
    from: (f32, f32),
    min_dist: f32,
    max_tries: u32,
) -> (f32, f32) {
    let mut best = (from.0, from.1);
    let mut best_dist = -1.0;
    for _ in 0..max_tries {
        let x = margin + rng.gen_range(0.0..(chamber_size - 2.0 * margin));
        let y = margin + rng.gen_range(0.0..(chamber_size - 2.0 * margin));
        let dist = geom::dist(from.0, from.1, x, y);
        if dist >= min_dist {
            return (x, y);
        }
        if dist > best_dist {
            best = (x, y);
            best_dist = dist;
        }
    }
    best
}

/// Position of a new or respawned Ocean food spore: with probability 0.75 in a ring of 50..600 px around a random
/// nutrient centre, otherwise uniform; clamped 100 px inside the world.
pub fn patchy_spore_position<R: Rng + ?Sized>(
    rng: &mut R,
    nutrient_centers: &[(f32, f32)],
    width: f32,
    height: f32,
) -> (f32, f32) {
    let roll = rng.gen_range(0.0..1.0);
    if !nutrient_centers.is_empty() && roll < 0.75 {
        let center_idx = rng.gen_range(0..nutrient_centers.len());
        let (cx, cy) = nutrient_centers[center_idx];
        let radius = rng.gen_range(50.0..600.0);
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let (dx, dy) = geom::polar_offset(radius, angle);
        (
            (cx + dx).clamp(100.0, width - 100.0),
            (cy + dy).clamp(100.0, height - 100.0),
        )
    } else {
        (
            rng.gen_range(100.0..width - 100.0),
            rng.gen_range(100.0..height - 100.0),
        )
    }
}

/// Moves an eaten food to its respawn point. `from` is the eater's position, (width, height) the world bounds.
pub fn respawn(
    food: &mut FoodSpore,
    policy: &RespawnPolicy,
    from: (f32, f32),
    width: f32,
    height: f32,
    rng: &mut SimRng,
) {
    match policy {
        RespawnPolicy::MinDistance { min_distance, margin } => {
            let (x, y) = spawn_at_min_distance(rng, width, *margin, from, *min_distance, SPAWN_MAX_TRIES);
            food.x = x;
            food.y = y;
            food.vx = 0.0;
            food.vy = 0.0;
        }
        RespawnPolicy::Patchy {
            nutrient_centers,
            recycle_meat,
        } => {
            let (x, y) = patchy_spore_position(rng, nutrient_centers, width, height);
            let vx = rng.gen_range(-0.15..0.15);
            let vy = rng.gen_range(-0.15..0.15);
            if *recycle_meat && food.type_id == 2 {
                food.type_id = 1;
            }
            food.x = x;
            food.y = y;
            food.vx = vx;
            food.vy = vy;
        }
    }
}

/// Creature `idx` eats every food within its eat radius, in candidate order (grid order with a grid, else index
/// order): energy `food_energy` capped by the stomach, one `CreatureEvent::Ate` per food. The food stays where it is
/// until the owner respawns it, so every food is eaten at most once per tick and creature.
pub fn eat(world: &mut World, idx: usize, ctx: &TickCtx, step: &mut CreatureStep) {
    let agent = &mut world.creatures[idx];
    let radius = eat_radius(agent.phenotype.spinal_harmonics.mean_radius, ctx.metabolism);
    step.nearby.clear();
    match world.grid {
        Some(grid) => grid.for_each_nearby(GridLayer::Food, agent.px, agent.py, radius, |i| step.nearby.push(i)),
        None => step.nearby.extend(0..world.foods.len() as u32),
    }
    for &f_idx in &step.nearby {
        let Some(f) = world.foods.get(f_idx as usize) else {
            continue;
        };
        let is_meat = f.type_id == 2;
        if geom::dist(agent.px, agent.py, f.x, f.y) <= radius {
            let gain = food_energy(agent.phenotype.carnivory, is_meat, ctx.metabolism);
            agent.energy = (agent.energy + gain).min(agent.phenotype.stomach_capacity);
            step.events.push(CreatureEvent::Ate {
                food: f_idx as usize,
                is_meat,
                x: f.x,
                y: f.y,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::rng::seeded;

    #[test]
    fn test_spawn_at_min_distance_respects_limit_and_terminates() {
        let mut rng = seeded(7);
        for _ in 0..50 {
            let (x, y) = spawn_at_min_distance(&mut rng, 1000.0, 25.0, (500.0, 500.0), 200.0, SPAWN_MAX_TRIES);
            assert!(((x - 500.0).powi(2) + (y - 500.0).powi(2)).sqrt() >= 200.0);
            assert!((25.0..=975.0).contains(&x) && (25.0..=975.0).contains(&y));
        }
        // Unreachable distance: returns the farthest candidate instead of looping forever
        let (x, y) = spawn_at_min_distance(&mut rng, 1000.0, 25.0, (500.0, 500.0), 5000.0, 20);
        assert!((25.0..=975.0).contains(&x) && (25.0..=975.0).contains(&y));
    }

    #[test]
    fn test_eat_radius_is_the_config_rule() {
        let mut metabolism = crate::config::test_config().metabolism.clone();
        metabolism.grazing_radius_multiplier = 1.5;
        metabolism.grazing_radius_offset = 4.0;
        assert_eq!(eat_radius(10.0, &metabolism), 19.0);
    }

    #[test]
    fn test_patchy_respawn_recycles_meat_and_stays_inside() {
        let policy = RespawnPolicy::Patchy {
            nutrient_centers: vec![(500.0, 500.0)],
            recycle_meat: true,
        };
        let mut rng = seeded(3);
        for _ in 0..50 {
            let mut f = FoodSpore {
                id: 1,
                type_id: 2,
                x: 0.0,
                y: 0.0,
                amount: 15.0,
                vx: 0.0,
                vy: 0.0,
            };
            respawn(&mut f, &policy, (0.0, 0.0), 2000.0, 1500.0, &mut rng);
            assert_eq!(f.type_id, 1);
            assert!(
                (100.0..=1900.0).contains(&f.x) && (100.0..=1400.0).contains(&f.y),
                "{:?}",
                (f.x, f.y)
            );
            assert!(f.vx.abs() < 0.15 && f.vy.abs() < 0.15);
        }
    }
}
