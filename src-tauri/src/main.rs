#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]


mod db;
mod steam;
mod settings;
mod utils;
mod commands;


fn main() {
    tauri::Builder::default()
        .invoke_handler(commands::register())
        .run(tauri::generate_context!())
        .expect("error while running SkinVolt");
}
