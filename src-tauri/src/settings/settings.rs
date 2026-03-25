use crate::db::get_db;
use rusqlite::params;

/// Helper: write a key/value setting.
fn write_setting(key: &str, value: &str) -> Result<(), String> {
    let conn = get_db().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO settings (key, value)
         VALUES (?1, ?2)
         ON CONFLICT(key)
         DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Internal utility to check the user's current tier
pub fn get_current_tier() -> String {
    let conn = get_db().expect("Failed to open DB for tier check");
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = 'tier_level'")
        .expect("Failed to prepare tier query");
    
    stmt.query_row([], |row| row.get(0))
        .unwrap_or_else(|_| "basic".to_string())
}

/// Helper to verify if a feature is allowed for the current tier
pub fn is_feature_allowed(required_tier: &str) -> bool {
    let current = get_current_tier();
    match (current.as_str(), required_tier) {
        ("elite", _) => true, // Elite access everything
        ("pro", "elite") => false,
        ("pro", _) => true,    // Pro access basic and pro
        ("basic", "basic") => true,
        _ => false,
    }
}

#[tauri::command]
pub fn get_setting(key: String) -> Result<String, String> {
    let conn = get_db().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;

    let value: String = stmt
        .query_row([key], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    Ok(value)
}

#[tauri::command]
pub fn dev_set_tier(tier: String) -> Result<(), String> {
    // Only allow valid tiers from your strategic model [cite: 131]
    let valid_tiers = vec!["free", "basic", "pro", "elite"];
    if !valid_tiers.contains(&tier.to_lowercase().as_str()) {
        return Err("Invalid tier specified".into());
    }

    write_setting("tier_level", &tier.to_lowercase())?;
    println!("🛠️ DEV MODE: Tier shifted to {}", tier.to_uppercase());
    Ok(())
}

#[tauri::command]
pub fn set_refresh_interval(seconds: u32) -> Result<(), String> {
    write_setting("refresh_interval", &seconds.to_string())
}

#[tauri::command]
pub fn set_currency_preference(currency: String) -> Result<(), String> {
    write_setting("currency", &currency)
}

#[tauri::command]
pub fn toggle_dark_mode(enabled: bool) -> Result<(), String> {
    write_setting("dark_mode", if enabled { "true" } else { "false" })
}

