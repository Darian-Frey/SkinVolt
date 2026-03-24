use crate::db::get_db;
use rusqlite::params;

#[tauri::command]
pub fn log_error(message: String, context: Option<String>) -> Result<(), String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    let timestamp = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO logs (message, context, timestamp)
         VALUES (?1, ?2, ?3)",
        params![message, context, timestamp],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

