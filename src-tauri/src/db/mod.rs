use rusqlite::{Connection, Result, params};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use serde::Serialize;

/// Returns the path to SkinVolt's database file.
pub fn db_path() -> PathBuf {
    let proj = ProjectDirs::from("com", "SkinVolt", "SkinVolt")
        .expect("Failed to get project directory");

    let data_dir = proj.data_dir();
    fs::create_dir_all(data_dir).expect("Failed to create data directory");

    data_dir.join("cache.db")
}

/// Opens the database connection.
pub fn get_db() -> Result<Connection> {
    let path = db_path();
    Connection::open(path)
}

/// Runs schema.sql to initialize all tables.
pub fn init_db() -> Result<()> {
    let path = db_path();
    eprintln!("[db] database path: {}", path.display());
    let conn = Connection::open(&path)?;
    let schema = include_str!("../schema.sql");
    conn.execute_batch(schema)?;
    eprintln!("[db] schema applied successfully");
    Ok(())
}

// ────────────────────────────────────────────────────────────────
//   INVENTORY & PRICE LOGIC
// ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct InventoryItemFull {
    pub market_hash_name: String,
    pub quantity: u32,
    pub price: Option<f64>,
    pub last_updated: Option<i64>,
}

/// Adds a history entry for Phase 3 analytics 
pub fn add_price_history(name: &str, price: f64, timestamp: i64) -> Result<(), String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO price_history (market_hash_name, price, timestamp) VALUES (?1, ?2, ?3)",
        params![name, price, timestamp],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Retrieves historical price points for an item, latest first
pub fn get_price_history(name: &str, limit: u32) -> Result<Vec<f64>, String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT price FROM price_history WHERE market_hash_name = ?1 ORDER BY timestamp DESC LIMIT ?2")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![name, limit], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut prices = Vec::new();
    for p in rows {
        if let Ok(val) = p { prices.push(val); }
    }
    Ok(prices)
}

pub fn get_inventory_full() -> Result<Vec<InventoryItemFull>, String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT i.market_hash_name, i.quantity, p.price, p.timestamp
             FROM inventory i
             LEFT JOIN price_cache p ON i.market_hash_name = p.market_hash_name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(InventoryItemFull {
                market_hash_name: row.get(0)?,
                quantity: row.get(1)?,
                price: row.get(2)?,
                last_updated: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for item in rows {
        items.push(item.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

pub fn add_inventory_item(name: String, quantity: u32) -> Result<(), String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO inventory (market_hash_name, quantity) VALUES (?1, ?2)",
        (name, quantity),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_inventory_items_internal() -> Result<Vec<String>, String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT market_hash_name FROM inventory")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for name in rows {
        if let Ok(n) = name { items.push(n); }
    }
    Ok(items)
}

pub fn get_last_fetch_time(name: &str) -> Result<i64, String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT timestamp FROM price_cache WHERE market_hash_name = ?1")
        .map_err(|e| e.to_string())?;
    
    let time: i64 = stmt.query_row([name], |row| row.get(0)).unwrap_or(0);
    Ok(time)
}

pub fn get_inventory() -> Result<Vec<InventoryItem>, String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT market_hash_name, quantity FROM inventory")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(InventoryItem {
                market_hash_name: row.get(0)?,
                quantity: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for item in rows {
        items.push(item.map_err(|e| e.to_string())?);
    }
    Ok(items)
}

#[derive(Serialize)]
pub struct InventoryItem {
    pub market_hash_name: String,
    pub quantity: u32,
}

// ────────────────────────────────────────────────────────────────
//   MARKET DISCOVERY & METADATA
// ────────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct ItemMetadata {
    pub market_hash_name: String,
    pub rarity: Option<String>,
    pub item_type: Option<String>,
    pub collection: Option<String>,
    pub icon_url: Option<String>,
}

pub fn upsert_item_metadata(m: ItemMetadata) -> Result<(), String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO item_metadata (market_hash_name, rarity, item_type, collection, icon_url)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(market_hash_name) DO UPDATE SET
            rarity = excluded.rarity,
            item_type = excluded.item_type,
            collection = excluded.collection,
            icon_url = excluded.icon_url",
        params![m.market_hash_name, m.rarity, m.item_type, m.collection, m.icon_url],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
pub struct TopMover {
    pub market_hash_name: String,
    pub current_price: f64,
    pub old_price: f64,
    pub change_pct: f64,
}

pub fn get_top_movers_db(limit: u32) -> Result<Vec<TopMover>, String> {
    let conn = get_db().map_err(|e| e.to_string())?;
    let day_ago = chrono::Utc::now().timestamp() - 86400;

    let mut stmt = conn.prepare(
        "SELECT 
            curr.market_hash_name, 
            curr.price as current_price,
            (SELECT price FROM price_history 
             WHERE market_hash_name = curr.market_hash_name 
             AND timestamp <= ?1 
             ORDER BY timestamp DESC LIMIT 1) as old_price
         FROM price_cache curr
         WHERE old_price IS NOT NULL
         ORDER BY ABS((curr.price - old_price) / old_price) DESC
         LIMIT ?2"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![day_ago, limit], |row| {
        let name: String = row.get(0)?;
        let curr: f64 = row.get(1)?;
        let old: f64 = row.get(2)?;
        let change = if old != 0.0 { (curr - old) / old * 100.0 } else { 0.0 };
        Ok(TopMover {
            market_hash_name: name,
            current_price: curr,
            old_price: old,
            change_pct: change,
        })
    }).map_err(|e| e.to_string())?;

    let mut movers = Vec::new();
    for m in rows {
        if let Ok(val) = m { movers.push(val); }
    }
    Ok(movers)
}