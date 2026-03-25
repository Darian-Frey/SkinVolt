use rusqlite::{Connection, Result};
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
    let schema = include_str!("schema.sql");
    conn.execute_batch(schema)?;
    eprintln!("[db] schema applied successfully");
    Ok(())
}

//
// ────────────────────────────────────────────────────────────────
//   INVENTORY STRUCT + QUERY
// ────────────────────────────────────────────────────────────────
//

#[derive(Serialize)]
pub struct InventoryItem {
    pub market_hash_name: String,
    pub quantity: u32,
}

/// Loads the user's inventory from the database.
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

