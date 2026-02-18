use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::Connection;
use tempfile::tempdir;

fn bin() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("agent-memory-cli"))
}

#[test]
fn doctor_command_returns_ready_message() {
    let mut cmd = bin();
    cmd.arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("agent-memory-cli is ready"));
}

#[test]
fn top_level_help_includes_spec_command_groups() {
    let mut cmd = bin();
    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("user")
            .and(predicate::str::contains("identity"))
            .and(predicate::str::contains("scope"))
            .and(predicate::str::contains("schema"))
            .and(predicate::str::contains("ingest"))
            .and(predicate::str::contains("query"))
            .and(predicate::str::contains("state"))
            .and(predicate::str::contains("admin")),
    );
}

#[test]
fn user_help_supports_create_list_show_update() {
    let mut cmd = bin();
    cmd.args(["user", "--help"]).assert().success().stdout(
        predicate::str::contains("create")
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("show"))
            .and(predicate::str::contains("update")),
    );
}

#[test]
fn identity_help_supports_link_resolve_unlink() {
    let mut cmd = bin();
    cmd.args(["identity", "--help"]).assert().success().stdout(
        predicate::str::contains("link")
            .and(predicate::str::contains("resolve"))
            .and(predicate::str::contains("unlink")),
    );
}

#[test]
fn scope_help_supports_create_add_member_list_members() {
    let mut cmd = bin();
    cmd.args(["scope", "--help"]).assert().success().stdout(
        predicate::str::contains("create")
            .and(predicate::str::contains("add-member"))
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("members")),
    );
}

#[test]
fn schema_help_supports_register_list_validate() {
    let mut cmd = bin();
    cmd.args(["schema", "--help"]).assert().success().stdout(
        predicate::str::contains("register")
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("validate")),
    );
}

#[test]
fn ingest_help_supports_event_and_batch() {
    let mut cmd = bin();
    cmd.args(["ingest", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("event").and(predicate::str::contains("batch")));
}

#[test]
fn query_help_supports_latest_metric_topk() {
    let mut cmd = bin();
    cmd.args(["query", "--help"]).assert().success().stdout(
        predicate::str::contains("latest")
            .and(predicate::str::contains("metric"))
            .and(predicate::str::contains("topk")),
    );
}

#[test]
fn state_help_supports_get_set_delete() {
    let mut cmd = bin();
    cmd.args(["state", "--help"]).assert().success().stdout(
        predicate::str::contains("get")
            .and(predicate::str::contains("set"))
            .and(predicate::str::contains("delete")),
    );
}

#[test]
fn admin_help_supports_migrate_reindex_compact_archive() {
    let mut cmd = bin();
    cmd.args(["admin", "--help"]).assert().success().stdout(
        predicate::str::contains("migrate")
            .and(predicate::str::contains("reindex"))
            .and(predicate::str::contains("compact"))
            .and(predicate::str::contains("archive")),
    );
}

#[test]
fn user_create_requires_name_flag() {
    let mut cmd = bin();
    cmd.args(["user", "create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--name"));
}

#[test]
fn identity_link_requires_uid_channel_and_channel_user_id() {
    let mut cmd = bin();
    cmd.args(["identity", "link", "--uid", "u1", "--channel", "telegram"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--channel-user-id"));
}

#[test]
fn scope_create_requires_id_and_type() {
    let mut cmd = bin();
    cmd.args(["scope", "create", "--id", "shared:couple"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--type"));
}

#[test]
fn user_create_with_name_runs() {
    let mut cmd = bin();
    cmd.args(["user", "create", "--name", "Yongseong"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "TODO: implement user::create name=Yongseong",
        ));
}

#[test]
fn admin_migrate_creates_sqlite_db_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("agent-memory-test.db");
    let db_str = db_path.to_string_lossy().to_string();

    let mut cmd = bin();
    cmd.args(["--db", &db_str, "admin", "migrate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("migrated schema to"));

    assert!(db_path.exists());
}

#[test]
fn admin_migrate_is_idempotent() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("agent-memory-idempotent.db");
    let db_str = db_path.to_string_lossy().to_string();

    let mut first = bin();
    first
        .args(["--db", &db_str, "admin", "migrate"])
        .assert()
        .success();

    let mut second = bin();
    second
        .args(["--db", &db_str, "admin", "migrate"])
        .assert()
        .success();
}

#[test]
fn admin_migrate_creates_expected_core_tables() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("agent-memory-schema.db");
    let db_str = db_path.to_string_lossy().to_string();

    let mut cmd = bin();
    cmd.args(["--db", &db_str, "admin", "migrate"])
        .assert()
        .success();

    let conn = Connection::open(&db_path).unwrap();
    let expected = [
        "users",
        "user_identities",
        "scopes",
        "scope_members",
        "events",
        "state",
        "metrics",
        "topk",
        "schema_registry",
    ];

    for table in expected {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(1) FROM sqlite_master WHERE type='table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1, "expected table {table} to exist");
    }
}

#[test]
fn db_flag_works_after_subcommand_too() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("agent-memory-postfix-flag.db");
    let db_str = db_path.to_string_lossy().to_string();

    let mut cmd = bin();
    cmd.args(["admin", "migrate", "--db", &db_str])
        .assert()
        .success();

    assert!(db_path.exists());
}
