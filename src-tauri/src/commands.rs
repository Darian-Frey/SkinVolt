use serde::{Deserialize, Serialize};
use tauri::command;

#[command]
pub fn ping() -> &'static str {
    "pong"
}

#[command]
pub fn get_inventory_full() -> Result<Vec<crate::db::InventoryItemFull>, String> {
    crate::db::get_inventory_full()
}

#[tauri::command]
pub fn get_inventory() -> Result<String, String> {
    match crate::db::get_inventory() {
        Ok(items) => serde_json::to_string(&items).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()), // Ensure this matches the Result<String, String> return type
    }
}

#[derive(Deserialize)]
pub struct RefreshArgs {
    pub item_name: String,
}

#[derive(Serialize)]
pub struct PriceResponse {
    pub market_hash_name: String,
    pub price: f64,
    pub timestamp: i64,
}

#[derive(Deserialize)]
pub struct AddItemArgs {
    pub name: String,
    pub quantity: u32,
}

#[command]
pub async fn add_item(args: AddItemArgs) -> Result<(), String> {
    if args.name.trim().is_empty() {
        return Err("Item name cannot be empty".into());
    }

    // Save to DB first
    crate::db::add_inventory_item(args.name.clone(), args.quantity)?;

    // THE HOOK: Every tier, including Free, gets one initial fetch on add 
    match crate::steam::fetch::fetch_price(&args.name).await {
        Ok((price, timestamp)) => {
            let _ = crate::steam::cache::cache_price_data(args.name, price, timestamp);
            Ok(())
        }
        Err(e) => {
            println!("⚠️ Initial fetch failed: {}", e);
            Ok(()) // Item is saved, will try again on next manual refresh
        }
    }
}

#[tauri::command]
pub async fn refresh_steam_data(args: RefreshArgs) -> Result<PriceResponse, String> {
    // 1. Tier Check
    let tier = crate::settings::settings::get_current_tier();
    
    // 2. Cooldown Logic: Check database for last fetch time
    let last_fetch = crate::db::get_last_fetch_time(&args.item_name).unwrap_or(0);
    let now = chrono::Utc::now().timestamp();
    let elapsed = now - last_fetch;

    // Apply Strategic Cooldowns [cite: 5, 7]
    match tier.as_str() {
        "free" => {
            // Only block if we have a successful previous fetch (last_fetch > 0) 
            if last_fetch != 0 && elapsed < 3600 { // 1 Hour limit 
                let remaining = (3600 - elapsed) / 60;
                return Err(format!("Free Tier limit: Please wait {} more minutes.", remaining));
            }
        },
        "basic" => {
            if last_fetch != 0 && elapsed < 600 {
                let remaining = (600 - elapsed) / 60;
                return Err(format!("Basic Tier limit: Please wait {} more minutes.", remaining));
            }
        },
        "pro" => {
            if last_fetch != 0 && elapsed < 60 {
                let remaining = 60 - elapsed;
                return Err(format!("Pro Tier limit: Please wait {} more seconds.", remaining));
            }
        },
        _ => {} // Elite: no manual cooldown
    }

    // 3. Call the fetcher
    match crate::steam::fetch::fetch_price(&args.item_name).await {
        Ok((price, timestamp)) => {
            // Cache it for Phase 1 offline support and Phase 2 history tracking
            let _ = crate::steam::cache::cache_price_data(args.item_name.clone(), price, timestamp);
            
            Ok(PriceResponse {
                market_hash_name: args.item_name,
                price,
                timestamp,
            })
        }
        Err(e) => {
            println!("❌ [Command Error] Fetch failed for {}: {}", args.item_name, e);
            Err(e)
        }
    }
}



