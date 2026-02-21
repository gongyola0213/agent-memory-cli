use rusqlite::{params, Connection};

pub fn enqueue(
    conn: &Connection,
    outbox_id: &str,
    stream: &str,
    key: &str,
    payload_json: &str,
    now: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO projection_outbox (outbox_id, stream, item_key, payload_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![outbox_id, stream, key, payload_json, now],
    )
    .map_err(|e| format!("failed to enqueue projection outbox: {e}"))?;
    Ok(())
}
