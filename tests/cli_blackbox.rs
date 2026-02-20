use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

fn bin() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("agent-memory-cli"))
}

fn migrate_db(db: &str) {
    let mut cmd = bin();
    cmd.args(["--db", db, "admin", "migrate"])
        .assert()
        .success();
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
fn user_help_supports_create_list_show_update_merge_delete() {
    let mut cmd = bin();
    cmd.args(["user", "--help"]).assert().success().stdout(
        predicate::str::contains("create")
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("show"))
            .and(predicate::str::contains("update"))
            .and(predicate::str::contains("merge"))
            .and(predicate::str::contains("delete")),
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
fn schema_validate_domain_without_ref_user_id_succeeds() {
    let dir = tempdir().unwrap();
    let schema_path = dir.path().join("domain.schema.json");
    fs::write(
        &schema_path,
        r#"{
  "schema_id": "place.v1",
  "version": "1",
  "class": "domain",
  "fields": [{"name":"placeId","type":"string"}]
}"#,
    )
    .unwrap();

    let mut cmd = bin();
    cmd.args([
        "schema",
        "validate",
        "--file",
        schema_path.to_string_lossy().as_ref(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("schema valid"));
}

#[test]
fn schema_validate_user_context_without_ref_user_id_fails() {
    let dir = tempdir().unwrap();
    let schema_path = dir.path().join("user-context-invalid.schema.json");
    fs::write(
        &schema_path,
        r#"{
  "schema_id": "restaurant.rating.v1",
  "version": "1",
  "class": "user_context",
  "fields": [{"name":"restaurantId","type":"string"}]
}"#,
    )
    .unwrap();

    let mut cmd = bin();
    cmd.args([
        "schema",
        "validate",
        "--file",
        schema_path.to_string_lossy().as_ref(),
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("refUserId"));
}

#[test]
fn schema_validate_fails_when_fields_missing() {
    let dir = tempdir().unwrap();
    let schema_path = dir.path().join("domain-no-fields.schema.json");
    fs::write(
        &schema_path,
        r#"{
  "schema_id": "place.v1",
  "version": "1",
  "class": "domain"
}"#,
    )
    .unwrap();

    let mut cmd = bin();
    cmd.args([
        "schema",
        "validate",
        "--file",
        schema_path.to_string_lossy().as_ref(),
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("invalid schema json"));
}

#[test]
fn schema_validate_fails_on_invalid_class() {
    let dir = tempdir().unwrap();
    let schema_path = dir.path().join("invalid-class.schema.json");
    fs::write(
        &schema_path,
        r#"{
  "schema_id": "place.v1",
  "version": "1",
  "class": "entity",
  "fields": [{"name":"placeId","type":"string"}]
}"#,
    )
    .unwrap();

    let mut cmd = bin();
    cmd.args([
        "schema",
        "validate",
        "--file",
        schema_path.to_string_lossy().as_ref(),
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("invalid schema json"));
}

#[test]
fn schema_validate_fails_on_duplicate_field_names() {
    let dir = tempdir().unwrap();
    let schema_path = dir.path().join("duplicate-fields.schema.json");
    fs::write(
        &schema_path,
        r#"{
  "schema_id": "restaurant.rating.v1",
  "version": "1",
  "class": "user_context",
  "fields": [
    {"name":"refUserId","type":"string"},
    {"name":"score","type":"number"},
    {"name":"score","type":"number"}
  ]
}"#,
    )
    .unwrap();

    let mut cmd = bin();
    cmd.args([
        "schema",
        "validate",
        "--file",
        schema_path.to_string_lossy().as_ref(),
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("duplicate field"));
}

#[test]
fn schema_register_and_list_works() {
    let dir = tempdir().unwrap();
    let db_str = dir
        .path()
        .join("schema-register.db")
        .to_string_lossy()
        .to_string();
    migrate_db(&db_str);

    let schema_path = dir.path().join("user-context-valid.schema.json");
    fs::write(
        &schema_path,
        r#"{
  "schema_id": "restaurant.rating.v1",
  "version": "1",
  "class": "user_context",
  "fields": [
    {"name":"refUserId","type":"string"},
    {"name":"restaurantId","type":"string"},
    {"name":"score","type":"number"}
  ]
}"#,
    )
    .unwrap();

    let mut register = bin();
    register
        .args([
            "--db",
            &db_str,
            "schema",
            "register",
            "--file",
            schema_path.to_string_lossy().as_ref(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("registered schema"));

    let mut list = bin();
    list.args(["--db", &db_str, "schema", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("schema_id=restaurant.rating.v1"));

    let conn = Connection::open(dir.path().join("schema-register.db")).unwrap();
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM sqlite_master WHERE type='table' AND name='dyn_restaurant_rating_v1_v1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(exists, 1);
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
    let dir = tempdir().unwrap();
    let db_str = dir
        .path()
        .join("create-user.db")
        .to_string_lossy()
        .to_string();

    migrate_db(&db_str);

    let mut cmd = bin();
    cmd.args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created user uid="));
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
        "dynamic_records",
        "projection_outbox",
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

#[test]
fn user_create_then_list_includes_user() {
    let dir = tempdir().unwrap();
    let db_str = dir
        .path()
        .join("list-user.db")
        .to_string_lossy()
        .to_string();
    migrate_db(&db_str);

    let mut create = bin();
    create
        .args(["--db", &db_str, "user", "create", "--name", "Irene"])
        .assert()
        .success();

    let mut list = bin();
    list.args(["--db", &db_str, "user", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name=Irene"));
}

#[test]
fn identity_link_and_resolve_flow() {
    let dir = tempdir().unwrap();
    let db_str = dir
        .path()
        .join("identity-flow.db")
        .to_string_lossy()
        .to_string();
    migrate_db(&db_str);

    let mut create = bin();
    create
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();

    let conn = Connection::open(dir.path().join("identity-flow.db")).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |row| row.get(0))
        .unwrap();

    let mut link = bin();
    link.args([
        "--db",
        &db_str,
        "identity",
        "link",
        "--uid",
        &uid,
        "--channel",
        "telegram",
        "--channel-user-id",
        "7992342261",
    ])
    .assert()
    .success();

    let mut resolve = bin();
    resolve
        .args([
            "--db",
            &db_str,
            "identity",
            "resolve",
            "--channel",
            "telegram",
            "--channel-user-id",
            "7992342261",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("resolved uid={uid}")));
}

#[test]
fn scope_create_add_member_and_list_members_flow() {
    let dir = tempdir().unwrap();
    let db_str = dir
        .path()
        .join("scope-flow.db")
        .to_string_lossy()
        .to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();

    let conn = Connection::open(dir.path().join("scope-flow.db")).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |row| row.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "shared:couple",
            "--type",
            "shared",
        ])
        .assert()
        .success();

    let mut add_member = bin();
    add_member
        .args([
            "--db",
            &db_str,
            "scope",
            "add-member",
            "--id",
            "shared:couple",
            "--uid",
            &uid,
        ])
        .assert()
        .success();

    let mut members = bin();
    members
        .args(["--db", &db_str, "scope", "members", "--id", "shared:couple"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("uid={uid}")));
}

#[test]
fn ingest_meal_rated_updates_food_topk() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("ingest-food.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();

    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |row| row.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let event_file = dir.path().join("meal.json");
    fs::write(&event_file, r#"{"cuisine":"korean"}"#).unwrap();
    let event_file_str = event_file.to_string_lossy().to_string();

    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &event_file_str,
        ])
        .assert()
        .success();

    let mut topk = bin();
    topk.args([
        "--db",
        &db_str,
        "query",
        "topk",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--topic",
        "food_pref",
        "--limit",
        "3",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("item=korean"));
}

#[test]
fn ingest_rejects_invalid_json_payload() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("invalid-json.db");
    let db_str = db_path.to_string_lossy().to_string();

    migrate_db(&db_str);

    let bad_file = dir.path().join("bad.json");
    fs::write(&bad_file, "{not-json}").unwrap();
    let bad_file_str = bad_file.to_string_lossy().to_string();

    let mut cmd = bin();
    cmd.args([
        "--db",
        &db_str,
        "ingest",
        "event",
        "--uid",
        "u_1",
        "--scope",
        "s_1",
        "--type",
        "meal.rated",
        "--file",
        &bad_file_str,
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("invalid json payload"));
}

#[test]
fn ingest_meal_rated_requires_cuisine() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("missing-cuisine.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let event_file = dir.path().join("meal-no-cuisine.json");
    fs::write(&event_file, r#"{"rating":5}"#).unwrap();
    let event_file_str = event_file.to_string_lossy().to_string();

    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &event_file_str,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "meal.rated requires string field: cuisine",
        ));
}

#[test]
fn ingest_expense_logged_requires_category() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("missing-category.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let event_file = dir.path().join("expense-no-category.json");
    fs::write(&event_file, r#"{"amount":12000}"#).unwrap();
    let event_file_str = event_file.to_string_lossy().to_string();

    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "expense.logged",
            "--file",
            &event_file_str,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "expense.logged requires string field: category",
        ));
}

#[test]
fn identity_unlink_then_resolve_fails() {
    let dir = tempdir().unwrap();
    let db_str = dir
        .path()
        .join("identity-unlink.db")
        .to_string_lossy()
        .to_string();
    migrate_db(&db_str);

    let mut create = bin();
    create
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();
    let conn = Connection::open(dir.path().join("identity-unlink.db")).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut link = bin();
    link.args([
        "--db",
        &db_str,
        "identity",
        "link",
        "--uid",
        &uid,
        "--channel",
        "telegram",
        "--channel-user-id",
        "123",
    ])
    .assert()
    .success();

    let mut unlink = bin();
    unlink
        .args([
            "--db",
            &db_str,
            "identity",
            "unlink",
            "--channel",
            "telegram",
            "--channel-user-id",
            "123",
        ])
        .assert()
        .success();

    let mut resolve = bin();
    resolve
        .args([
            "--db",
            &db_str,
            "identity",
            "resolve",
            "--channel",
            "telegram",
            "--channel-user-id",
            "123",
        ])
        .assert()
        .failure();
}

#[test]
fn topk_orders_by_frequency() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("topk-order.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let e1 = dir.path().join("m1.json");
    let e2 = dir.path().join("m2.json");
    let e3 = dir.path().join("m3.json");
    fs::write(&e1, r#"{"cuisine":"korean"}"#).unwrap();
    fs::write(&e2, r#"{"cuisine":"korean"}"#).unwrap();
    fs::write(&e3, r#"{"cuisine":"japanese"}"#).unwrap();

    for f in [&e1, &e2, &e3] {
        let mut ingest = bin();
        ingest
            .args([
                "--db",
                &db_str,
                "ingest",
                "event",
                "--uid",
                &uid,
                "--scope",
                "private:test",
                "--type",
                "meal.rated",
                "--file",
                &f.to_string_lossy(),
            ])
            .assert()
            .success();
    }

    let mut topk = bin();
    topk.args([
        "--db",
        &db_str,
        "query",
        "topk",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--topic",
        "food_pref",
        "--limit",
        "2",
    ])
    .assert()
    .success()
    .stdout(
        predicate::str::contains("rank=1 item=korean")
            .and(predicate::str::contains("rank=2 item=japanese")),
    );
}

#[test]
fn query_latest_returns_most_recent_event() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("latest.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let meal = dir.path().join("meal.json");
    let exp = dir.path().join("exp.json");
    fs::write(&meal, r#"{"cuisine":"korean"}"#).unwrap();
    fs::write(&exp, r#"{"category":"coffee"}"#).unwrap();

    let mut i1 = bin();
    i1.args([
        "--db",
        &db_str,
        "ingest",
        "event",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--type",
        "meal.rated",
        "--file",
        &meal.to_string_lossy(),
    ])
    .assert()
    .success();
    let mut i2 = bin();
    i2.args([
        "--db",
        &db_str,
        "ingest",
        "event",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--type",
        "expense.logged",
        "--file",
        &exp.to_string_lossy(),
    ])
    .assert()
    .success();

    let mut latest = bin();
    latest
        .args([
            "--db",
            &db_str,
            "query",
            "latest",
            "--uid",
            &uid,
            "--scope",
            "private:test",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("type=expense.logged"));
}

#[test]
fn query_topk_respects_limit() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("topk-limit.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    for (name, cuisine) in [("a", "korean"), ("b", "japanese"), ("c", "thai")] {
        let f = dir.path().join(format!("{name}.json"));
        fs::write(&f, format!("{{\"cuisine\":\"{cuisine}\"}}")).unwrap();
        let mut ingest = bin();
        ingest
            .args([
                "--db",
                &db_str,
                "ingest",
                "event",
                "--uid",
                &uid,
                "--scope",
                "private:test",
                "--type",
                "meal.rated",
                "--file",
                &f.to_string_lossy(),
            ])
            .assert()
            .success();
    }

    let mut topk = bin();
    topk.args([
        "--db",
        &db_str,
        "query",
        "topk",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--topic",
        "food_pref",
        "--limit",
        "1",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("rank=1").and(predicate::str::contains("rank=2").not()));
}

#[test]
fn ingest_with_same_idempotency_key_is_ignored() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("idempotent.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let event_file = dir.path().join("meal.json");
    fs::write(&event_file, r#"{"cuisine":"korean"}"#).unwrap();
    let event_file_str = event_file.to_string_lossy().to_string();

    let mut first = bin();
    first
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &event_file_str,
            "--idempotency-key",
            "k1",
        ])
        .assert()
        .success();

    let mut second = bin();
    second
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &event_file_str,
            "--idempotency-key",
            "k1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("duplicate event ignored"));

    let event_count: i64 = conn
        .query_row("SELECT COUNT(1) FROM events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(event_count, 1);
}

#[test]
fn failed_ingest_does_not_insert_event() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("rollback.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let bad = dir.path().join("bad-meal.json");
    fs::write(&bad, r#"{"rating":5}"#).unwrap();

    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &bad.to_string_lossy(),
        ])
        .assert()
        .failure();

    let event_count: i64 = conn
        .query_row("SELECT COUNT(1) FROM events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(event_count, 0);
}

#[test]
fn ingest_expense_logged_updates_spend_category_topk() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("ingest-expense.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();

    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |row| row.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let event_file = dir.path().join("expense.json");
    fs::write(&event_file, r#"{"category":"coffee"}"#).unwrap();

    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "expense.logged",
            "--file",
            &event_file.to_string_lossy(),
        ])
        .assert()
        .success();

    let mut topk = bin();
    topk.args([
        "--db",
        &db_str,
        "query",
        "topk",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--topic",
        "spend_category",
        "--limit",
        "3",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("item=coffee"));
}

#[test]
fn query_metric_by_key_and_prefix() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metric-query.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();

    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |row| row.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let event_file = dir.path().join("meal.json");
    fs::write(&event_file, r#"{"cuisine":"korean"}"#).unwrap();

    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &event_file.to_string_lossy(),
        ])
        .assert()
        .success();

    let mut by_key = bin();
    by_key
        .args([
            "--db",
            &db_str,
            "query",
            "metric",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--key",
            "counter:food_pref:korean",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("counter:food_pref:korean"));

    let mut by_prefix = bin();
    by_prefix
        .args([
            "--db",
            &db_str,
            "query",
            "metric",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--prefix",
            "counter:food_pref:",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("counter:food_pref:korean"));
}

#[test]
fn query_metric_requires_key_or_prefix() {
    let mut cmd = bin();
    cmd.args(["query", "metric", "--uid", "u_1", "--scope", "private:test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "query metric requires either --key or --prefix",
        ));
}

#[test]
fn ingest_request_logged_updates_request_pattern_topk() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("request-pattern.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();

    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |row| row.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let event_file = dir.path().join("request.json");
    fs::write(&event_file, r#"{"pattern":"restaurant_reco"}"#).unwrap();

    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "request.logged",
            "--file",
            &event_file.to_string_lossy(),
        ])
        .assert()
        .success();

    let mut topk = bin();
    topk.args([
        "--db",
        &db_str,
        "query",
        "topk",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--topic",
        "request_pattern",
        "--limit",
        "3",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("item=restaurant_reco"));
}

#[test]
fn ingest_request_logged_requires_pattern() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("request-missing-pattern.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Yongseong"])
        .assert()
        .success();

    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |row| row.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let event_file = dir.path().join("request-bad.json");
    fs::write(&event_file, r#"{"foo":"bar"}"#).unwrap();

    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "request.logged",
            "--file",
            &event_file.to_string_lossy(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "request.logged requires string field: pattern",
        ));
}

#[test]
fn doctor_supports_json_output() {
    let dir = tempdir().unwrap();
    let db_str = dir
        .path()
        .join("doctor-json.db")
        .to_string_lossy()
        .to_string();
    migrate_db(&db_str);

    let mut cmd = bin();
    cmd.args(["--db", &db_str, "--json", "doctor"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"ok\":true")
                .and(predicate::str::contains("\"schema_initialized\":true")),
        );
}

#[test]
fn query_topk_supports_json_output() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("topk-json.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let f = dir.path().join("m.json");
    fs::write(&f, r#"{"cuisine":"korean"}"#).unwrap();
    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &f.to_string_lossy(),
        ])
        .assert()
        .success();

    let mut topk = bin();
    topk.args([
        "--db",
        &db_str,
        "--json",
        "query",
        "topk",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--topic",
        "food_pref",
        "--limit",
        "1",
    ])
    .assert()
    .success()
    .stdout(
        predicate::str::contains("\"rank\":1").and(predicate::str::contains("\"item\":\"korean\"")),
    );
}

#[test]
fn query_latest_supports_json_output() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("latest-json.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let f = dir.path().join("meal.json");
    fs::write(&f, r#"{"cuisine":"korean"}"#).unwrap();
    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &f.to_string_lossy(),
        ])
        .assert()
        .success();

    let mut latest = bin();
    latest
        .args([
            "--db",
            &db_str,
            "--json",
            "query",
            "latest",
            "--uid",
            &uid,
            "--scope",
            "private:test",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"event_id\":")
                .and(predicate::str::contains("\"event_type\":\"meal.rated\""))
                .and(predicate::str::contains("\"event_ts\":")),
        );
}

#[test]
fn query_metric_supports_json_key_output() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metric-json-key.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let f = dir.path().join("meal.json");
    fs::write(&f, r#"{"cuisine":"korean"}"#).unwrap();
    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "meal.rated",
            "--file",
            &f.to_string_lossy(),
        ])
        .assert()
        .success();

    let mut metric = bin();
    metric
        .args([
            "--db",
            &db_str,
            "--json",
            "query",
            "metric",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--key",
            "counter:food_pref:korean",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"key\":\"counter:food_pref:korean\"")
                .and(predicate::str::contains("\"value\":1.0")),
        );
}

#[test]
fn query_metric_supports_json_prefix_output() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metric-json-prefix.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let f = dir.path().join("expense.json");
    fs::write(&f, r#"{"category":"coffee"}"#).unwrap();
    let mut ingest = bin();
    ingest
        .args([
            "--db",
            &db_str,
            "ingest",
            "event",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--type",
            "expense.logged",
            "--file",
            &f.to_string_lossy(),
        ])
        .assert()
        .success();

    let mut metric = bin();
    metric
        .args([
            "--db",
            &db_str,
            "--json",
            "query",
            "metric",
            "--uid",
            &uid,
            "--scope",
            "private:test",
            "--prefix",
            "counter:spend_category:",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("counter:spend_category:coffee"));
}

#[test]
fn query_topk_json_empty_returns_array() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("topk-empty-json.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let mut create_user = bin();
    create_user
        .args(["--db", &db_str, "user", "create", "--name", "Y"])
        .assert()
        .success();
    let conn = Connection::open(&db_path).unwrap();
    let uid: String = conn
        .query_row("SELECT uid FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let mut create_scope = bin();
    create_scope
        .args([
            "--db",
            &db_str,
            "scope",
            "create",
            "--id",
            "private:test",
            "--type",
            "private",
        ])
        .assert()
        .success();

    let mut topk = bin();
    topk.args([
        "--db",
        &db_str,
        "--json",
        "query",
        "topk",
        "--uid",
        &uid,
        "--scope",
        "private:test",
        "--topic",
        "food_pref",
        "--limit",
        "3",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("[]"));
}

#[test]
fn user_delete_soft_marks_user_deleted() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("user-delete-soft.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO users (uid, display_name, status, created_at, updated_at) VALUES ('u_del', 'DeleteMe', 'active', '1', '1')",
        [],
    )
    .unwrap();

    let mut cmd = bin();
    cmd.args(["--db", &db_str, "user", "delete", "--uid", "u_del"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=soft"));

    let status: String = conn
        .query_row("SELECT status FROM users WHERE uid='u_del'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(status, "deleted");
}

#[test]
fn user_delete_hard_requires_force() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("user-delete-hard-force.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO users (uid, display_name, status, created_at, updated_at) VALUES ('u_hard', 'Hard', 'merged', '1', '1')",
        [],
    )
    .unwrap();

    let mut cmd = bin();
    cmd.args([
        "--db", &db_str, "user", "delete", "--uid", "u_hard", "--mode", "hard",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("--force"));
}

#[test]
fn user_delete_dry_run_prints_preflight_counts() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("user-delete-dry-run.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO users (uid, display_name, status, created_at, updated_at) VALUES ('u_dry', 'Dry', 'active', '1', '1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO scopes (scope_id, scope_type, created_at) VALUES ('shared:dry', 'shared', '1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO scope_members (scope_id, uid, role, added_at) VALUES ('shared:dry', 'u_dry', 'member', '1')",
        [],
    )
    .unwrap();

    let mut cmd = bin();
    cmd.args([
        "--db",
        &db_str,
        "user",
        "delete",
        "--uid",
        "u_dry",
        "--dry-run",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("delete preflight"));
}

#[test]
fn user_merge_moves_relations_and_marks_source_merged() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("user-merge.db");
    let db_str = db_path.to_string_lossy().to_string();
    migrate_db(&db_str);

    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO users (uid, display_name, status, created_at, updated_at) VALUES ('u_from', 'From', 'active', '1', '1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO users (uid, display_name, status, created_at, updated_at) VALUES ('u_to', 'To', 'active', '1', '1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO scopes (scope_id, scope_type, created_at) VALUES ('shared:couple', 'shared', '1')",
        [],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO user_identities (identity_id, uid, channel, channel_user_id, is_verified, confidence, created_at, updated_at)
         VALUES ('ident_1', 'u_from', 'telegram', '999', 1, 1.0, '1', '1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO scope_members (scope_id, uid, role, added_at) VALUES ('shared:couple', 'u_from', 'member', '1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO scope_members (scope_id, uid, role, added_at) VALUES ('shared:couple', 'u_to', 'member', '1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO events (event_id, uid, scope_id, event_type, event_ts, payload_json, idempotency_key, created_at)
         VALUES ('evt_from_dup', 'u_from', 'shared:couple', 'meal.rated', '10', '{}', 'dup-key', '10')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO events (event_id, uid, scope_id, event_type, event_ts, payload_json, idempotency_key, created_at)
         VALUES ('evt_to_dup', 'u_to', 'shared:couple', 'meal.rated', '11', '{}', 'dup-key', '11')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO state (scope_id, uid, state_key, value_json, updated_at)
         VALUES ('shared:couple', 'u_from', 'travel_food_style', '{\"spicy\":0.5}', '100')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO metrics (scope_id, uid, metric_key, metric_value, metric_json, updated_at)
         VALUES ('shared:couple', 'u_from', 'counter:food_pref:korean', 2.0, NULL, '100')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO topk (scope_id, uid, topic, rank, item_key, weight, updated_at)
         VALUES ('shared:couple', 'u_from', 'food_pref', 1, 'korean', 1.0, '100')",
        [],
    )
    .unwrap();

    let mut merge = bin();
    merge
        .args([
            "--db", &db_str, "user", "merge", "--from", "u_from", "--to", "u_to",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "merged user from_uid=u_from to_uid=u_to",
        ));

    let moved_identity_uid: String = conn
        .query_row(
            "SELECT uid FROM user_identities WHERE channel='telegram' AND channel_user_id='999'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(moved_identity_uid, "u_to");

    let scope_member_count_to: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM scope_members WHERE scope_id='shared:couple' AND uid='u_to'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(scope_member_count_to, 1);

    let event_from_count: i64 = conn
        .query_row("SELECT COUNT(1) FROM events WHERE uid='u_from'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(event_from_count, 0);

    let dedup_event_count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM events WHERE uid='u_to' AND scope_id='shared:couple' AND idempotency_key='dup-key'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(dedup_event_count, 1);

    let state_uid: String = conn
        .query_row(
            "SELECT uid FROM state WHERE scope_id='shared:couple' AND state_key='travel_food_style'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(state_uid, "u_to");

    let metric_uid: String = conn
        .query_row(
            "SELECT uid FROM metrics WHERE scope_id='shared:couple' AND metric_key='counter:food_pref:korean'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(metric_uid, "u_to");

    let topk_uid: String = conn
        .query_row(
            "SELECT uid FROM topk WHERE scope_id='shared:couple' AND topic='food_pref' AND rank=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(topk_uid, "u_to");

    let from_status: String = conn
        .query_row("SELECT status FROM users WHERE uid='u_from'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(from_status, "merged");
}
