#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod commands;
mod db;
mod steam;
mod settings;
mod utils;

use commands::register_commands;

fn main() {
    // Initialize DB before Tauri starts
    if let Err(e) = db::init_db() {
        eprintln!("Database initialization failed: {}", e);
    }

    tauri::Builder::default()
        .invoke_handler(register_commands())
        .run(tauri::generate_context!())
        .expect("error while running SkinVolt");
}
