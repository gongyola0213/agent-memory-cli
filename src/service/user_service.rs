use crate::domain::{DomainEvent, EventObserver};
use crate::repository::user_repo;
use rusqlite::Connection;

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
