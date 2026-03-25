use serde::{Deserialize, Serialize};
use tauri::command;

#[command]
pub fn ping() -> &'static str {
    "pong"
}

#[command]
pub fn get_inventory() -> Result<String, String> {
    match crate::db::get_inventory() {
        Ok(items) => Ok(serde_json::to_string(&items).unwrap()),
        Err(e) => Err(e.to_string()),
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

#[command]
pub fn refresh_steam_data(args: RefreshArgs) -> Result<PriceResponse, String> {
    if args.item_name.trim().is_empty() {
        return Err("Expected non-empty item_name".into());
    }

    match crate::steam::fetch::fetch_price(&args.item_name) {
        Ok((price, timestamp)) => {
            let _ = crate::steam::cache::cache_price_data(
                args.item_name.clone(),
                price,
                timestamp,
            );

            Ok(PriceResponse {
                market_hash_name: args.item_name,
                price,
                timestamp,
            })
        }
        Err(e) => Err(e),
    }
}



