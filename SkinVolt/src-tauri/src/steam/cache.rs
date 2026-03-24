use crate::db::get_db;
use rusqlite::params;
use serde::{Serialize, Deserialize};

#[tauri::command]
pub fn cache_price_data(market_hash_name: String, price: f64, timestamp: i64) -> Result<(), String> {
    // TODO: Implement Phase 1 caching logic
    Ok(())
}

#[tauri::command]
pub fn load_cached_price(market_hash_name: String) -> Result<Option<f64>, String> {
    // TODO: Implement Phase 1 load logic
    Ok(None)
}

#[tauri::command]
pub fn cache_price_history(market_hash_name: String, history: Vec<(i64, f64)>) -> Result<(), String> {
    // TODO: Implement Phase 1 history caching
    Ok(())
}

#[tauri::command]
pub fn prune_old_cache_entries() -> Result<(), String> {
    // TODO: Implement Phase 1 pruning logic
    Ok(())
}
