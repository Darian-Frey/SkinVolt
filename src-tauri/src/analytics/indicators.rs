/// Calculates the volatility of a price set (Standard Deviation)
pub fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 { return 0.0; }
    
    let n = prices.len() as f64;
    let mean = prices.iter().sum::<f64>() / n;
    
    let variance = prices.iter()
        .map(|p| (p - mean).powi(2))
        .sum::<f64>() / n;
        
    variance.sqrt()
}

/// Calculates a Simple Moving Average (SMA)
pub fn calculate_moving_average(prices: &[f64]) -> f64 {
    if prices.is_empty() { return 0.0; }
    prices.iter().sum::<f64>() / (prices.len() as f64)
}

/// Generates a trend signal: "uptrend", "downtrend", or "stable"
pub fn generate_trend_signal(current_price: f64, sma: f64) -> String {
    if sma == 0.0 { return "stable".into(); }
    
    let diff_pct = (current_price - sma) / sma;
    
    if diff_pct > 0.02 {
        "uptrend".into()
    } else if diff_pct < -0.02 {
        "downtrend".into()
    } else {
        "stable".into()
    }
}
