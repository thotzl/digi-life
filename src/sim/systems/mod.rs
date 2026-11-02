//! Creature systems (TCK-200): one module per dynamic, each a plain function over the `World`. Systems listed in
//! `sim::System` run through `sim::step_creature`; `bite` and `reproduce` need Ocean state (peers, cooldown ledger,
//! id counter) and are called by the owner after the step. Contract: `wiki/contracts/creature-system.md`.

pub mod bite;
pub mod death;
pub mod eat;
pub mod food_drift;
pub mod metabolize;
pub mod r#move;
pub mod reproduce;
pub mod sense;
pub mod think;
