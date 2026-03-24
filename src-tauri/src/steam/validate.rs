use serde_json::Value;

#[tauri::command]
pub fn validate_steam_response(raw: String) -> Result<String, String> {
    // TODO: Implement Phase 1 validation logic
    Ok(raw)
}
