use crate::domain::{DomainEvent, EventObserver};
use crate::repository::scope_repo;
use rusqlite::Connection;

pub fn create(
    conn: &Connection,
    scope_id: &str,
    scope_type: &str,
    now: &str,
) -> Result<(), String> {
    scope_repo::insert_scope(conn, scope_id, scope_type, now)
}

pub fn add_member(
    conn: &Connection,
    scope_id: &str,
    uid: &str,
    role: &str,
    now: &str,
    observer: &dyn EventObserver,
) -> Result<(), String> {
    scope_repo::insert_member(conn, scope_id, uid, role, now)?;
    observer.on_event(&DomainEvent::ScopeMemberAdded {
        scope_id: scope_id.to_string(),
        uid: uid.to_string(),
    })?;
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    scope_repo::list_scopes(conn)
}

pub fn members(conn: &Connection, scope_id: &str) -> Result<Vec<(String, String)>, String> {
    scope_repo::list_members(conn, scope_id)
}
