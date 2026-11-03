pub mod config;
pub mod persistence;
pub mod logging;
pub mod server;
pub mod sim;
pub mod trainer;
/// Diagnosis probes, the headless trainer report (TCK-201) and the headless Ocean report (TCK-158): compiled for tests
/// and the `runner` feature only, not into the game library.
#[cfg(any(test, feature = "runner"))]
pub mod diagnostics {
    pub mod ocean_report;
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
