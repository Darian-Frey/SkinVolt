use rusqlite::{Connection, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Returns the path to SkinVolt's database file.
fn db_path() -> PathBuf {
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
    let conn = get_db()?;

    // Load schema.sql from the same directory as this file
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src-tauri")
        .join("src")
        .join("schema.sql");

    let schema = fs::read_to_string(&schema_path)
        .expect("Failed to read schema.sql");

    conn.execute_batch(&schema)?;

    Ok(())
}

