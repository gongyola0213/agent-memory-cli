use crate::repository::{event_repo, metric_repo, topk_repo};
use rusqlite::Connection;

pub fn latest(
    conn: &Connection,
    uid: &str,
    scope_id: &str,
) -> Result<Option<(String, String, String)>, String> {
    event_repo::latest(conn, uid, scope_id)
}

pub fn metric(
    conn: &Connection,
    uid: &str,
    scope_id: &str,
    key: Option<&str>,
    prefix: Option<&str>,
) -> Result<Vec<(String, f64, String)>, String> {
    if key.is_none() && prefix.is_none() {
        return Err("query metric requires either --key or --prefix".to_string());
    }

    let mut out = Vec::new();
    if let Some(key) = key {
        if let Some(row) = metric_repo::query_by_key(conn, scope_id, uid, key)? {
            out.push(row);
        }
    }
    if let Some(prefix) = prefix {
        out.extend(metric_repo::query_by_prefix(conn, scope_id, uid, prefix)?);
    }
    Ok(out)
}

pub fn topk(
    conn: &Connection,
    uid: &str,
    scope_id: &str,
    topic: &str,
    limit: usize,
) -> Result<Vec<(i64, String, f64)>, String> {
    topk_repo::query(conn, scope_id, uid, topic, limit)
}
