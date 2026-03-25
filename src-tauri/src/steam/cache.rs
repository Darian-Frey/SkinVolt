use crate::db::get_db;
use rusqlite::{params, Result};
use serde_json;
use rusqlite::OptionalExtension;


/// Store the latest price for an item.
#[tauri::command]
pub fn cache_price_data(
    market_hash_name: String,
    price: f64,
    timestamp: i64,
) -> Result<(), String> {
    let conn = crate::db::get_db().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO price_cache (market_hash_name, price, timestamp)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(market_hash_name)
         DO UPDATE SET price = excluded.price, timestamp = excluded.timestamp",
        (market_hash_name, price, timestamp),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Load the last cached price for an item.
#[tauri::command]
pub fn load_cached_price(market_hash_name: String) -> Result<Option<f64>, String> {
    let conn = get_db().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT price FROM price_cache WHERE market_hash_name = ?1")
        .map_err(|e| e.to_string())?;

    let result = stmt
        .query_row(params![market_hash_name], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Cache historical price data (compressed JSON).
#[tauri::command]
pub fn cache_price_history(
    market_hash_name: String,
    history: Vec<(i64, f64)>
) -> Result<(), String> {
    let conn = get_db().map_err(|e| e.to_string())?;

    let json = serde_json::to_string(&history).map_err(|e| e.to_string())?;
    let timestamp = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO price_history (market_hash_name, history_json, last_updated)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(market_hash_name)
         DO UPDATE SET history_json = excluded.history_json,
                       last_updated = excluded.last_updated",
        params![market_hash_name, json, timestamp],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Remove cache entries older than 30 days.
#[tauri::command]
pub fn prune_old_cache_entries() -> Result<(), String> {
    let conn = get_db().map_err(|e| e.to_string())?;

    let cutoff = chrono::Utc::now().timestamp() - (30 * 24 * 60 * 60);

    conn.execute(
        "DELETE FROM price_cache WHERE timestamp < ?1",
        params![cutoff],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
