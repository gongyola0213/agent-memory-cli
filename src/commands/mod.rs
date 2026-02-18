use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn doctor() {
    println!("agent-memory-cli is ready");
}

pub fn todo(group: &str, command: &str) {
    println!("TODO: implement {group}::{command}");
}

pub fn admin_migrate(db_path: &str) -> Result<(), String> {
    let db = Path::new(db_path);
    if let Some(parent) = db.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create db directory {}: {e}", parent.display()))?;
        }
    }

    let conn =
        Connection::open(db).map_err(|e| format!("failed to open db {}: {e}", db.display()))?;

    let schema_sql = include_str!("../../specs/SCHEMA_SQLITE_V01.sql");
    conn.execute_batch(schema_sql)
        .map_err(|e| format!("migration failed: {e}"))?;

    println!("migrated schema to {}", db.display());
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
    Connection::open(db_path).map_err(|e| format!("failed to open db: {e}"))
}

pub fn user_create(db_path: &str, name: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let uid = new_id("u");
    let now = now_ts();
    conn.execute(
        "INSERT INTO users (uid, display_name, status, created_at, updated_at) VALUES (?1, ?2, 'active', ?3, ?3)",
        params![uid, name, now],
    )
    .map_err(|e| format!("failed to create user: {e}"))?;
    println!("created user uid={uid} name={name}");
    Ok(())
}

pub fn user_list(db_path: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let mut stmt = conn
        .prepare("SELECT uid, display_name FROM users ORDER BY created_at DESC")
        .map_err(|e| format!("failed to prepare user list: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let uid: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((uid, name))
        })
        .map_err(|e| format!("failed to list users: {e}"))?;

    for row in rows {
        let (uid, name) = row.map_err(|e| format!("failed to read user row: {e}"))?;
        println!("uid={uid} name={name}");
    }
    Ok(())
}

pub fn user_show(db_path: &str, uid: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let name: Option<String> = conn
        .query_row(
            "SELECT display_name FROM users WHERE uid = ?1",
            params![uid],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("failed to query user: {e}"))?;

    match name {
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
    let n = conn
        .execute(
            "UPDATE users SET display_name = ?1, updated_at = ?2 WHERE uid = ?3",
            params![name, now, uid],
        )
        .map_err(|e| format!("failed to update user: {e}"))?;
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

    let exists: Option<String> = conn
        .query_row(
            "SELECT uid FROM users WHERE uid = ?1",
            params![uid],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("failed to check user: {e}"))?;
    if exists.is_none() {
        return Err(format!("user not found: {uid}"));
    }

    let id = new_id("ident");
    let now = now_ts();
    conn.execute(
        "INSERT INTO user_identities (identity_id, uid, channel, channel_user_id, is_verified, confidence, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, 1.0, ?5, ?5)",
        params![id, uid, channel, channel_user_id, now],
    )
    .map_err(|e| format!("failed to link identity: {e}"))?;

    println!("linked identity uid={uid} channel={channel} channel_user_id={channel_user_id}");
    Ok(())
}

pub fn identity_resolve(db_path: &str, channel: &str, channel_user_id: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let uid: Option<String> = conn
        .query_row(
            "SELECT uid FROM user_identities WHERE channel = ?1 AND channel_user_id = ?2",
            params![channel, channel_user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("failed to resolve identity: {e}"))?;

    match uid {
        Some(uid) => {
            println!("resolved uid={uid} channel={channel} channel_user_id={channel_user_id}");
            Ok(())
        }
        None => Err(format!("identity not found: {channel}:{channel_user_id}")),
    }
}

pub fn identity_unlink(db_path: &str, channel: &str, channel_user_id: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let n = conn
        .execute(
            "DELETE FROM user_identities WHERE channel = ?1 AND channel_user_id = ?2",
            params![channel, channel_user_id],
        )
        .map_err(|e| format!("failed to unlink identity: {e}"))?;
    if n == 0 {
        return Err(format!("identity not found: {channel}:{channel_user_id}"));
    }
    println!("unlinked identity channel={channel} channel_user_id={channel_user_id}");
    Ok(())
}

pub fn scope_create(db_path: &str, scope_id: &str, scope_type: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let now = now_ts();
    conn.execute(
        "INSERT INTO scopes (scope_id, scope_type, created_at) VALUES (?1, ?2, ?3)",
        params![scope_id, scope_type, now],
    )
    .map_err(|e| format!("failed to create scope: {e}"))?;
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
    conn.execute(
        "INSERT INTO scope_members (scope_id, uid, role, added_at) VALUES (?1, ?2, ?3, ?4)",
        params![scope_id, uid, role, now],
    )
    .map_err(|e| format!("failed to add member: {e}"))?;
    println!("added scope member scope_id={scope_id} uid={uid} role={role}");
    Ok(())
}

pub fn scope_list(db_path: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let mut stmt = conn
        .prepare("SELECT scope_id, scope_type FROM scopes ORDER BY created_at DESC")
        .map_err(|e| format!("failed to prepare scope list: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let kind: String = row.get(1)?;
            Ok((id, kind))
        })
        .map_err(|e| format!("failed to list scopes: {e}"))?;

    for row in rows {
        let (id, kind) = row.map_err(|e| format!("failed to read scope row: {e}"))?;
        println!("id={id} type={kind}");
    }
    Ok(())
}

pub fn scope_members(db_path: &str, scope_id: &str) -> Result<(), String> {
    let conn = open_and_migrate(db_path)?;
    let mut stmt = conn
        .prepare("SELECT uid, role FROM scope_members WHERE scope_id = ?1 ORDER BY added_at DESC")
        .map_err(|e| format!("failed to prepare scope members: {e}"))?;

    let rows = stmt
        .query_map(params![scope_id], |row| {
            let uid: String = row.get(0)?;
            let role: String = row.get(1)?;
            Ok((uid, role))
        })
        .map_err(|e| format!("failed to list scope members: {e}"))?;

    for row in rows {
        let (uid, role) = row.map_err(|e| format!("failed to read scope member row: {e}"))?;
        println!("scope_id={scope_id} uid={uid} role={role}");
    }
    Ok(())
}
