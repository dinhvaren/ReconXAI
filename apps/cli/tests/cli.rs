use serde_json::Value;
use std::process::Command;
fn run(db: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_reconx"))
        .arg("--database")
        .arg(db)
        .arg("--json")
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn native_cli_persists_projects_and_emits_machine_errors() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("cli.db");
    let created = run(&db, &["project", "create", "CLI test"]);
    assert!(created.status.success());
    let value: Value = serde_json::from_slice(&created.stdout).unwrap();
    assert_eq!(value["name"], "CLI test");
    let listed = run(&db, &["project", "list"]);
    assert!(listed.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&listed.stdout).unwrap()[0]["id"],
        value["id"]
    );
    let duplicate = run(&db, &["project", "create", "CLI test"]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&duplicate.stderr).unwrap()["error"]["code"],
        "DUPLICATE_NAME"
    );
    let missing = run(&db, &["project", "open", "missing"]);
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stderr).unwrap()["error"]["code"],
        "NOT_FOUND"
    );
}
