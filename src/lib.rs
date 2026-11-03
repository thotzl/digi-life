pub mod config;
pub mod persistence;
pub mod logging;
pub mod server;
pub mod sim;
pub mod trainer;
/// Diagnosis probes and the headless trainer report (TCK-201): compiled for tests and the `runner` feature only, not
/// into the game library.
#[cfg(any(test, feature = "runner"))]
pub mod diagnostics {
    pub mod probes;
    pub mod report;
}
pub mod biology {
    pub mod brain;
    pub mod genome;
    pub mod organs;
    pub mod phenotype;
    pub mod trainer_config;
    pub mod trainer_store;
    pub mod creature_factory;
    pub mod metabolism;
    /// Old path of the training modes and the room trait (moved to `crate::trainer`, TCK-201). Kept only for
    /// `server/engine.rs` until it imports `crate::trainer` (TCK-156); do not use in new code.
    pub mod scenarios {
        pub use crate::trainer::mode::Scenario;
        pub use crate::trainer::room::Room as TrainingScenarioPlugin;
    }
    /// Old path of `TrainerSim` (moved to `crate::trainer::sim`, TCK-201). Kept only for `server/engine.rs` until it
    /// imports `crate::trainer` (TCK-156); do not use in new code.
    pub mod trainer_sim {
        pub use crate::trainer::sim::*;
    }
}
pub mod shared {
    pub mod dto;
    pub mod geom;
    pub mod json_col;
    pub mod time;
    pub mod spatial_grid;
    pub mod physics;
    pub mod brain;
    pub mod map_generator;
    pub mod rng;
    pub mod fmath;
    pub mod stats;
}
