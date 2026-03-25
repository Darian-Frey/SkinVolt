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

    // initial fetch: price, metadata, AND FULL HISTORY
    let name = args.name.clone();
    tokio::spawn(async move {
        let _ = crate::steam::fetch::fetch_price(&name).await.map(|(p, ts)| {
            let _ = crate::steam::cache::cache_price_data(name.clone(), p, ts);
        });
        
        let mut appid = Some(730);
        let _ = crate::steam::fetch::fetch_item_details(&name).await.map(|m| {
            appid = Some(m.appid);
            let _ = crate::db::upsert_item_metadata(m);
        });

        let _ = crate::steam::fetch::fetch_price_history(name.clone(), appid).await.map(|h| {
            let _ = crate::db::bulk_add_price_history(&name, h);
        });
    });

    Ok(())
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
    let (price, timestamp) = match crate::steam::fetch::fetch_price(&args.item_name).await {
        Ok(res) => res,
        Err(e) => {
            println!("❌ [Command Error] Price failed for {}: {}", args.item_name, e);
            return Err(e);
        }
    };

    // Cache it
    let _ = crate::steam::cache::cache_price_data(args.item_name.clone(), price, timestamp);

    // 4. Background metadata fetch if missing
    let name = args.item_name.clone();
    tokio::spawn(async move {
        let mut appid = None;
        if let Ok(m_opt) = crate::db::get_item_metadata(&name) {
            if let Some(m) = m_opt {
                appid = Some(m.appid);
            } else {
                let _ = crate::steam::fetch::fetch_item_details(&name).await.map(|m| {
                    appid = Some(m.appid);
                    let _ = crate::db::upsert_item_metadata(m);
                });
            }
        }

        // Backfill history if empty or sparse
        let _ = crate::steam::fetch::fetch_price_history(name.clone(), appid).await.map(|h| {
            let _ = crate::db::bulk_add_price_history(&name, h);
        });
    });

    Ok(PriceResponse {
        market_hash_name: args.item_name,
        price,
        timestamp,
    })
}

#[tauri::command]
pub fn get_item_metadata(market_hash_name: String) -> Result<Option<crate::db::ItemMetadata>, String> {
    crate::db::get_item_metadata(&market_hash_name)
}



