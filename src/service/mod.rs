pub mod identity_service;
pub mod ingest_service;
pub mod query_service;
pub mod scope_service;
pub mod user_service;

#[cfg(test)]
mod tests {
    use super::{identity_service, scope_service, user_service};
    use crate::domain::NoopObserver;
    use rusqlite::Connection;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../specs/SCHEMA_SQLITE_V01.sql"))
            .unwrap();
        conn
    }

    #[test]
    fn user_service_create_show_update_list() {
        let conn = setup_conn();
        let observer = NoopObserver;

        user_service::create(&conn, "u_1", "Yongseong", "100", &observer).unwrap();
        let name = user_service::show(&conn, "u_1").unwrap();
        assert_eq!(name.as_deref(), Some("Yongseong"));

        user_service::update(&conn, "u_1", "Yong", "101").unwrap();
        let name = user_service::show(&conn, "u_1").unwrap();
        assert_eq!(name.as_deref(), Some("Yong"));

        let users = user_service::list(&conn).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].0, "u_1");
    }

    #[test]
    fn identity_service_link_and_resolve() {
        let conn = setup_conn();
        let observer = NoopObserver;

        user_service::create(&conn, "u_1", "Yongseong", "100", &observer).unwrap();
        identity_service::link(
            &conn,
            "ident_1",
            "u_1",
            "telegram",
            "7992342261",
            "101",
            &observer,
        )
        .unwrap();

        let uid = identity_service::resolve(&conn, "telegram", "7992342261").unwrap();
        assert_eq!(uid.as_deref(), Some("u_1"));
    }

    #[test]
    fn scope_service_create_add_member_members() {
        let conn = setup_conn();
        let observer = NoopObserver;

        user_service::create(&conn, "u_1", "Yongseong", "100", &observer).unwrap();
        scope_service::create(&conn, "shared:couple", "shared", "101").unwrap();
        scope_service::add_member(&conn, "shared:couple", "u_1", "member", "102", &observer)
            .unwrap();

        let scopes = scope_service::list(&conn).unwrap();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].0, "shared:couple");

        let members = scope_service::members(&conn, "shared:couple").unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0, "u_1");
    }
}
