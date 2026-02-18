use rusqlite::{params, Connection, OptionalExtension};

pub fn insert(
    conn: &Connection,
    identity_id: &str,
    uid: &str,
    channel: &str,
    channel_user_id: &str,
    now: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO user_identities (identity_id, uid, channel, channel_user_id, is_verified, confidence, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, 1.0, ?5, ?5)",
        params![identity_id, uid, channel, channel_user_id, now],
    )
    .map_err(|e| format!("failed to link identity: {e}"))?;
    Ok(())
}

pub fn resolve_uid(
    conn: &Connection,
    channel: &str,
    channel_user_id: &str,
) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT uid FROM user_identities WHERE channel = ?1 AND channel_user_id = ?2",
        params![channel, channel_user_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| format!("failed to resolve identity: {e}"))
}

pub fn delete(conn: &Connection, channel: &str, channel_user_id: &str) -> Result<usize, String> {
    conn.execute(
        "DELETE FROM user_identities WHERE channel = ?1 AND channel_user_id = ?2",
        params![channel, channel_user_id],
    )
    .map_err(|e| format!("failed to unlink identity: {e}"))
}
