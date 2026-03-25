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

#[tauri::command]
pub async fn fetch_price_history(market_hash_name: String) -> Result<Vec<(i64, f64)>, String> {
    println!("📡 [SkinVolt] fetchPriceHistory called for: {}", market_hash_name);
    Ok(vec![])
}
