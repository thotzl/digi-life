//! Death system: whether a creature dies this tick and what the world does with it (`DeathPolicy`).

use rand::Rng;

use crate::server::ocean_config::OceanConfig;
use crate::shared::dto::{CreatureAgent, FoodSpore};
use crate::sim::{CreatureEvent, CreatureStep, World};

/// Energy a creature must keep after the bite damage of a tick to survive it (Ocean, applied after the step).
pub const POST_DAMAGE_MIN_ENERGY: f32 = 0.1;

/// What happens at death.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeathPolicy {
    /// At 0 energy the creature stops (energy and velocity 0) and does nothing else; the owner keeps it frozen for
    /// the rest of the trial (trainer starvation, TCK-178). No age limit.
    Freeze,
    /// At 0 energy or at the lifespan `min(mature_age · lifespan_multiplier, max_age_ticks)` the creature dies and
    /// decomposes into `floor(length · radius / size_ratio)` meat spores clamped to `[spore_min, spore_max]` (Ocean).
    Decompose { lifespan_multiplier: u32, max_age_ticks: u32, size_ratio: f32, spore_min: i32, spore_max: i32 },
}

impl DeathPolicy {
    /// The Ocean's lifespan and decomposition rules (config section `ocean`).
    pub fn decompose(ocean: &OceanConfig) -> Self {
        DeathPolicy::Decompose {
            lifespan_multiplier: ocean.creature_lifespan_mature_age_multiplier,
            max_age_ticks: ocean.creature_max_age_ticks,
            size_ratio: ocean.decomposition_size_ratio,
            spore_min: ocean.decomposition_spore_min,
            spore_max: ocean.decomposition_spore_max,
        }
    }

    /// Meat spores the corpse of `agent` decomposes into (0 under `Freeze`).
    pub fn spores(&self, agent: &CreatureAgent) -> i32 {
        match *self {
            DeathPolicy::Freeze => 0,
            DeathPolicy::Decompose { size_ratio, spore_min, spore_max, .. } => {
                let body = &agent.phenotype.spinal_harmonics;
                (((body.base_length * body.mean_radius) / size_ratio).floor() as i32).clamp(spore_min, spore_max)
            }
        }
    }
}

/// Checks creature `idx` after the metabolism and applies the world's death policy (`step.died` ends its tick).
pub fn death(world: &mut World, idx: usize, step: &mut CreatureStep) {
    let policy = world.settings.death;
    let agent = &mut world.creatures[idx];
    match policy {
        DeathPolicy::Freeze => {
            if agent.energy <= 0.0 {
                agent.energy = 0.0;
                agent.vx = 0.0;
                agent.vy = 0.0;
                step.died = true;
            }
        }
        DeathPolicy::Decompose { lifespan_multiplier, max_age_ticks, .. } => {
            let lifespan = (agent.phenotype.mature_age * lifespan_multiplier).min(max_age_ticks);
            if agent.energy <= 0.0 || agent.age >= lifespan {
                step.died = true;
                step.events.push(CreatureEvent::Decomposed { x: agent.px, y: agent.py, spores: policy.spores(agent) });
            }
        }
    }
}

/// Turns `spores` random spores of the field into meat spores scattered ±16 px around (x, y) with a slow random drift
/// (elastic relocation, the field keeps its size). Draws index, x, y, vx, vy per spore from `rng`.
pub fn decompose<R: Rng + ?Sized>(foods: &mut [FoodSpore], x: f32, y: f32, spores: i32, rng: &mut R) {
    if foods.is_empty() {
        return;
    }
    for _ in 0..spores {
        let p = rng.gen_range(0..foods.len());
        foods[p].type_id = 2;
        foods[p].x = x + rng.gen_range(-16.0..16.0);
        foods[p].y = y + rng.gen_range(-16.0..16.0);
        foods[p].vx = rng.gen_range(-0.2..0.2);
        foods[p].vy = rng.gen_range(-0.2..0.2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::rng::seeded;

    #[test]
    fn test_decompose_turns_spores_into_meat_near_the_corpse() {
        let mut foods: Vec<FoodSpore> = (0..10).map(|id| FoodSpore { id, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }).collect();
        decompose(&mut foods, 500.0, 400.0, 3, &mut seeded(1));
        let meat: Vec<&FoodSpore> = foods.iter().filter(|f| f.type_id == 2).collect();
        assert!(!meat.is_empty() && meat.len() <= 3);
        assert!(meat.iter().all(|f| (f.x - 500.0).abs() <= 16.0 && (f.y - 400.0).abs() <= 16.0));
        decompose(&mut [], 0.0, 0.0, 5, &mut seeded(1));
    }
}
