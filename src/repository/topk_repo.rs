use rusqlite::{params, Connection, Transaction};

pub struct TopkRow<'a> {
    pub scope_id: &'a str,
    pub uid: &'a str,
    pub topic: &'a str,
    pub rank: i64,
    pub item_key: &'a str,
    pub weight: f64,
    pub now: &'a str,
}

pub fn clear(tx: &Transaction<'_>, scope_id: &str, uid: &str, topic: &str) -> Result<(), String> {
    tx.execute(
        "DELETE FROM topk WHERE scope_id = ?1 AND uid = ?2 AND topic = ?3",
        params![scope_id, uid, topic],
    )
    .map_err(|e| format!("failed to clear topk: {e}"))?;
    Ok(())
}

pub fn insert(tx: &Transaction<'_>, row: TopkRow<'_>) -> Result<(), String> {
    tx.execute(
        "INSERT INTO topk (scope_id, uid, topic, rank, item_key, weight, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            row.scope_id,
            row.uid,
            row.topic,
            row.rank,
            row.item_key,
            row.weight,
            row.now
        ],
    )
    .map_err(|e| format!("failed to insert topk: {e}"))?;
    Ok(())
}

pub fn query(
    conn: &Connection,
    scope_id: &str,
    uid: &str,
    topic: &str,
    limit: usize,
) -> Result<Vec<(i64, String, f64)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT rank, item_key, weight FROM topk
             WHERE scope_id = ?1 AND uid = ?2 AND topic = ?3
             ORDER BY rank ASC
             LIMIT ?4",
        )
        .map_err(|e| format!("failed to prepare topk query: {e}"))?;

    let rows = stmt
        .query_map(params![scope_id, uid, topic, limit as i64], |row| {
            let rank: i64 = row.get(0)?;
            let item: String = row.get(1)?;
            let weight: f64 = row.get(2)?;
            Ok((rank, item, weight))
        })
        .map_err(|e| format!("failed topk query: {e}"))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("failed row read: {e}"))?);
    }
    Ok(out)
}
