#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]


mod db;
mod steam;
mod settings;
mod utils;


fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running SkinVolt");
}
