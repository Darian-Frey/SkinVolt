// Placeholder until implemented
pub fn fetch_placeholder() {}
use reqwest::blocking::get;
use serde_json::Value;

pub fn fetch_price(market_hash_name: &str) -> Result<(f64, i64), String> {
    let url = format!(
        "https://steamcommunity.com/market/priceoverview/?currency=1&appid=730&market_hash_name={}",
        urlencoding::encode(market_hash_name)
    );

    let resp = get(&url)
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())?;

    let json: Value = serde_json::from_str(&resp).map_err(|e| e.to_string())?;

    let price_str = json["lowest_price"]
        .as_str()
        .ok_or("Missing price")?
        .replace("$", "")
        .replace(",", "");

    let price = price_str.parse::<f64>().map_err(|e| e.to_string())?;
    let timestamp = chrono::Utc::now().timestamp();

    Ok((price, timestamp))
}
