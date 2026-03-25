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
            // 1. Check the Gatekeeper for the current tier [cite: 3]
            let tier = settings::settings::get_current_tier();
            
            // 2. Set interval based on the 4-tier model
            let interval_secs = match tier.as_str() {
                "elite" => 60,      // 1 minute [cite: 13]
                "pro"   => 3600,    // 1 hour [cite: 10]
                "basic" => 86400,   // 24 hours (Daily) [cite: 7]
                _ => 0,             // Free tier: Manual/Startup sync only 
            };

            if interval_secs > 0 {
                println!("🔄 [Polling] Tier: {} | Cycle: {}s", tier.to_uppercase(), interval_secs);
                
                if let Ok(items) = db::get_inventory_items_internal() {
                    for item_name in items {
                        println!("📡 [Auto-Refresh] Updating: {}", item_name);
                        let _ = steam::fetch::fetch_price(&item_name).await;
                        
                        // Prevent Steam 429 Rate Limits
                        sleep(Duration::from_millis(2000)).await;
                    }
                }
                
                // 4. Emit event to frontend to refresh the table UI
                let _ = app_handle.emit("inventory-updated", ());
                
                // Sleep until the next scheduled cycle
                sleep(Duration::from_secs(interval_secs)).await;

            } else if tier == "free" {
                // ONE-TIME BOOT SYNC: Fetch once 
                println!("🔄 [Free Tier] Performing initial startup sync...");
                if let Ok(items) = db::get_inventory_items_internal() {
                    for name in items {
                        let _ = steam::fetch::fetch_price(&name).await;
                        sleep(Duration::from_millis(2000)).await;
                    }
                }
                let _ = app_handle.emit("inventory-updated", ());
                
                // Sleep the Free tier loop for an hour to enforce the cooldown 
                sleep(Duration::from_secs(3600)).await;
            } else {
                // Safety sleep for undefined tiers
                sleep(Duration::from_secs(60)).await;
            }
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

