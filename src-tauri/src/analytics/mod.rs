pub mod indicators;
use serde::Serialize;

#[derive(Serialize)]
pub struct ItemAnalytics {
    pub volatility: f64,
    pub sma_7: f64,
    pub sma_30: f64,
    pub trend: String,
}

#[tauri::command]
pub fn get_item_analytics(market_hash_name: String) -> Result<ItemAnalytics, String> {
    // Retrieve historical data for calculations
    let history_30 = crate::db::get_price_history(&market_hash_name, 30)?;
    let current_price = history_30.first().cloned().unwrap_or(0.0);
    
    let volatility = indicators::calculate_volatility(&history_30);
    let sma_30 = indicators::calculate_moving_average(&history_30);
    
    let history_7 = if history_30.len() >= 7 { &history_30[0..7] } else { &history_30 };
    let sma_7 = indicators::calculate_moving_average(history_7);
    
    let trend = indicators::generate_trend_signal(current_price, sma_7);
    
    Ok(ItemAnalytics {
        volatility,
        sma_7,
        sma_30,
        trend,
    })
}

#[derive(Serialize)]
pub struct PricePoint {
    pub timestamp: i64,
    pub price: f64,
    pub sma: Option<f64>,
    pub upper_band: Option<f64>,
    pub lower_band: Option<f64>,
}

#[tauri::command]
pub fn get_item_history_full(market_hash_name: String) -> Result<Vec<PricePoint>, String> {
    let conn = crate::db::get_db().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT timestamp, price FROM price_history WHERE market_hash_name = ?1 ORDER BY timestamp ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([market_hash_name], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
    })
    .map_err(|e| e.to_string())?;

    let mut raw_data = Vec::new();
    for r in rows {
        if let Ok(p) = r { raw_data.push(p); }
    }

    // --- CALCULATE OVERLAYS (Moving Average & Volatility Bands) ---
    let mut history = Vec::with_capacity(raw_data.is_empty().then(|| 0).unwrap_or(raw_data.len()));
    let window_size = 20; // Standard Bollinger window

    for i in 0..raw_data.len() {
        let (ts, price) = raw_data[i];
        
        // Window calculations
        let start = if i >= window_size { i + 1 - window_size } else { 0 };
        let slice = &raw_data[start..=i];
        let prices: Vec<f64> = slice.iter().map(|&(_, p)| p).collect();
        
        let sma = indicators::calculate_moving_average(&prices);
        let vol = indicators::calculate_volatility(&prices);
        
        history.push(PricePoint {
            timestamp: ts,
            price,
            sma: Some(sma),
            upper_band: Some(sma + (vol * 2.0)), // 2 Std Dev
            lower_band: Some(sma - (vol * 2.0)),
        });
    }

    Ok(history)
}
