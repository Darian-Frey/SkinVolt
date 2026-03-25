use serde_json::Value;
use tokio::time::{sleep, Duration};
use crate::utils::backoff::retry_with_backoff;
use once_cell::sync::Lazy;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent("SkinVolt/1.0")
        .build()
        .unwrap()
});

#[tauri::command]
pub async fn fetch_inventory_prices(items: Vec<String>) -> Result<(), String> {
    for item in items {
        match fetch_single_price_async(&item).await {
            Ok(_) => {
                sleep(Duration::from_secs(4)).await; 
            }
            Err(e) => println!("Error fetching {}: {}", item, e),
        }
    }
    Ok(())
}

pub async fn fetch_price(market_hash_name: &str) -> Result<(f64, i64), String> {
    fetch_single_price_async(market_hash_name).await
}

pub async fn fetch_single_price_async(market_hash_name: &str) -> Result<(f64, i64), String> {
    let url = format!(
        "https://steamcommunity.com/market/priceoverview/?currency=1&appid=730&market_hash_name={}",
        urlencoding::encode(market_hash_name)
    );

    retry_with_backoff(|| async {
        let resp = CLIENT.get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
        process_price_json(&text, market_hash_name)
    }, 3).await
}

fn process_price_json(text: &str, name: &str) -> Result<(f64, i64), String> {
    crate::steam::validate::validate_steam_response(text.to_string())?;

    let json: Value = serde_json::from_str(text).map_err(|_| "Invalid JSON format")?;

    if json["success"] != true {
        return Err("Steam reported success: false".into());
    }

    let lowest = json["lowest_price"].as_str().unwrap_or("0").replace('$', "").replace(',', "");
    let median = json["median_price"].as_str().unwrap_or("0").replace('$', "").replace(',', "");
    let volume = json["volume"].as_str().unwrap_or("0").replace(',', "");

    let price = lowest.parse::<f64>().unwrap_or_else(|_| median.parse::<f64>().unwrap_or(0.0));
    let timestamp = chrono::Utc::now().timestamp();

    println!("✅ [SkinVolt] {} | Price: {} | Vol: {}", name, price, volume);

    Ok((price, timestamp))
}

pub async fn fetch_item_details(market_hash_name: &str) -> Result<crate::db::ItemMetadata, String> {
    let url = format!(
        "https://steamcommunity.com/market/search/render/?query={}&search_descriptions=0&count=1&norender=1",
        urlencoding::encode(market_hash_name)
    );

    retry_with_backoff(|| async {
        let resp = CLIENT.get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let json: Value = resp.json().await.map_err(|e| format!("Invalid JSON: {}", e))?;
        
        if json["success"] != true || json["results"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
            return Err("Steam search failed or item not found".into());
        }

        let result = &json["results"][0];
        let asset = &result["asset_description"];
        
        let appid = asset["appid"].as_u64().or_else(|| asset["appid"].as_str().and_then(|s| s.parse().ok())).unwrap_or(730);

        let icon_hash = asset["icon_url"].as_str().unwrap_or("");
        let icon_url = if !icon_hash.is_empty() {
            format!("https://community.cloudflare.steamstatic.com/economy/image/{}/330x192", icon_hash)
        } else {
            "".into()
        };

        let rarity = asset["type"].as_str().unwrap_or("Unknown").to_string();
        
        // Find collection in descriptions
        let mut collection = None;
        if let Some(desc_list) = asset["descriptions"].as_array() {
            for d in desc_list {
                if let Some(val) = d["value"].as_str() {
                    if val.contains("Collection") || val.contains("Case") {
                        collection = Some(val.to_string());
                        break;
                    }
                }
            }
        }

        Ok(crate::db::ItemMetadata {
            market_hash_name: market_hash_name.to_string(),
            appid,
            rarity: Some(rarity),
            item_type: Some(asset["type"].as_str().unwrap_or("Item").to_string()),
            collection,
            icon_url: Some(icon_url),
        })
    }, 2).await
}

#[tauri::command]
pub async fn fetch_price_history(market_hash_name: String, appid: Option<u64>) -> Result<Vec<(i64, f64)>, String> {
    let appid = appid.unwrap_or(730);
    let url = format!(
        "https://steamcommunity.com/market/pricehistory/?appid={}&market_hash_name={}",
        appid,
        urlencoding::encode(&market_hash_name)
    );

    retry_with_backoff(|| async {
        let resp = CLIENT.get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
        let json: Value = serde_json::from_str(&text).map_err(|_| "Invalid JSON format")?;
        
        if json["success"] != true {
            return Err("Steam history fetch failed: success=false".into());
        }

        let mut history = Vec::new();
        if let Some(prices) = json["prices"].as_array() {
            for p in prices {
                if let Some(point) = p.as_array() {
                    if point.len() >= 2 {
                        let date_str = point[0].as_str().unwrap_or("");
                        let price = point[1].as_f64().unwrap_or(0.0);
                        
                        // Parse date: "Mar 25 2026 01:00:00 +00"
                        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%b %d %Y %H:%M:%S +00") {
                            history.push((dt.and_utc().timestamp(), price));
                        }
                    }
                }
            }
        }

        println!("✅ [SkinVolt] Fetched {} history points for {}", history.len(), market_hash_name);
        Ok(history)
    }, 2).await
}
