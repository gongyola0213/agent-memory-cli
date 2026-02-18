use rusqlite::{params, OptionalExtension, Transaction};

pub struct NewEvent<'a> {
    pub event_id: &'a str,
    pub uid: &'a str,
    pub scope_id: &'a str,
    pub event_type: &'a str,
    pub event_ts: &'a str,
    pub payload_json: &'a str,
    pub idempotency_key: Option<&'a str>,
}

pub fn idempotency_exists(
    tx: &Transaction<'_>,
    scope_id: &str,
    uid: &str,
    key: &str,
) -> Result<bool, String> {
    let count: i64 = tx
        .query_row(
            "SELECT COUNT(1) FROM events WHERE scope_id = ?1 AND uid = ?2 AND idempotency_key = ?3",
            params![scope_id, uid, key],
            |row| row.get(0),
        )
        .map_err(|e| format!("failed idempotency check: {e}"))?;
    Ok(count > 0)
}

pub fn insert(tx: &Transaction<'_>, e: NewEvent<'_>) -> Result<(), String> {
    tx.execute(
        "INSERT INTO events (event_id, uid, scope_id, event_type, event_ts, payload_json, idempotency_key, schema_version, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '1', ?5)",
        params![
            e.event_id,
            e.uid,
            e.scope_id,
            e.event_type,
            e.event_ts,
            e.payload_json,
            e.idempotency_key
        ],
    )
    .map_err(|e| format!("failed to insert event: {e}"))?;
    Ok(())
}

pub fn latest(
    conn: &rusqlite::Connection,
    uid: &str,
    scope_id: &str,
) -> Result<Option<(String, String, String)>, String> {
    conn.query_row(
        "SELECT event_id, event_type, event_ts FROM events
         WHERE uid = ?1 AND scope_id = ?2
         ORDER BY rowid DESC
         LIMIT 1",
        params![uid, scope_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .optional()
    .map_err(|e| format!("failed latest query: {e}"))
}
