use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn doctor_command_returns_ready_message() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("agent-memory-cli is ready"));
}

#[test]
fn top_level_help_includes_spec_command_groups() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
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
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.args(["user", "--help"]).assert().success().stdout(
        predicate::str::contains("create")
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("show"))
            .and(predicate::str::contains("update")),
    );
}

#[test]
fn identity_help_supports_link_resolve_unlink() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.args(["identity", "--help"]).assert().success().stdout(
        predicate::str::contains("link")
            .and(predicate::str::contains("resolve"))
            .and(predicate::str::contains("unlink")),
    );
}

#[test]
fn scope_help_supports_create_add_member_list_members() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.args(["scope", "--help"]).assert().success().stdout(
        predicate::str::contains("create")
            .and(predicate::str::contains("add-member"))
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("members")),
    );
}

#[test]
fn schema_help_supports_register_list_validate() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.args(["schema", "--help"]).assert().success().stdout(
        predicate::str::contains("register")
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("validate")),
    );
}

#[test]
fn ingest_help_supports_event_and_batch() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.args(["ingest", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("event").and(predicate::str::contains("batch")));
}

#[test]
fn query_help_supports_latest_metric_topk() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.args(["query", "--help"]).assert().success().stdout(
        predicate::str::contains("latest")
            .and(predicate::str::contains("metric"))
            .and(predicate::str::contains("topk")),
    );
}

#[test]
fn state_help_supports_get_set_delete() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.args(["state", "--help"]).assert().success().stdout(
        predicate::str::contains("get")
            .and(predicate::str::contains("set"))
            .and(predicate::str::contains("delete")),
    );
}

#[test]
fn admin_help_supports_migrate_reindex_compact_archive() {
    let mut cmd = Command::cargo_bin("agent-memory-cli").unwrap();
    cmd.args(["admin", "--help"]).assert().success().stdout(
        predicate::str::contains("migrate")
            .and(predicate::str::contains("reindex"))
            .and(predicate::str::contains("compact"))
            .and(predicate::str::contains("archive")),
    );
}
