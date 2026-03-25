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

/// Calculates Relative Strength Index (RSI) for a given period (standard 14)
pub fn calculate_rsi(prices: &[f64], period: usize) -> f64 {
    if prices.len() <= period { return 50.0; } // Neutral if insufficient data

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..=period {
        let diff = prices[prices.len() - i] - prices[prices.len() - i - 1];
        if diff >= 0.0 { gains += diff; } else { losses -= diff; }
    }

    if losses == 0.0 { return 100.0; }
    
    let rs = (gains / period as f64) / (losses / period as f64);
    100.0 - (100.0 / (1.0 + rs))
}

/// Calculates price Momentum (current - n periods ago)
pub fn calculate_momentum(prices: &[f64], period: usize) -> f64 {
    if prices.len() <= period { return 0.0; }
    prices[prices.len() - 1] - prices[prices.len() - 1 - period]
}
