pub mod config;
pub mod persistence;
pub mod logging;
pub mod server;
pub mod sim;
pub mod biology {
    pub mod brain;
    pub mod genome;
    pub mod organs;
    pub mod phenotype;
    pub mod trainer_config;
    pub mod trainer_engine;
    pub mod trainer_sim;
    pub mod trainer_store;
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
