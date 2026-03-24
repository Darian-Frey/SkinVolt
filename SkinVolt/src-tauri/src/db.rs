use rusqlite::{Connection, Result};
use directories::ProjectDirs;

pub fn get_db() -> Result<Connection> {
    let proj = ProjectDirs::from("com", "SkinVolt", "SkinVolt")
        .expect("Failed to get project directory");

    let db_path = proj.data_dir().join("cache.db");
    std::fs::create_dir_all(proj.data_dir()).ok();

    Connection::open(db_path)
}
