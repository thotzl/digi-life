// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Mutex;

use src_tauri::config::AppConfig;
use src_tauri::persistence;
use src_tauri::server::api;
use src_tauri::server::engine::spawn_simulation_thread;

fn main() {
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    tauri::Builder::default()
        .manage(tx)
        .invoke_handler(tauri::generate_handler![
            api::handle_client_action,
            api::get_registered_species,
            api::get_trainer_runs,
            api::get_trainer_population,
            api::get_trainer_hof,
            api::save_trainer_generation,
            api::clear_trainer_history,
            api::apply_champion,
            api::get_fossil_phenotype,
            api::get_catalogue_creatures,
            api::save_to_catalogue,
            api::delete_from_catalogue,
            api::rename_catalogue_creature,
            api::spawn_catalogue_creature_to_ocean,
            api::add_catalogue_creature_to_training
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            // Release builds log to <app data>/logs/broodbattle.log (no console with windows_subsystem), dev builds to stderr
            src_tauri::logging::init_app(&data_dir.join("logs"));
            // Entry point: the one place the app reads the process config; commands get it as managed state and the
            // engine thread owns a copy
            let config = AppConfig::global().clone();
            app.manage(config.clone());

            // Persist the database in the per-user app data dir instead of the working directory
            let db_path = data_dir.join(persistence::DB_FILE_NAME);
            match persistence::migrate_legacy_db(std::path::Path::new(persistence::LEGACY_DB_PATH), &db_path) {
                Ok(true) => log::info!("copied legacy database '{}' to '{}'", persistence::LEGACY_DB_PATH, db_path.display()),
                Ok(false) => {}
                Err(e) => log::error!("legacy database migration failed: {}", e),
            }
            log::info!("using database '{}'", db_path.display());
            persistence::set_db_path(db_path.clone());
            let conn = persistence::init_db(&db_path)?;
            app.manage(persistence::DbState(Mutex::new(conn)));

            let window = app.get_webview_window("main").expect("Failed to locate main window");

            // Start parallel modularized headless simulation and replication thread
            spawn_simulation_thread(window, rx, config);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
