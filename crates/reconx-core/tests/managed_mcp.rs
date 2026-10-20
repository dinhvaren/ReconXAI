use reconx_core::{Core, ExtensionApproval, MANAGED_MCP_EVERYTHING as ID, ManagedMcpApproval};
use reconx_storage::rusqlite::Connection;

#[tokio::test]
async fn managed_packages_cannot_use_native_signature_or_unapproved_apis() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("test.db"))).unwrap();
    assert_eq!(core.extension_entry(ID).unwrap().status, "not_installed");
    let plan = core.managed_install_plan(ID, "2.0.0", true).unwrap();
    assert_eq!(plan.package_plan.as_ref().unwrap().dependencies.len(), 94);
    assert!(plan.package_plan.as_ref().unwrap().runtime.managed);
    assert_eq!(
        core.managed_enqueue_install(ManagedMcpApproval {
            id: ID.into(),
            version: "2.0.0".into(),
            managed_runtime: true,
            fingerprint: plan.fingerprint.clone(),
            approved: false
        })
        .unwrap_err()
        .code,
        "APPROVAL_REQUIRED"
    );
    assert_eq!(
        core.managed_enqueue_install(ManagedMcpApproval {
            id: ID.into(),
            version: "2.0.0".into(),
            managed_runtime: true,
            fingerprint: "stale".into(),
            approved: true
        })
        .unwrap_err()
        .code,
        "PLAN_CHANGED"
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
        "MANAGED_EXTENSION_API"
    );
    assert_eq!(
        core.extension_set_enabled(ID, true).unwrap_err().code,
        "MANAGED_EXTENSION_API"
    );
    assert_eq!(
        core.extension_remove(ID).unwrap_err().code,
        "MANAGED_EXTENSION_API"
    );
    assert_eq!(
        core.extension_invoke(ID, "file").unwrap_err().code,
        "MANAGED_EXTENSION_API"
    );
    assert_eq!(
        core.managed_allow_tools(vec!["echo".into()], false)
            .await
            .unwrap_err()
            .code,
        "APPROVAL_REQUIRED"
    );
    assert!(!dir.path().join("extensions").exists());
}

#[test]
fn cancelled_install_recovers_registration_on_restart_and_retains_job_history() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("test.db");
    let core = Core::open(Some(db.clone())).unwrap();
    let plan = core.managed_install_plan(ID, "1.0.0", true).unwrap();
    let job = core
        .managed_enqueue_install(ManagedMcpApproval {
            id: ID.into(),
            version: "1.0.0".into(),
            managed_runtime: true,
            fingerprint: plan.fingerprint,
            approved: true,
        })
        .unwrap();
    assert_eq!(core.extension_entry(ID).unwrap().status, "installing");
    core.cancel_job(&job.id).unwrap();
    drop(core);
    let core = Core::open(Some(db.clone())).unwrap();
    let entry = core.extension_entry(ID).unwrap();
    assert_eq!(entry.status, "failed");
    assert!(!entry.installed.unwrap().enabled);
    assert_eq!(
        core.managed_install_history(ID).unwrap()[0].status,
        "cancelled"
    );
    assert!(core.mcp_servers().unwrap().is_empty());
    let conn = Connection::open(db).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(!dir.path().join("extensions").exists());
}
