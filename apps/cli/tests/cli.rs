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
#[test]
fn native_cli_durable_queue_runs_typed_work_and_cancels_queued_jobs() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("jobs.db");
    assert!(run(&db, &["project", "create", "demo"]).status.success());
    let sample = dir.path().join("owned.raw");
    std::fs::write(&sample, b"real owned fixture strings").unwrap();
    let imported = run(
        &db,
        &[
            "reverse",
            "import",
            sample.to_str().unwrap(),
            "--project",
            "demo",
            "--authorized",
        ],
    );
    assert!(imported.status.success());
    let file: Value = serde_json::from_slice(&imported.stdout).unwrap();
    let id = file["id"].as_str().unwrap();
    let note_path = dir.path().join("note.txt");
    std::fs::write(&note_path, "owned local notes").unwrap();
    let note = run(
        &db,
        &[
            "reverse",
            "note",
            id,
            "--input",
            note_path.to_str().unwrap(),
        ],
    );
    assert!(note.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&note.stdout).unwrap()["note"],
        "owned local notes"
    );
    let queued = run(
        &db,
        &[
            "jobs",
            "enqueue",
            "--kind",
            "strings_extraction",
            "--file",
            id,
        ],
    );
    assert!(queued.status.success());
    let job: Value = serde_json::from_slice(&queued.stdout).unwrap();
    assert_eq!(job["status"], "queued");
    let drain = run(&db, &["jobs", "run", "--concurrency", "1"]);
    assert!(
        drain.status.success(),
        "{}",
        String::from_utf8_lossy(&drain.stderr)
    );
    let result = run(&db, &["jobs", "result", job["id"].as_str().unwrap()]);
    assert!(result.status.success());
    let result: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(result["analysis"]["sha256"], file["sha256"]);
    let queued = run(
        &db,
        &["jobs", "enqueue", "--kind", "artifact_index", "--file", id],
    );
    let job: Value = serde_json::from_slice(&queued.stdout).unwrap();
    let cancelled = run(&db, &["jobs", "cancel", job["id"].as_str().unwrap()]);
    assert!(cancelled.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&cancelled.stdout).unwrap()["status"],
        "cancelled"
    );
    assert!(
        !run(&db, &["jobs", "enqueue", "--kind", "shell", "--file", id])
            .status
            .success()
    );
}
#[test]
fn native_cli_targets_and_reverse_use_shared_core_and_restart_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("inputs.db");
    assert!(run(&db, &["project", "create", "demo"]).status.success());
    let add = run(
        &db,
        &[
            "target",
            "add",
            "--project",
            "demo",
            "--type",
            "domain",
            "--value",
            "example.test",
            "--authorized",
        ],
    );
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let target: Value = serde_json::from_slice(&add.stdout).unwrap();
    let id = target["id"].as_str().unwrap();
    let listed: Value =
        serde_json::from_slice(&run(&db, &["target", "list", "--project", "demo"]).stdout).unwrap();
    assert_eq!(listed[0]["id"], id);
    let scope: Value =
        serde_json::from_slice(&run(&db, &["scope", "validate", id]).stdout).unwrap();
    assert_eq!(scope["status"], "allowed");
    let invalid = run(
        &db,
        &[
            "target",
            "add",
            "--project",
            "demo",
            "--type",
            "ipv4",
            "--value",
            "bad",
        ],
    );
    assert!(!invalid.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&invalid.stderr).unwrap()["error"]["code"],
        "INVALID_TARGET"
    );
    let sample = std::env::current_exe().unwrap();
    let imported = run(
        &db,
        &[
            "reverse",
            "import",
            sample.to_str().unwrap(),
            "--project",
            "demo",
            "--authorized",
        ],
    );
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    let f: Value = serde_json::from_slice(&imported.stdout).unwrap();
    let fid = f["id"].as_str().unwrap();
    assert!(run(&db, &["reverse", "analyze", fid]).status.success());
    let metadata: Value =
        serde_json::from_slice(&run(&db, &["reverse", "metadata", fid]).stdout).unwrap();
    assert_eq!(metadata["analysis"]["sha256"], f["sha256"]);
    assert!(
        !metadata["analysis"]["sections"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let jobs: Value = serde_json::from_slice(&run(&db, &["jobs", "list"]).stdout).unwrap();
    assert_eq!(jobs[0]["status"], "completed");
    assert!(run(&db, &["target", "remove", id]).status.success());
}
