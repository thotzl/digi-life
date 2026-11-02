//! Movement system: the shared kinematics (`shared::physics::step_creature_kinematics`, invariant 4: thrust is
//! scaled there, never applied raw) against the world's obstacles and bounds.

use crate::shared::physics::step_creature_kinematics;
use crate::sim::{CreatureStep, TickCtx, World};

/// Moves creature `idx` with this tick's motor outputs; records the heading before the move and the wall flag.
pub fn move_creature(world: &mut World, idx: usize, ctx: &TickCtx, step: &mut CreatureStep) {
    let agent = &mut world.creatures[idx];
    step.old_heading = agent.heading_angle;
    step.hit_wall = step_creature_kinematics(agent, step.motors.thrust, step.motors.bend, ctx.physics, world.obstacles, world.width, world.height);
}
