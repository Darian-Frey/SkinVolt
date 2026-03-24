use serde_json::Value;

#[tauri::command]
pub fn validate_steam_response(raw: String) -> Result<String, String> {
    // Check for Cloudflare / HTML errors
    if raw.trim_start().starts_with("<") {
        return Err("Steam returned HTML instead of JSON (Cloudflare or rate limit)".into());
    }

    // Parse JSON
    let json: Value = serde_json::from_str(&raw)
        .map_err(|_| "Invalid JSON from Steam".to_string())?;

    // Basic structural validation
    if json.get("success").is_none() {
        return Err("Missing 'success' field in Steam response".into());
    }

    // Optional: ensure price fields exist
    if let Some(listing) = json.get("lowest_price") {
        if listing.is_null() {
            return Err("Steam returned null price".into());
        }
    }

    Ok(raw)
}

