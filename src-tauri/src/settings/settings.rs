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

