use rusqlite::{params, Connection};

pub fn insert_scope(
    conn: &Connection,
    scope_id: &str,
    scope_type: &str,
    now: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO scopes (scope_id, scope_type, created_at) VALUES (?1, ?2, ?3)",
        params![scope_id, scope_type, now],
    )
    .map_err(|e| format!("failed to create scope: {e}"))?;
    Ok(())
}

pub fn insert_member(
    conn: &Connection,
    scope_id: &str,
    uid: &str,
    role: &str,
    now: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO scope_members (scope_id, uid, role, added_at) VALUES (?1, ?2, ?3, ?4)",
        params![scope_id, uid, role, now],
    )
    .map_err(|e| format!("failed to add member: {e}"))?;
    Ok(())
}

pub fn list_scopes(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT scope_id, scope_type FROM scopes ORDER BY created_at DESC")
        .map_err(|e| format!("failed to prepare scope list: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let kind: String = row.get(1)?;
            Ok((id, kind))
        })
        .map_err(|e| format!("failed to list scopes: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("failed to read scope row: {e}"))?);
    }
    Ok(out)
}

pub fn list_members(conn: &Connection, scope_id: &str) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT uid, role FROM scope_members WHERE scope_id = ?1 ORDER BY added_at DESC")
        .map_err(|e| format!("failed to prepare scope members: {e}"))?;

    let rows = stmt
        .query_map(params![scope_id], |row| {
            let uid: String = row.get(0)?;
            let role: String = row.get(1)?;
            Ok((uid, role))
        })
        .map_err(|e| format!("failed to list scope members: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("failed to read scope member row: {e}"))?);
    }
    Ok(out)
}
