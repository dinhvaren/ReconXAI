use reconx_core::{
    Core, ExtensionApproval, ExtensionManifest, JobInput, JobKind, validate_extension_manifest,
};
use reconx_storage::rusqlite::{Connection, params};
use serde_json::json;
const ID: &str = "reconx.yara-signatures";
fn fixture() -> (tempfile::TempDir, Core) {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("reconx.db"))).unwrap();
    (dir, core)
}
fn registration(conn: &Connection, enabled: bool) {
    // Policy fixture only; no tool or successful capability execution is simulated.
    conn.execute("INSERT INTO extension_installations VALUES(?1,?2)", params![ID,json!({"version":"0.1.0","enabled":enabled,"status":"installed","revision":"policy-fixture","updated_at":"2026-01-01T00:00:00Z","last_health_check":null,"error_code":null}).to_string()]).unwrap();
    conn.execute(
        "INSERT INTO extension_requirements VALUES(?1,'yara-x')",
        [ID],
    )
    .unwrap();
}
#[test]
fn catalog_and_manifest_never_authorize_arbitrary_runtime_code() {
    let (_dir, core) = fixture();
    let entry = core.extension_entry(ID).unwrap();
    assert_eq!(entry.status, "not_installed");
    assert!(entry.installed.is_none());
    assert!(core.extension_catalog(Some("yara")).unwrap().len() == 1);
    assert!(
        core.extension_catalog(Some("nonexistent"))
            .unwrap()
            .is_empty()
    );
    validate_extension_manifest(&entry.manifest).unwrap();
    for field in ["entrypoint", "version", "install_source", "publisher"] {
        let mut value = serde_json::to_value(&entry.manifest).unwrap();
        value[field] = json!("attacker-controlled");
        let imported: ExtensionManifest = serde_json::from_value(value).unwrap();
        assert_eq!(
            validate_extension_manifest(&imported).unwrap_err().code,
            "EXTENSION_POLICY"
        );
    }
    let mut value = serde_json::to_value(&entry.manifest).unwrap();
    value["execute_shell"] = json!(true);
    assert!(serde_json::from_value::<ExtensionManifest>(value).is_err());
    assert_eq!(
        core.extension_entry("../yara-x").unwrap_err().code,
        "EXTENSION_NOT_FOUND"
    );
    assert_eq!(
        core.extension_invoke(ID, "missing").unwrap_err().code,
        "EXTENSION_NOT_READY"
    );
}
#[tokio::test]
async fn installation_requires_exact_approval_and_never_downloads_on_rejection() {
    let (dir, core) = fixture();
    assert_eq!(
        core.extension_install(ExtensionApproval {
            id: ID.into(),
            fingerprint: String::new(),
            approved: false
        })
        .await
        .unwrap_err()
        .code,
        "APPROVAL_REQUIRED"
    );
    assert_eq!(
        core.extension_install(ExtensionApproval {
            id: ID.into(),
            fingerprint: "changed".into(),
            approved: true
        })
        .await
        .unwrap_err()
        .code,
        "PLAN_CHANGED"
    );
    assert!(!dir.path().join("tools").exists());
    let plan = core.extension_install_plan(ID).unwrap();
    registration(
        &Connection::open(dir.path().join("reconx.db")).unwrap(),
        false,
    );
    assert_eq!(
        core.extension_install(ExtensionApproval {
            id: ID.into(),
            fingerprint: plan.fingerprint,
            approved: true
        })
        .await
        .unwrap_err()
        .code,
        "PLAN_CHANGED"
    );
    assert_eq!(
        core.extension_set_enabled(ID, true).unwrap_err().code,
        "EXTENSION_HEALTH_REQUIRED"
    );
}
#[test]
fn disable_cancels_bound_jobs_and_removal_preserves_history_and_shared_tools() {
    let (dir, core) = fixture();
    let project = core.create_project("Extension policy", "").unwrap();
    let path = dir.path().join("owned.bin");
    std::fs::write(&path, b"RECONX_BENIGN_FIXTURE").unwrap();
    let file = core.import_file(&project.id, &path, true).unwrap();
    let queued = core
        .enqueue_job(JobInput {
            kind: JobKind::ArtifactIndex,
            file_id: Some(file.id),
            target_id: None,
            timeout_ms: 60_000,
        })
        .unwrap();
    let conn = Connection::open(dir.path().join("reconx.db")).unwrap();
    registration(&conn, true);
    conn.execute(
        "INSERT INTO extension_invocations VALUES(?1,?2,'policy-fixture','2026-01-01')",
        params![queued.id, ID],
    )
    .unwrap();
    assert_eq!(
        core.tool_uninstall("yara-x").unwrap_err().code,
        "TOOL_REQUIRED_BY_EXTENSION"
    );
    core.extension_set_enabled(ID, false).unwrap();
    assert_eq!(core.job(&queued.id).unwrap().status, "cancelled");
    assert!(core.job(&queued.id).unwrap().cancel_requested);
    core.extension_remove(ID).unwrap();
    assert_eq!(core.extension_entry(ID).unwrap().status, "not_installed");
    assert_eq!(core.extension_history(ID).unwrap()[0].id, queued.id);
    assert!(
        core.imported_file(core.job(&queued.id).unwrap().file_id.as_deref().unwrap())
            .is_ok()
    );
    let reopened = Core::open(Some(dir.path().join("reconx.db"))).unwrap();
    assert_eq!(reopened.extension_history(ID).unwrap().len(), 1);
    assert_eq!(
        conn.query_row("PRAGMA foreign_key_check", [], |_| Ok(()))
            .optional()
            .unwrap(),
        None
    );
}
use reconx_storage::rusqlite::OptionalExtension;
