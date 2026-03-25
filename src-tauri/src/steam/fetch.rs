use serde_json::Value;
use tokio::time::{sleep, Duration};

#[tauri::command]
pub async fn fetch_inventory_prices(items: Vec<String>) -> Result<(), String> {
    for item in items {
        // 1. Check if we already have a fresh price (Phase 1 Cache)
        // 2. If not, fetch from Steam
        match fetch_single_price(&item) {
            Ok(_) => {
                // Mandatory 4-second delay to respect Valve's limits
                sleep(Duration::from_secs(4)).await; 
            }
            Err(e) => println!("Error fetching {}: {}", item, e),
        }
    }
    Ok(())
}

pub fn fetch_price(market_hash_name: &str) -> Result<(f64, i64), String> {
    fetch_single_price(market_hash_name)
}

pub fn fetch_single_price(market_hash_name: &str) -> Result<(f64, i64), String> {
    // 1. Construct the Steam API URL for CS2 (AppID 730) [cite: 78]
    let url = format!(
        "https://steamcommunity.com/market/priceoverview/?currency=1&appid=730&market_hash_name={}",
        urlencoding::encode(market_hash_name)
    );

    // --- DIAGNOSTIC LOGS ---
    println!("🔍 [SkinVolt] Fetching: {}", market_hash_name);
    println!("🌐 [SkinVolt] URL: {}", url);

    // 2. Perform the GET request [cite: 78]
    let client = reqwest::blocking::Client::new();
    let resp = client.get(&url)
        .header("User-Agent", "SkinVolt/1.0") // Adding a User-Agent can help bypass some basic blocks
        .send()
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    println!("📡 [SkinVolt] HTTP Status: {}", status);

    let text = resp.text().map_err(|e| format!("Failed to read response: {}", e))?;
    
    // Check for Cloudflare / HTML errors manually for the terminal log [cite: 13, 101]
    if text.trim_start().starts_with('<') {
        println!("⚠️ [SkinVolt] ALERT: Steam returned HTML (likely a Rate Limit or Cloudflare block)");
    }

    // 3. Use the Phase 1 Validation Layer [cite: 13, 101]
    crate::steam::validate::validate_steam_response(text.clone())?;

    let json: Value = serde_json::from_str(&text).map_err(|_| "Invalid JSON format from Steam")?;

    if json["success"] != true {
        println!("❌ [SkinVolt] Steam Success: false");
        return Err("Steam reported success: false. Item might not exist.".into());
    }

    // 4. Extract and clean the price string [cite: 78]
    let price_str = json["lowest_price"]
        .as_str()
        .ok_or("No 'lowest_price' found for this item")?
        .replace('$', "")
        .replace(',', "");

    let price = price_str.parse::<f64>().map_err(|_| "Failed to parse price as number")?;
    let timestamp = chrono::Utc::now().timestamp();

    println!("✅ [SkinVolt] Success: {} parsed as {}", market_hash_name, price);

    Ok((price, timestamp))
}
