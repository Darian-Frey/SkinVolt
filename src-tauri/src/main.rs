mod commands;
mod db;
mod steam;
mod settings;
mod utils;

fn main() {
    // Initialize the database BEFORE Tauri starts
    db::init_db().expect("Failed to initialize database");
    println!("DB PATH = {:?}", db::db_path());


    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_inventory,
            commands::refresh_steam_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running SkinVolt");
}


