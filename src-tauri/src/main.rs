mod commands;
mod db;
mod steam;
mod settings;
mod utils;

fn main() {
    // Initialize the database BEFORE Tauri starts
    db::init_db().expect("Failed to initialize database");
    println!("DB PATH = {:?}", db::db_path());

    // Debug: Check the tier on startup
    let tier = settings::settings::get_current_tier();
    println!("⚡ SkinVolt Booting... Current Tier: {}", tier.to_uppercase());

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_inventory,
            commands::refresh_steam_data,
            commands::add_item,
            settings::settings::get_setting,
            settings::settings::set_refresh_interval,
            settings::settings::set_currency_preference,
            settings::settings::toggle_dark_mode,
            settings::settings::dev_set_tier
        ])
        .run(tauri::generate_context!())
        .expect("error while running SkinVolt");
}


