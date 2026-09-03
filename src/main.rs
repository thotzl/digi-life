// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use std::sync::mpsc::{channel, Sender, Receiver};

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
            let window = app.get_webview_window("main").expect("Failed to locate main window");

            // Start parallel modularized headless simulation and replication thread
            spawn_simulation_thread(window, rx);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
