use crate::db::get_db;
use rusqlite::params;

#[tauri::command]
pub fn set_refresh_interval(seconds: u32) -> Result<(), String> {
    // TODO: Implement Phase 1 settings write
    Ok(())
}

#[tauri::command]
pub fn set_currency_preference(currency: String) -> Result<(), String> {
    // TODO: Implement Phase 1 settings write
    Ok(())
}

#[tauri::command]
pub fn toggle_dark_mode(enabled: bool) -> Result<(), String> {
    // TODO: Implement Phase 1 theme toggle
    Ok(())
}
