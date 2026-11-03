//! `SporeField` (TCK-156): the Ocean's food spores and the nutrient centres they cluster around. One place for the
//! spawn rules that `engine.rs` held three times (fresh start, reset, eaten-spore respawn) plus twice the centres.

use rand::Rng;

use crate::shared::dto::FoodSpore;
use crate::sim::systems::eat::patchy_spore_position;

/// Nutrient centres of a fresh or reset Ocean.
pub const NUTRIENT_CENTER_COUNT: usize = 12;
/// Distance of a nutrient centre from the world edge (px).
pub const NUTRIENT_CENTER_MARGIN: f32 = 1000.0;
/// Energy content of a spore (`FoodSpore::amount`, `INIT_STATE.rules.sporeEnergy`).
pub const SPORE_AMOUNT: f32 = 15.0;
/// Drift speed bound of a new spore (px/tick, each axis).
pub const SPORE_DRIFT: f32 = 0.15;
/// First spore id of a new Ocean.
pub const FIRST_SPORE_ID: u32 = 1000;
/// `FoodSpore::type_id` of a plant spore.
pub const PLANT: u32 = 1;

/// The spores of the Ocean and their nutrient centres. Spores are never deleted: eaten ones respawn
/// (`eat::respawn`), corpses relocate spores as meat (`death::decompose`).
#[derive(Debug, Clone)]
pub struct SporeField {
    pub spores: Vec<FoodSpore>,
    pub nutrient_centers: Vec<(f32, f32)>,
    /// Id of the next spawned spore
    pub next_id: u32,
}

impl Default for SporeField {
    fn default() -> Self {
        Self { spores: Vec::new(), nutrient_centers: Vec::new(), next_id: FIRST_SPORE_ID }
    }
}

impl SporeField {
    /// New nutrient centres, uniform at least `NUTRIENT_CENTER_MARGIN` inside the world.
    pub fn regenerate_centers<R: Rng + ?Sized>(&mut self, rng: &mut R, width: f32, height: f32) {
        self.nutrient_centers.clear();
        for _ in 0..NUTRIENT_CENTER_COUNT {
            let cx = rng.gen_range(NUTRIENT_CENTER_MARGIN..width - NUTRIENT_CENTER_MARGIN);
            let cy = rng.gen_range(NUTRIENT_CENTER_MARGIN..height - NUTRIENT_CENTER_MARGIN);
            self.nutrient_centers.push((cx, cy));
        }
    }

    /// Adds `count` plant spores at patchy positions around the centres, each with a slow random drift.
    pub fn spawn<R: Rng + ?Sized>(&mut self, count: usize, rng: &mut R, width: f32, height: f32) {
        for _ in 0..count {
            let (x, y) = patchy_spore_position(rng, &self.nutrient_centers, width, height);
            let vx = rng.gen_range(-SPORE_DRIFT..SPORE_DRIFT);
            let vy = rng.gen_range(-SPORE_DRIFT..SPORE_DRIFT);
            self.spores.push(FoodSpore { id: self.next_id, type_id: PLANT, x, y, amount: SPORE_AMOUNT, vx, vy });
            self.next_id += 1;
        }
    }

    /// Fresh field: spores cleared, new centres, `count` new spores (ids continue).
    pub fn reseed<R: Rng + ?Sized>(&mut self, count: usize, rng: &mut R, width: f32, height: f32) {
        self.spores.clear();
        self.regenerate_centers(rng, width, height);
        self.spawn(count, rng, width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::rng::seeded;

    #[test]
    fn test_reseed_places_spores_inside_with_fresh_ids_and_centres() {
        let mut field = SporeField::default();
        field.reseed(50, &mut seeded(1), 4000.0, 3000.0);
        assert_eq!((field.spores.len(), field.nutrient_centers.len()), (50, NUTRIENT_CENTER_COUNT));
        assert!(field.nutrient_centers.iter().all(|&(x, y)| (1000.0..3000.0).contains(&x) && (1000.0..2000.0).contains(&y)));
        assert!(field.spores.iter().all(|s| s.type_id == PLANT && s.amount == SPORE_AMOUNT && (100.0..=3900.0).contains(&s.x) && (100.0..=2900.0).contains(&s.y)));
        assert_eq!((field.spores[0].id, field.next_id), (FIRST_SPORE_ID, FIRST_SPORE_ID + 50));
        field.reseed(10, &mut seeded(2), 4000.0, 3000.0);
        assert_eq!((field.spores.len(), field.spores[0].id), (10, FIRST_SPORE_ID + 50), "ids continue across a reseed");
    }
}
