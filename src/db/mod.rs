use rusqlite::Connection;
use std::path::Path;

pub fn connect(db_path: &str) -> Result<Connection, String> {
    Connection::open(db_path).map_err(|e| format!("failed to open db {}: {e}", db_path))
}

pub fn ensure_parent_dir(db_path: &str) -> Result<(), String> {
    let db = Path::new(db_path);
    if let Some(parent) = db.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create db directory {}: {e}", parent.display()))?;
        }
    }
    Ok(())
}
