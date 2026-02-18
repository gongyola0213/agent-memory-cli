use crate::db;
use crate::domain::NoopObserver;
use crate::service::{identity_service, scope_service, user_service};
use rusqlite::Connection;
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
