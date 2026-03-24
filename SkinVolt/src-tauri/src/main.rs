#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod commands;
mod db;
mod steam;
mod settings;
mod utils;

use commands::register_commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(register_commands())
        .run(tauri::generate_context!())
        .expect("error while running SkinVolt");
}
