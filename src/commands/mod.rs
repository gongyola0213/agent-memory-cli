use crate::db;
use crate::domain::schema::{validate_schema_def, SchemaDef};
use crate::domain::NoopObserver;
use crate::repository::{dynamic_table_repo, projection_outbox_repo, schema_registry_repo};
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

pub fn user_merge(db_path: &str, from_uid: &str, to_uid: &str) -> Result<(), String> {
    if from_uid == to_uid {
        return Err("--from and --to must be different users".to_string());
    }

    let mut conn = open_db_checked(db_path)?;
    let now = now_ts();
    user_service::merge(&mut conn, from_uid, to_uid, &now)?;
    println!("merged user from_uid={from_uid} to_uid={to_uid}");
    Ok(())
}

pub fn user_delete(
    db_path: &str,
    uid: &str,
    mode: &str,
    force: bool,
    dry_run: bool,
) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    let counts = user_service::ref_counts(&conn, uid)?;

    if dry_run {
        println!(
            "delete preflight uid={uid} mode={mode} identities={} scope_members={} events={} state={} metrics={} topk={}",
            counts.identities,
            counts.scope_members,
            counts.events,
            counts.state,
            counts.metrics,
            counts.topk,
        );
        return Ok(());
    }

    let now = now_ts();
    match mode {
        "soft" => {
            user_service::delete_soft(&conn, uid, &now)?;
            println!("deleted user uid={uid} mode=soft");
            Ok(())
        }
        "hard" => {
            user_service::delete_hard(&conn, uid, &now, force)?;
            println!("deleted user uid={uid} mode=hard");
            Ok(())
        }
        _ => Err("invalid --mode. expected: soft|hard".to_string()),
    }
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

fn parse_and_validate_schema(file: &str) -> Result<(SchemaDef, String), String> {
    let raw = fs::read_to_string(file).map_err(|e| format!("failed to read schema file: {e}"))?;
    let def: SchemaDef =
        serde_json::from_str(&raw).map_err(|e| format!("invalid schema json: {e}"))?;
    validate_schema_def(&def)?;
    Ok((def, raw))
}

pub fn schema_validate(file: &str) -> Result<(), String> {
    let (def, _) = parse_and_validate_schema(file)?;
    println!("schema valid schema_id={}", def.schema_id);
    Ok(())
}

pub fn schema_register(db_path: &str, file: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    let (def, raw) = parse_and_validate_schema(file)?;

    let now = now_ts();
    schema_registry_repo::upsert(&conn, &def, &raw, &now)?;
    let table_name = dynamic_table_repo::create_table_for_schema(&conn, &def)?;

    let outbox_id = new_id("outbox");
    let payload = json!({
        "event": "schema.registered",
        "schema_id": &def.schema_id,
        "version": &def.version,
        "table": table_name,
    })
    .to_string();
    projection_outbox_repo::enqueue(
        &conn,
        &outbox_id,
        "schema",
        "schema.registered",
        &payload,
        &now,
    )?;

    println!(
        "registered schema schema_id={} version={} table={}",
        def.schema_id,
        def.version,
        dynamic_table_repo::table_name_for(&def)
    );
    Ok(())
}

pub fn schema_list(db_path: &str) -> Result<(), String> {
    let conn = open_db_checked(db_path)?;
    let rows = schema_registry_repo::list(&conn)?;

    for (schema_id, version, is_active, created_at) in rows {
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
