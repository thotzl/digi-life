//! Bite system (predation, TCK-145): at most one bite per attacker and tick on the nearest eligible peer in range.
//! The damage is collected in a damage map and applied by the owner after the step, so all attackers of a tick see the
//! same victims (`sim::ocean::OceanSim::step`).

use std::collections::HashMap;

use crate::biology::metabolism::{MetabolismConfig, bite_energy_gain};
use crate::config::OceanConfig;
use crate::shared::dto::CreatureAgent;
use crate::shared::geom;
use crate::sim::PeerSnapshot;

/// Who bites and how far (config section `ocean`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiteRule {
    /// Carnivory at or above which a creature bites
    pub carnivory_threshold: f32,
    /// Bite range `mean_radius · radius_multiplier + radius_offset`
    pub radius_multiplier: f32,
    pub radius_offset: f32,
}

impl BiteRule {
    pub fn from_ocean(ocean: &OceanConfig) -> Self {
        Self {
            carnivory_threshold: ocean.biting_carnivory_threshold,
            radius_multiplier: ocean.biting_radius_multiplier,
            radius_offset: ocean.biting_radius_offset,
        }
    }

    pub fn range(&self, mean_radius: f32) -> f32 {
        mean_radius * self.radius_multiplier + self.radius_offset
    }
}

/// A bite of this tick (for the owner's events and the cooldown ledger).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiteEvent {
    pub attacker: u32,
    pub victim: u32,
    pub gain: f32,
    /// Victim position (shockwave ring)
    pub x: f32,
    pub y: f32,
}

/// Bite target of an attacker: the nearest candidate within `bite_range` that is not the attacker, not of its species
/// and not bitten by this attacker within the cooldown (`bitten_recently(victim_id)`). One target at most.
pub fn bite_target<'a>(
    attacker: &CreatureAgent,
    candidates: impl IntoIterator<Item = &'a PeerSnapshot>,
    bite_range: f32,
    bitten_recently: impl Fn(u32) -> bool,
) -> Option<&'a PeerSnapshot> {
    let mut target: Option<(&PeerSnapshot, f32)> = None;
    for victim in candidates {
        if victim.id == attacker.id || victim.species_id == attacker.species_id || bitten_recently(victim.id) {
            continue;
        }
        let dist = geom::dist(attacker.px, attacker.py, victim.px, victim.py);
        if dist <= bite_range && target.is_none_or(|(_, best)| dist < best) {
            target = Some((victim, dist));
        }
    }
    target.map(|(victim, _)| victim)
}

/// The attacker bites its target (if it is a biter and has one): the victim's damage is added to `damage_map`, the
/// attacker gains `bite_energy_gain` capped by what the victim has left after earlier bites this tick and by the
/// stomach, and is marked as fed.
pub fn bite<'a>(
    attacker: &mut CreatureAgent,
    candidates: impl IntoIterator<Item = &'a PeerSnapshot>,
    rule: &BiteRule,
    metabolism: &MetabolismConfig,
    damage_map: &mut HashMap<u32, f32>,
    bitten_recently: impl Fn(u32) -> bool,
) -> Option<BiteEvent> {
    if attacker.phenotype.carnivory < rule.carnivory_threshold {
        return None;
    }
    let range = rule.range(attacker.phenotype.spinal_harmonics.mean_radius);
    let victim = bite_target(attacker, candidates, range, bitten_recently)?;
    let damage_taken = damage_map.entry(victim.id).or_insert(0.0);
    let remaining = victim.energy - *damage_taken;
    *damage_taken += metabolism.biting_energy_damage;
    let gain = bite_energy_gain(attacker.phenotype.carnivory, remaining, metabolism);
    attacker.energy = (attacker.energy + gain).min(attacker.phenotype.stomach_capacity);
    attacker.has_eaten = true;
    Some(BiteEvent {
        attacker: attacker.id,
        victim: victim.id,
        gain,
        x: victim.px,
        y: victim.py,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::creature_factory::{SpawnOptions, spawn_creature};
    use crate::biology::genome::WILDTYPE_GENOME;

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

    fn attacker() -> CreatureAgent {
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

    #[test]
    fn test_bite_target_is_nearest_eligible_victim_and_respects_cooldown() {
        let attacker = attacker();
        let peers = vec![
            peer(1, "other", 0.0),              // the attacker itself
            peer(2, &attacker.species_id, 1.0), // same species
            peer(3, "other", 8.0),
            peer(4, "other", 5.0),
            peer(5, "other", 50.0), // out of range
        ];
        let none_recent = |_: u32| false;
        assert_eq!(bite_target(&attacker, &peers, 10.0, none_recent).map(|p| p.id), Some(4));
        assert_eq!(
            bite_target(&attacker, &peers, 10.0, |id| id == 4).map(|p| p.id),
            Some(3)
        );
        assert!(bite_target(&attacker, &peers, 10.0, |id| id == 4 || id == 3).is_none());
        assert!(bite_target(&attacker, &peers, 4.0, none_recent).is_none());
    }

    #[test]
    fn test_two_attackers_cannot_take_more_than_the_victim_had() {
        let config = crate::config::test_config();
        let rule = BiteRule {
            carnivory_threshold: 0.0,
            radius_multiplier: 0.0,
            radius_offset: 10.0,
        };
        let mut victim = peer(9, "other", 5.0);
        victim.energy = config.metabolism.biting_energy_damage * 0.5;
        let mut damage = HashMap::new();
        let (mut a, mut b) = (attacker(), attacker());
        b.id = 2;
        a.phenotype.carnivory = 1.0;
        b.phenotype.carnivory = 1.0;
        let first = bite(&mut a, [&victim], &rule, &config.metabolism, &mut damage, |_| false).expect("bites");
        let second = bite(&mut b, [&victim], &rule, &config.metabolism, &mut damage, |_| false).expect("bites");
        assert!(
            first.gain <= victim.energy && second.gain == 0.0,
            "{:?} {:?}",
            first,
            second
        );
        assert_eq!(damage[&9], 2.0 * config.metabolism.biting_energy_damage);
        assert!(a.has_eaten);
        // Herbivores never bite
        let mut herbivore = attacker();
        herbivore.phenotype.carnivory = 0.0;
        let rule = BiteRule {
            carnivory_threshold: 0.6,
            ..rule
        };
        assert!(
            bite(
                &mut herbivore,
                [&victim],
                &rule,
                &config.metabolism,
                &mut damage,
                |_| false
            )
            .is_none()
        );
    }
}
