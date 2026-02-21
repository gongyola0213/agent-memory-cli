use crate::domain::schema::SchemaDef;
use rusqlite::{params, Connection};

pub fn upsert(
    conn: &Connection,
    schema: &SchemaDef,
    schema_json: &str,
    now: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO schema_registry (schema_id, version, schema_json, is_active, created_at)
         VALUES (?1, ?2, ?3, 1, ?4)
         ON CONFLICT(schema_id) DO UPDATE SET version=excluded.version, schema_json=excluded.schema_json, is_active=1",
        params![schema.schema_id, schema.version, schema_json, now],
    )
    .map_err(|e| format!("failed to register schema: {e}"))?;
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<(String, String, i64, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT schema_id, version, is_active, created_at FROM schema_registry ORDER BY created_at DESC")
        .map_err(|e| format!("failed to list schemas: {e}"))?;

    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("failed to read schemas: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}
