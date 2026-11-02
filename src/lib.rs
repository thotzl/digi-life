pub mod config;
pub mod database;
pub mod logging;
pub mod server;
pub mod biology {
    pub mod dna;
    pub mod trainer_config;
    pub mod trainer_engine;
    pub mod trainer_sim;
    pub mod trainer_report;
    pub mod scenarios;
    pub mod creature_factory;
    pub mod metabolism;
    pub mod probes;
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
}
