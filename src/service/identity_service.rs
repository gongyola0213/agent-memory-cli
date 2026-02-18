use crate::domain::{DomainEvent, EventObserver};
use crate::repository::{identity_repo, user_repo};
use rusqlite::Connection;

pub fn link(
    conn: &Connection,
    identity_id: &str,
    uid: &str,
    channel: &str,
    channel_user_id: &str,
    now: &str,
    observer: &dyn EventObserver,
) -> Result<(), String> {
    if user_repo::get_name(conn, uid)?.is_none() {
        return Err(format!("user not found: {uid}"));
    }
    identity_repo::insert(conn, identity_id, uid, channel, channel_user_id, now)?;
    observer.on_event(&DomainEvent::IdentityLinked {
        uid: uid.to_string(),
        channel: channel.to_string(),
    })?;
    Ok(())
}

pub fn resolve(
    conn: &Connection,
    channel: &str,
    channel_user_id: &str,
) -> Result<Option<String>, String> {
    identity_repo::resolve_uid(conn, channel, channel_user_id)
}

pub fn unlink(conn: &Connection, channel: &str, channel_user_id: &str) -> Result<usize, String> {
    identity_repo::delete(conn, channel, channel_user_id)
}
