use rusqlite::{params, Connection, OptionalExtension};

pub fn insert(conn: &Connection, uid: &str, name: &str, now: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO users (uid, display_name, status, created_at, updated_at) VALUES (?1, ?2, 'active', ?3, ?3)",
        params![uid, name, now],
    )
    .map_err(|e| format!("failed to create user: {e}"))?;
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT uid, display_name FROM users ORDER BY created_at DESC")
        .map_err(|e| format!("failed to prepare user list: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let uid: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((uid, name))
        })
        .map_err(|e| format!("failed to list users: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("failed to read user row: {e}"))?);
    }
    Ok(out)
}

pub fn get_name(conn: &Connection, uid: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT display_name FROM users WHERE uid = ?1",
        params![uid],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| format!("failed to query user: {e}"))
}

pub fn update_name(conn: &Connection, uid: &str, name: &str, now: &str) -> Result<usize, String> {
    conn.execute(
        "UPDATE users SET display_name = ?1, updated_at = ?2 WHERE uid = ?3",
        params![name, now, uid],
    )
    .map_err(|e| format!("failed to update user: {e}"))
}
