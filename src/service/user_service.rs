use crate::domain::{DomainEvent, EventObserver};
use crate::repository::user_repo;
use rusqlite::{params, Connection};

pub fn create(
    conn: &Connection,
    uid: &str,
    name: &str,
    now: &str,
    observer: &dyn EventObserver,
) -> Result<(), String> {
    user_repo::insert(conn, uid, name, now)?;
    observer.on_event(&DomainEvent::UserCreated {
        uid: uid.to_string(),
    })?;
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    user_repo::list(conn)
}

pub fn show(conn: &Connection, uid: &str) -> Result<Option<String>, String> {
    user_repo::get_name(conn, uid)
}

pub fn update(conn: &Connection, uid: &str, name: &str, now: &str) -> Result<usize, String> {
    user_repo::update_name(conn, uid, name, now)
}

pub fn merge(conn: &mut Connection, from_uid: &str, to_uid: &str, now: &str) -> Result<(), String> {
    if !user_repo::exists(conn, from_uid)? {
        return Err(format!("user not found: {from_uid}"));
    }
    if !user_repo::exists(conn, to_uid)? {
        return Err(format!("user not found: {to_uid}"));
    }

    let tx = conn.transaction().map_err(|e| format!("failed to start tx: {e}"))?;

    tx.execute(
        "UPDATE user_identities SET uid = ?1, updated_at = ?2 WHERE uid = ?3",
        params![to_uid, now, from_uid],
    )
    .map_err(|e| format!("failed to migrate identities: {e}"))?;

    tx.execute(
        "DELETE FROM scope_members
         WHERE uid = ?1 AND EXISTS (
           SELECT 1 FROM scope_members t WHERE t.scope_id = scope_members.scope_id AND t.uid = ?2
         )",
        params![from_uid, to_uid],
    )
    .map_err(|e| format!("failed to dedupe scope members: {e}"))?;

    tx.execute(
        "UPDATE scope_members SET uid = ?1 WHERE uid = ?2",
        params![to_uid, from_uid],
    )
    .map_err(|e| format!("failed to migrate scope members: {e}"))?;

    tx.execute(
        "DELETE FROM events
         WHERE uid = ?1
           AND idempotency_key IS NOT NULL
           AND EXISTS (
             SELECT 1 FROM events t
             WHERE t.uid = ?2
               AND t.scope_id = events.scope_id
               AND t.idempotency_key = events.idempotency_key
           )",
        params![from_uid, to_uid],
    )
    .map_err(|e| format!("failed to dedupe events: {e}"))?;

    tx.execute(
        "UPDATE events SET uid = ?1 WHERE uid = ?2",
        params![to_uid, from_uid],
    )
    .map_err(|e| format!("failed to migrate events: {e}"))?;

    tx.execute(
        "INSERT INTO state (scope_id, uid, state_key, value_json, updated_at)
         SELECT scope_id, ?1, state_key, value_json, updated_at FROM state WHERE uid = ?2
         ON CONFLICT(scope_id, uid, state_key) DO UPDATE SET
           value_json = CASE
             WHEN excluded.updated_at >= state.updated_at THEN excluded.value_json
             ELSE state.value_json
           END,
           updated_at = CASE
             WHEN excluded.updated_at >= state.updated_at THEN excluded.updated_at
             ELSE state.updated_at
           END",
        params![to_uid, from_uid],
    )
    .map_err(|e| format!("failed to migrate state: {e}"))?;

    tx.execute("DELETE FROM state WHERE uid = ?1", params![from_uid])
        .map_err(|e| format!("failed to cleanup state: {e}"))?;

    tx.execute(
        "INSERT INTO metrics (scope_id, uid, metric_key, metric_value, metric_json, updated_at)
         SELECT scope_id, ?1, metric_key, metric_value, metric_json, updated_at FROM metrics WHERE uid = ?2
         ON CONFLICT(scope_id, uid, metric_key) DO UPDATE SET
           metric_value = CASE
             WHEN excluded.updated_at >= metrics.updated_at THEN excluded.metric_value
             ELSE metrics.metric_value
           END,
           metric_json = CASE
             WHEN excluded.updated_at >= metrics.updated_at THEN excluded.metric_json
             ELSE metrics.metric_json
           END,
           updated_at = CASE
             WHEN excluded.updated_at >= metrics.updated_at THEN excluded.updated_at
             ELSE metrics.updated_at
           END",
        params![to_uid, from_uid],
    )
    .map_err(|e| format!("failed to migrate metrics: {e}"))?;

    tx.execute("DELETE FROM metrics WHERE uid = ?1", params![from_uid])
        .map_err(|e| format!("failed to cleanup metrics: {e}"))?;

    tx.execute(
        "INSERT INTO topk (scope_id, uid, topic, rank, item_key, weight, updated_at)
         SELECT scope_id, ?1, topic, rank, item_key, weight, updated_at FROM topk WHERE uid = ?2
         ON CONFLICT(scope_id, uid, topic, rank) DO UPDATE SET
           item_key = CASE
             WHEN excluded.updated_at >= topk.updated_at THEN excluded.item_key
             ELSE topk.item_key
           END,
           weight = CASE
             WHEN excluded.updated_at >= topk.updated_at THEN excluded.weight
             ELSE topk.weight
           END,
           updated_at = CASE
             WHEN excluded.updated_at >= topk.updated_at THEN excluded.updated_at
             ELSE topk.updated_at
           END",
        params![to_uid, from_uid],
    )
    .map_err(|e| format!("failed to migrate topk: {e}"))?;

    tx.execute("DELETE FROM topk WHERE uid = ?1", params![from_uid])
        .map_err(|e| format!("failed to cleanup topk: {e}"))?;

    tx.execute(
        "UPDATE users SET status = 'merged', updated_at = ?1 WHERE uid = ?2",
        params![now, from_uid],
    )
    .map_err(|e| format!("failed to mark source user as merged: {e}"))?;

    tx.commit().map_err(|e| format!("failed to commit merge: {e}"))?;
    Ok(())
}
