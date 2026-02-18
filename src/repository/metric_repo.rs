use rusqlite::{params, Connection, Transaction};

pub fn upsert_counter(
    tx: &Transaction<'_>,
    scope_id: &str,
    uid: &str,
    topic: &str,
    item: &str,
    delta: f64,
    now: &str,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO metrics (scope_id, uid, metric_key, metric_value, metric_json, updated_at)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5)
         ON CONFLICT(scope_id, uid, metric_key)
         DO UPDATE SET metric_value = COALESCE(metrics.metric_value, 0) + excluded.metric_value, updated_at = excluded.updated_at",
        params![scope_id, uid, format!("counter:{topic}:{item}"), delta, now],
    )
    .map_err(|e| format!("failed to update counter: {e}"))?;
    Ok(())
}

pub fn query_by_key(
    conn: &Connection,
    scope_id: &str,
    uid: &str,
    key: &str,
) -> Result<Option<(String, f64, String)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT metric_key, COALESCE(metric_value, 0), COALESCE(metric_json, '')
             FROM metrics
             WHERE scope_id = ?1 AND uid = ?2 AND metric_key = ?3",
        )
        .map_err(|e| format!("failed to prepare metric query: {e}"))?;
    let mut rows = stmt
        .query(params![scope_id, uid, key])
        .map_err(|e| format!("failed to run metric query: {e}"))?;
    if let Some(row) = rows.next().map_err(|e| format!("failed row read: {e}"))? {
        let k: String = row.get(0).map_err(|e| format!("failed col read: {e}"))?;
        let v: f64 = row.get(1).map_err(|e| format!("failed col read: {e}"))?;
        let j: String = row.get(2).map_err(|e| format!("failed col read: {e}"))?;
        Ok(Some((k, v, j)))
    } else {
        Ok(None)
    }
}

pub fn query_by_prefix(
    conn: &Connection,
    scope_id: &str,
    uid: &str,
    prefix: &str,
) -> Result<Vec<(String, f64, String)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT metric_key, COALESCE(metric_value, 0), COALESCE(metric_json, '')
             FROM metrics
             WHERE scope_id = ?1 AND uid = ?2 AND metric_key LIKE ?3
             ORDER BY metric_key ASC",
        )
        .map_err(|e| format!("failed to prepare metric prefix query: {e}"))?;
    let like = format!("{prefix}%");
    let rows = stmt
        .query_map(params![scope_id, uid, like], |row| {
            let k: String = row.get(0)?;
            let v: f64 = row.get(1)?;
            let j: String = row.get(2)?;
            Ok((k, v, j))
        })
        .map_err(|e| format!("failed to run metric prefix query: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("failed row read: {e}"))?);
    }
    Ok(out)
}

pub fn topk_source(
    tx: &Transaction<'_>,
    scope_id: &str,
    uid: &str,
    topic: &str,
) -> Result<Vec<(String, f64)>, String> {
    let mut stmt = tx
        .prepare(
            "SELECT metric_key, COALESCE(metric_value, 0) as score
             FROM metrics
             WHERE scope_id = ?1 AND uid = ?2 AND metric_key LIKE ?3
             ORDER BY score DESC, metric_key ASC
             LIMIT 10",
        )
        .map_err(|e| format!("failed to prepare topk query: {e}"))?;

    let like = format!("counter:{topic}:%");
    let rows = stmt
        .query_map(params![scope_id, uid, like], |row| {
            let key: String = row.get(0)?;
            let score: f64 = row.get(1)?;
            Ok((key, score))
        })
        .map_err(|e| format!("failed to load topk counters: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("failed row: {e}"))?);
    }
    Ok(out)
}
