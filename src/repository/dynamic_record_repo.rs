use rusqlite::{params, Connection};

#[allow(dead_code)]
pub struct DynamicRecordUpsert<'a> {
    pub record_id: &'a str,
    pub schema_id: &'a str,
    pub entity_key: &'a str,
    pub uid: Option<&'a str>,
    pub scope_id: Option<&'a str>,
    pub payload_json: &'a str,
    pub now: &'a str,
}

#[allow(dead_code)]
pub fn upsert(conn: &Connection, input: DynamicRecordUpsert<'_>) -> Result<(), String> {
    conn.execute(
        "INSERT INTO dynamic_records (record_id, schema_id, entity_key, uid, scope_id, payload_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT(record_id) DO UPDATE SET
           schema_id=excluded.schema_id,
           entity_key=excluded.entity_key,
           uid=excluded.uid,
           scope_id=excluded.scope_id,
           payload_json=excluded.payload_json,
           updated_at=excluded.updated_at",
        params![
            input.record_id,
            input.schema_id,
            input.entity_key,
            input.uid,
            input.scope_id,
            input.payload_json,
            input.now
        ],
    )
    .map_err(|e| format!("failed to upsert dynamic record: {e}"))?;
    Ok(())
}
