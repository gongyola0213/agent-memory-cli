use crate::db;
use crate::domain::NoopObserver;
use crate::service::{
    identity_service, ingest_service, query_service, scope_service, user_service,
};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn doctor(db_path: &str, as_json: bool) -> Result<(), String> {
    let db_exists = std::path::Path::new(db_path).exists();
    let schema_initialized = if db_exists {
        match db::connect(db_path) {
            Ok(conn) => {
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(1) FROM sqlite_master WHERE type='table' AND name='users'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                count > 0
            }
            Err(_) => false,
        }
    } else {
        false
    };

    if as_json {
        println!(
            "{}",
            json!({
                "ok": true,
                "db_path": db_path,
                "db_exists": db_exists,
                "schema_initialized": schema_initialized
            })
        );
    } else {
        println!("agent-memory-cli is ready");
        println!("db_path={db_path}");
        println!("db_exists={db_exists}");
        println!("schema_initialized={schema_initialized}");
    }
    Ok(())
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
    let _ = conn.execute("ALTER TABLE events ADD COLUMN idempotency_key TEXT", []);
    let _ = conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_events_idempotency ON events(scope_id, uid, idempotency_key) WHERE idempotency_key IS NOT NULL",
        [],
    );
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

fn open_db_checked(db_path: &str) -> Result<Connection, String> {
    let conn = db::connect(db_path)?;
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM sqlite_master WHERE type='table' AND name='users'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("failed schema check: {e}"))?;
    if exists == 0 {
        return Err(
            "schema not initialized. run: agent-memory-cli admin migrate --db <path>".to_string(),
        );
    }
    Ok(conn)
}

pub fn user_create(db_path: &str, name: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    let uid = new_id("u");
    let now = now_ts();
    let observer = NoopObserver;

    user_service::create(&conn, &uid, name, &now, &observer)?;
    println!("created user uid={uid} name={name}");
    Ok(())
}

pub fn user_list(db_path: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    for (uid, name) in user_service::list(&conn)? {
        println!("uid={uid} name={name}");
    }
    Ok(())
}

pub fn user_show(db_path: &str, uid: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    match user_service::show(&conn, uid)? {
        Some(name) => {
            println!("uid={uid} name={name}");
            Ok(())
        }
        None => Err(format!("user not found: {uid}")),
    }
}

pub fn user_update(db_path: &str, uid: &str, name: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
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
    let conn = open_db_checked(db_path)?;
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
    let conn = open_db_checked(db_path)?;
    match identity_service::resolve(&conn, channel, channel_user_id)? {
        Some(uid) => {
            println!("resolved uid={uid} channel={channel} channel_user_id={channel_user_id}");
            Ok(())
        }
        None => Err(format!("identity not found: {channel}:{channel_user_id}")),
    }
}

pub fn identity_unlink(db_path: &str, channel: &str, channel_user_id: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    let n = identity_service::unlink(&conn, channel, channel_user_id)?;
    if n == 0 {
        return Err(format!("identity not found: {channel}:{channel_user_id}"));
    }
    println!("unlinked identity channel={channel} channel_user_id={channel_user_id}");
    Ok(())
}

pub fn scope_create(db_path: &str, scope_id: &str, scope_type: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
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
    let conn = open_db_checked(db_path)?;
    let now = now_ts();
    let observer = NoopObserver;
    scope_service::add_member(&conn, scope_id, uid, role, &now, &observer)?;
    println!("added scope member scope_id={scope_id} uid={uid} role={role}");
    Ok(())
}

pub fn scope_list(db_path: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    for (id, kind) in scope_service::list(&conn)? {
        println!("id={id} type={kind}");
    }
    Ok(())
}

pub fn scope_members(db_path: &str, scope_id: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    for (uid, role) in scope_service::members(&conn, scope_id)? {
        println!("scope_id={scope_id} uid={uid} role={role}");
    }
    Ok(())
}

fn validate_dynamic_schema(v: &Value) -> Result<(), String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "schema file must be a JSON object".to_string())?;

    let schema_id = obj
        .get("schema_id")
        .and_then(|x| x.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "schema_id is required".to_string())?;

    let _version = obj
        .get("version")
        .and_then(|x| x.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "version is required".to_string())?;

    let class = obj
        .get("class")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "class is required: domain|user_context".to_string())?;

    if class != "domain" && class != "user_context" {
        return Err(format!(
            "invalid class for schema_id={schema_id}: {class} (expected domain|user_context)"
        ));
    }

    if class == "user_context" {
        let fields = obj
            .get("fields")
            .and_then(|x| x.as_array())
            .ok_or_else(|| "user_context schema requires fields[]".to_string())?;

        let has_ref_user = fields.iter().any(|f| {
            f.as_object()
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|n| n == "refUserId")
                .unwrap_or(false)
        });

        if !has_ref_user {
            return Err(format!(
                "user_context schema_id={schema_id} must include field name=refUserId"
            ));
        }
    }

    Ok(())
}

pub fn schema_validate(file: &str) -> Result<(), String> {
    let raw = fs::read_to_string(file).map_err(|e| format!("failed to read schema file: {e}"))?;
    let v: Value = serde_json::from_str(&raw).map_err(|e| format!("invalid schema json: {e}"))?;
    validate_dynamic_schema(&v)?;
    let schema_id = v.get("schema_id").and_then(|x| x.as_str()).unwrap_or("unknown");
    println!("schema valid schema_id={schema_id}");
    Ok(())
}

pub fn schema_register(db_path: &str, file: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    let raw = fs::read_to_string(file).map_err(|e| format!("failed to read schema file: {e}"))?;
    let v: Value = serde_json::from_str(&raw).map_err(|e| format!("invalid schema json: {e}"))?;
    validate_dynamic_schema(&v)?;

    let schema_id = v
        .get("schema_id")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "schema_id is required".to_string())?;
    let version = v
        .get("version")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "version is required".to_string())?;

    let now = now_ts();
    conn.execute(
        "INSERT INTO schema_registry (schema_id, version, schema_json, is_active, created_at)
         VALUES (?1, ?2, ?3, 1, ?4)
         ON CONFLICT(schema_id) DO UPDATE SET version=excluded.version, schema_json=excluded.schema_json, is_active=1",
        rusqlite::params![schema_id, version, raw, now],
    )
    .map_err(|e| format!("failed to register schema: {e}"))?;

    println!("registered schema schema_id={schema_id} version={version}");
    Ok(())
}

pub fn schema_list(db_path: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
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

    for row in rows {
        let (schema_id, version, is_active, created_at) = row.map_err(|e| e.to_string())?;
        println!(
            "schema_id={schema_id} version={version} active={} created_at={created_at}",
            is_active == 1
        );
    }

    Ok(())
}

pub fn ingest_event(
    db_path: &str,
    uid: &str,
    scope_id: &str,
    event_type: &str,
    file: &str,
    idempotency_key: Option<&str>,
) -> Result<(), String> {
    let mut conn = open_db_checked(db_path)?;
    let raw = fs::read_to_string(file).map_err(|e| format!("failed to read event file: {e}"))?;
    let payload: Value =
        serde_json::from_str(&raw).map_err(|e| format!("invalid json payload: {e}"))?;

    let event_id = new_id("evt");
    let now = now_ts();

    match ingest_service::ingest(
        &mut conn,
        ingest_service::IngestInput {
            uid,
            scope_id,
            event_type,
            payload: &payload,
            idempotency_key,
            event_id: &event_id,
            now: &now,
        },
    )? {
        ingest_service::IngestOutcome::Duplicate { idempotency_key } => {
            println!("duplicate event ignored idempotency_key={idempotency_key}");
        }
        ingest_service::IngestOutcome::Inserted {
            event_id,
            event_type,
        } => {
            println!("ingested event id={event_id} type={event_type}");
        }
    }

    Ok(())
}

pub fn query_latest(db_path: &str, uid: &str, scope_id: &str, as_json: bool) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    if let Some((event_id, event_type, event_ts)) = query_service::latest(&conn, uid, scope_id)? {
        if as_json {
            println!(
                "{}",
                json!({"event_id": event_id, "event_type": event_type, "event_ts": event_ts})
            );
        } else {
            println!("latest event_id={event_id} type={event_type} ts={event_ts}");
        }
    }
    Ok(())
}

pub fn query_metric(
    db_path: &str,
    uid: &str,
    scope_id: &str,
    key: Option<&str>,
    prefix: Option<&str>,
    as_json: bool,
) -> Result<(), String> {
    if key.is_none() && prefix.is_none() {
        return Err("query metric requires either --key or --prefix".to_string());
    }
    let conn = open_db_checked(db_path)?;
    let rows = query_service::metric(&conn, uid, scope_id, key, prefix)?;
    if as_json {
        let mapped: Vec<_> = rows
            .into_iter()
            .map(|(k, v, j)| json!({"key": k, "value": v, "json": j}))
            .collect();
        println!("{}", json!(mapped));
    } else {
        for (k, v, j) in rows {
            println!("metric key={k} value={v} json={j}");
        }
    }
    Ok(())
}

pub fn query_topk(
    db_path: &str,
    uid: &str,
    scope_id: &str,
    topic: &str,
    limit: usize,
    as_json: bool,
) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    let rows = query_service::topk(&conn, uid, scope_id, topic, limit)?;
    if as_json {
        let mapped: Vec<_> = rows
            .into_iter()
            .map(|(rank, item, weight)| json!({"rank": rank, "item": item, "weight": weight}))
            .collect();
        println!("{}", json!(mapped));
    } else {
        for (rank, item, weight) in rows {
            println!("rank={rank} item={item} weight={weight}");
        }
    }
    Ok(())
}
