use std::fs;
use std::path::Path;

pub fn doctor() {
    println!("agent-memory-cli is ready");
}

pub fn todo(group: &str, command: &str) {
    println!("TODO: implement {group}::{command}");
}

pub fn admin_migrate(db_path: &str) -> Result<(), String> {
    let db = Path::new(db_path);
    if let Some(parent) = db.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create db directory {}: {e}", parent.display()))?;
        }
    }

    let conn = rusqlite::Connection::open(db)
        .map_err(|e| format!("failed to open db {}: {e}", db.display()))?;

    let schema_sql = include_str!("../../specs/SCHEMA_SQLITE_V01.sql");
    conn.execute_batch(schema_sql)
        .map_err(|e| format!("migration failed: {e}"))?;

    println!("migrated schema to {}", db.display());
    Ok(())
}
