use crate::db::get_db;
use rusqlite::params;

#[tauri::command]
pub fn log_error(message: String, context: Option<String>) -> Result<(), String> {
    // TODO: Implement Phase 1 logging
    Ok(())
}
