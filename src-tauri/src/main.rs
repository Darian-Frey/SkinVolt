mod commands;
mod db;
mod steam;
mod settings;
mod utils;
mod analytics;
mod alerts;
mod inventory;

use std::time::Duration;
// FIX 1: Ensure Emitter and Manager are imported correctly for Tauri v2
use tauri::{AppHandle, async_runtime, Emitter};
// FIX 2: Standardizing on tokio::time::sleep for the async loop
use tokio::time::sleep;

/// Background engine to auto-refresh prices based on subscription tier
fn start_background_polling(app_handle: AppHandle) {
    async_runtime::spawn(async move {
        loop {
            let tier = settings::settings::get_current_tier();
            
            let interval_secs = match tier.as_str() {
                "elite" => 60,      // 1 minute
                "pro"   => 300,     // 5 minutes
                "basic" => 600,     // 10 minutes
                "free"  => 3600,    // 1 hour
                _ => 3600,
            };

            println!("🔄 [Polling] Tier: {} | Cycle: {}s", tier.to_uppercase(), interval_secs);
            
            if let Ok(items) = db::get_inventory_items_internal() {
                for item_name in items {
                    if let Ok((price, ts)) = steam::fetch::fetch_price(&item_name).await {
                        let _ = steam::cache::cache_price_data(item_name, price, ts);
                    }
                    sleep(Duration::from_millis(2000)).await;
                }
            }
            
            let _ = app_handle.emit("inventory-updated", ());
            sleep(Duration::from_secs(interval_secs)).await;
        }
    });
}

fn main() {
    // Initialize the database BEFORE Tauri starts
    db::init_db().expect("Failed to initialize database");

    let tier = settings::settings::get_current_tier();
    println!("⚡ SkinVolt Booting... Current Tier: {}", tier.to_uppercase());

    tauri::Builder::default()
        .setup(|app| {
            // Start the background engine logic 
            start_background_polling(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_inventory,
            commands::get_inventory_full,
            commands::refresh_steam_data,
            commands::add_item,
            commands::get_item_metadata,
            settings::settings::get_setting,
            settings::settings::set_refresh_interval,
            settings::settings::set_currency_preference,
            settings::settings::toggle_dark_mode,
            settings::settings::dev_set_tier,
            analytics::get_item_analytics,
            analytics::get_item_history_full,
            analytics::get_top_movers,
            analytics::search_market_items
        ])
        .run(tauri::generate_context!())
        .expect("error while running SkinVolt");
}

