use crate::db;
use crate::domain::NoopObserver;
use crate::service::{identity_service, scope_service, user_service};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn doctor() {
    println!("agent-memory-cli is ready");
}

pub fn todo(group: &str, command: &str) {
    println!("TODO: implement {group}::{command}");
}

pub fn admin_migrate(db_path: &str) -> Result<(), String> {
    db::ensure_parent_dir(db_path)?;
    let conn = db::connect(db_path)?;
    let schema_sql = include_str!("../../specs/SCHEMA_SQLITE_V01.sql");
    conn.execute_batch(schema_sql)
        .map_err(|e| format!("migration failed: {e}"))?;
    println!("migrated schema to {db_path}");
    Ok(())
}

fn now_ts() -> String {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    n.to_string()
}

fn new_id(prefix: &str) -> String {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}_{n}")
}

fn open_and_migrate(db_path: &str) -> Result<Connection, String> {
    admin_migrate(db_path)?;
    db::connect(db_path)
}

pub fn user_create(db_path: &str, name: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let uid = new_id("u");
    let now = now_ts();
    let observer = NoopObserver;

    user_service::create(&conn, &uid, name, &now, &observer)?;
    println!("created user uid={uid} name={name}");
    Ok(())
}

pub fn user_list(db_path: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    for (uid, name) in user_service::list(&conn)? {
        println!("uid={uid} name={name}");
    }
    Ok(())
}

pub fn user_show(db_path: &str, uid: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    match user_service::show(&conn, uid)? {
        Some(name) => {
            println!("uid={uid} name={name}");
            Ok(())
        }
        None => Err(format!("user not found: {uid}")),
    }
}

pub fn user_update(db_path: &str, uid: &str, name: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let now = now_ts();
    let n = user_service::update(&conn, uid, name, &now)?;
    if n == 0 {
        return Err(format!("user not found: {uid}"));
    }
    println!("updated user uid={uid} name={name}");
    Ok(())
}

pub fn identity_link(
    db_path: &str,
    uid: &str,
    channel: &str,
    channel_user_id: &str,
) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let now = now_ts();
    let identity_id = new_id("ident");
    let observer = NoopObserver;

    identity_service::link(
        &conn,
        &identity_id,
        uid,
        channel,
        channel_user_id,
        &now,
        &observer,
    )?;

    println!("linked identity uid={uid} channel={channel} channel_user_id={channel_user_id}");
    Ok(())
}

pub fn identity_resolve(db_path: &str, channel: &str, channel_user_id: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    match identity_service::resolve(&conn, channel, channel_user_id)? {
        Some(uid) => {
            println!("resolved uid={uid} channel={channel} channel_user_id={channel_user_id}");
            Ok(())
        }
        None => Err(format!("identity not found: {channel}:{channel_user_id}")),
    }
}

pub fn identity_unlink(db_path: &str, channel: &str, channel_user_id: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let n = identity_service::unlink(&conn, channel, channel_user_id)?;
    if n == 0 {
        return Err(format!("identity not found: {channel}:{channel_user_id}"));
    }
    println!("unlinked identity channel={channel} channel_user_id={channel_user_id}");
    Ok(())
}

pub fn scope_create(db_path: &str, scope_id: &str, scope_type: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let now = now_ts();
    scope_service::create(&conn, scope_id, scope_type, &now)?;
    println!("created scope id={scope_id} type={scope_type}");
    Ok(())
}

pub fn scope_add_member(
    db_path: &str,
    scope_id: &str,
    uid: &str,
    role: &str,
) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let now = now_ts();
    let observer = NoopObserver;
    scope_service::add_member(&conn, scope_id, uid, role, &now, &observer)?;
    println!("added scope member scope_id={scope_id} uid={uid} role={role}");
    Ok(())
}

pub fn scope_list(db_path: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    for (id, kind) in scope_service::list(&conn)? {
        println!("id={id} type={kind}");
    }
    Ok(())
}

pub fn scope_members(db_path: &str, scope_id: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    for (uid, role) in scope_service::members(&conn, scope_id)? {
        println!("scope_id={scope_id} uid={uid} role={role}");
    }
    Ok(())
}

pub fn ingest_event(
    db_path: &str,
    uid: &str,
    scope_id: &str,
    event_type: &str,
    file: &str,
) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let raw = fs::read_to_string(file).map_err(|e| format!("failed to read event file: {e}"))?;
    let payload: Value =
        serde_json::from_str(&raw).map_err(|e| format!("invalid json payload: {e}"))?;

    let event_id = new_id("evt");
    let now = now_ts();
    conn.execute(
        "INSERT INTO events (event_id, uid, scope_id, event_type, event_ts, payload_json, schema_version, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, '1', ?5)",
        params![event_id, uid, scope_id, event_type, now, payload.to_string()],
    )
    .map_err(|e| format!("failed to insert event: {e}"))?;

    if event_type == "meal.rated" {
        let cuisine = payload
            .get("cuisine")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "meal.rated requires string field: cuisine".to_string())?;
        upsert_topk_counter(&conn, scope_id, uid, "food_pref", cuisine, 1.0)?;
    } else if event_type == "expense.logged" {
        let category = payload
            .get("category")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "expense.logged requires string field: category".to_string())?;
        upsert_topk_counter(&conn, scope_id, uid, "spend_category", category, 1.0)?;
    }

    rebuild_topk(&conn, scope_id, uid, "food_pref")?;
    rebuild_topk(&conn, scope_id, uid, "spend_category")?;

    println!("ingested event id={event_id} type={event_type}");
    Ok(())
}

fn upsert_topk_counter(
    conn: &Connection,
    scope_id: &str,
    uid: &str,
    topic: &str,
    item: &str,
    delta: f64,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO metrics (scope_id, uid, metric_key, metric_value, metric_json, updated_at)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5)
         ON CONFLICT(scope_id, uid, metric_key)
         DO UPDATE SET metric_value = COALESCE(metrics.metric_value, 0) + excluded.metric_value, updated_at = excluded.updated_at",
        params![scope_id, uid, format!("counter:{topic}:{item}"), delta, now_ts()],
    )
    .map_err(|e| format!("failed to update counter: {e}"))?;
    Ok(())
}

fn rebuild_topk(conn: &Connection, scope_id: &str, uid: &str, topic: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM topk WHERE scope_id = ?1 AND uid = ?2 AND topic = ?3",
        params![scope_id, uid, topic],
    )
    .map_err(|e| format!("failed to clear topk: {e}"))?;

    let mut stmt = conn
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

    for (idx, row) in rows.enumerate() {
        let (key, score) = row.map_err(|e| format!("failed row: {e}"))?;
        let item_key = key.splitn(3, ':').nth(2).unwrap_or_default().to_string();
        conn.execute(
            "INSERT INTO topk (scope_id, uid, topic, rank, item_key, weight, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                scope_id,
                uid,
                topic,
                (idx + 1) as i64,
                item_key,
                score,
                now_ts()
            ],
        )
        .map_err(|e| format!("failed to insert topk: {e}"))?;
    }

    Ok(())
}

pub fn query_latest(db_path: &str, uid: &str, scope_id: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT event_id, event_type, event_ts FROM events
             WHERE uid = ?1 AND scope_id = ?2
             ORDER BY event_ts DESC, created_at DESC
             LIMIT 1",
        )
        .map_err(|e| format!("failed to prepare latest query: {e}"))?;

    let mut rows = stmt
        .query(params![uid, scope_id])
        .map_err(|e| format!("failed latest query: {e}"))?;

    if let Some(row) = rows.next().map_err(|e| format!("failed row read: {e}"))? {
        let event_id: String = row.get(0).map_err(|e| format!("failed col read: {e}"))?;
        let event_type: String = row.get(1).map_err(|e| format!("failed col read: {e}"))?;
        let event_ts: String = row.get(2).map_err(|e| format!("failed col read: {e}"))?;
        println!("latest event_id={event_id} type={event_type} ts={event_ts}");
    }

    Ok(())
}

pub fn query_topk(
    db_path: &str,
    uid: &str,
    scope_id: &str,
    topic: &str,
    limit: usize,
) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
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

    for row in rows {
        let (rank, item, weight) = row.map_err(|e| format!("failed row read: {e}"))?;
        println!("rank={rank} item={item} weight={weight}");
    }

    Ok(())
}
