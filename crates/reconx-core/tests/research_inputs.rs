use reconx_core::{Core, SecurityMode, TargetExport, TargetInput, TargetKind};
use reconx_storage::{Database, rusqlite::Connection};
use std::fs;
fn input(project: &str, kind: TargetKind, value: &str) -> TargetInput {
    TargetInput {
        project_id: project.into(),
        name: "Fixture target".into(),
        kind,
        value: value.into(),
        description: "Owned test input".into(),
        tags: vec!["local".into()],
        allowed_scope: vec![],
        excluded_scope: vec![],
        authorized: true,
        mode: SecurityMode::PassiveAuto,
    }
}
#[test]
fn target_crud_scope_import_and_restart_share_one_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("research.db");
    let core = Core::open(Some(path.clone())).unwrap();
    let project = core.create_project("inputs", "notes").unwrap();
    let mut i = input(&project.id, TargetKind::Domain, "EXAMPLE.TEST.");
    i.allowed_scope = vec!["example.test".into(), "*.example.test".into()];
    i.excluded_scope = vec!["admin.example.test".into()];
    let target = core.create_target(i).unwrap();
    assert_eq!(target.input.value, "example.test");
    assert_eq!(
        core.validate_scope(&target.id, Some("admin.example.test"))
            .unwrap()
            .status,
        "excluded"
    );
    assert_eq!(
        core.validate_scope(&target.id, Some("https://api.example.test/v1"))
            .unwrap()
            .status,
        "allowed"
    );
    assert_eq!(
        core.validate_scope(&target.id, Some("evilexample.test"))
            .unwrap()
            .status,
        "out_of_scope"
    );
    let reopened = Core::open(Some(path)).unwrap();
    assert_eq!(
        reopened.target(&target.id).unwrap().input.tags,
        vec!["local"]
    );
    let mut changed = target.input.clone();
    changed.authorized = false;
    changed.name = "Updated target".into();
    reopened.update_target(&target.id, changed).unwrap();
    assert_eq!(
        reopened.validate_scope(&target.id, None).unwrap().status,
        "awaiting_authorization"
    );
    let before = reopened.list_targets(None).unwrap().len();
    let batch = TargetExport {
        format_version: 1,
        targets: vec![
            input(&project.id, TargetKind::Domain, "second.test"),
            target.input.clone(),
        ],
    };
    assert!(reopened.import_targets(batch).is_err());
    assert_eq!(reopened.list_targets(None).unwrap().len(), before);
    reopened.remove_target(&target.id).unwrap();
    assert!(reopened.target(&target.id).is_err());
    assert_eq!(reopened.project(&project.id).unwrap().description, "notes");
}
#[test]
fn every_registration_type_is_supported_without_analyzer_dependencies() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("db"))).unwrap();
    let p = core.create_project("types", "").unwrap();
    let local = dir.path().to_string_lossy().into_owned();
    let sample = dir.path().join("sample.bin");
    fs::write(&sample, b"owned sample").unwrap();
    let binary = sample.to_string_lossy().into_owned();
    for (kind, value) in [
        (TargetKind::Domain, "example.test"),
        (TargetKind::Subdomain, "api.example.test"),
        (TargetKind::Hostname, "localhost"),
        (TargetKind::Url, "http://localhost:8080/"),
        (TargetKind::Ipv4, "127.0.0.1"),
        (TargetKind::Ipv6, "::1"),
        (TargetKind::Cidr, "192.0.2.10/24"),
        (TargetKind::RestApi, "https://example.test/api"),
        (TargetKind::Graphql, "https://example.test/graphql"),
        (TargetKind::GitRepository, "https://example.test/repo.git"),
        (TargetKind::SourceDirectory, &local),
        (TargetKind::Binary, &binary),
        (TargetKind::SharedLibrary, &binary),
        (TargetKind::Firmware, &binary),
        (TargetKind::Apk, &binary),
        (
            TargetKind::ContainerImage,
            "registry.example.test/team/image:1",
        ),
        (TargetKind::IacDirectory, &local),
    ] {
        core.create_target(input(&p.id, kind, value)).unwrap();
    }
    assert_eq!(core.list_targets(Some(&p.id)).unwrap().len(), 17);
    for (kind, value) in [
        (TargetKind::Domain, "bad..test"),
        (TargetKind::Ipv4, "999.1.2.3"),
        (TargetKind::Ipv6, "bad"),
        (TargetKind::Cidr, "127.0.0.1/200"),
        (TargetKind::Url, "https://example.test/?api_key=private"),
        (TargetKind::Binary, "../sample.exe"),
        (TargetKind::ContainerImage, "--help;command"),
    ] {
        assert!(core.create_target(input(&p.id, kind, value)).is_err());
    }
    let cidr = core
        .list_targets(None)
        .unwrap()
        .into_iter()
        .find(|t| t.input.kind == TargetKind::Cidr)
        .unwrap();
    assert_eq!(cidr.input.value, "192.0.2.0/24");
    assert!(
        core.validate_scope(&cidr.id, Some("192.0.2.25"))
            .unwrap()
            .in_scope
    );
    assert!(
        !core
            .validate_scope(&cidr.id, Some("192.0.3.25"))
            .unwrap()
            .in_scope
    );
}
#[test]
fn url_and_local_scope_respect_component_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("db"))).unwrap();
    let p = core.create_project("scope", "").unwrap();
    let target = core
        .create_target(input(
            &p.id,
            TargetKind::RestApi,
            "http://localhost:8080/api",
        ))
        .unwrap();
    for value in ["http://localhost:8080/api/v1", "http://localhost:8080/api"] {
        assert!(
            core.validate_scope(&target.id, Some(value))
                .unwrap()
                .in_scope
        );
    }
    for value in [
        "http://localhost:8080/apievil",
        "http://localhost:9090/api",
        "https://localhost:8080/api",
    ] {
        assert!(
            !core
                .validate_scope(&target.id, Some(value))
                .unwrap()
                .in_scope
        );
    }
    let local = core
        .create_target(input(
            &p.id,
            TargetKind::SourceDirectory,
            &dir.path().to_string_lossy(),
        ))
        .unwrap();
    assert!(
        !core
            .validate_scope(&local.id, Some(&format!("{}-other", dir.path().display())))
            .unwrap()
            .in_scope
    );
}
#[test]
fn real_native_binary_import_analysis_jobs_restart_and_integrity() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db");
    let core = Core::open(Some(db.clone())).unwrap();
    let p = core.create_project("binary", "").unwrap();
    let file = core
        .import_file(&p.id, &std::env::current_exe().unwrap(), true)
        .unwrap();
    assert!(matches!(file.format.as_str(), "Pe" | "Elf" | "MachO"));
    let record = core.analyze_file(&file.id).unwrap();
    assert!(!record.analysis.sections.is_empty());
    assert!(!record.analysis.strings.is_empty());
    assert_eq!(record.analysis.sha256, file.sha256);
    let reopened = Core::open(Some(db)).unwrap();
    assert_eq!(reopened.file_analysis(&file.id).unwrap().id, record.id);
    assert_eq!(reopened.job(&record.job_id).unwrap().status, "completed");
    assert!(
        reopened
            .target(file.target_id.as_deref().unwrap())
            .unwrap()
            .last_analyzed_at
            .is_some()
    );
    let artifact = dir
        .path()
        .join("imports")
        .join(format!("{}.json", record.id));
    fs::write(&artifact, b"{}").unwrap();
    assert_eq!(
        reopened.file_analysis(&file.id).unwrap_err().code,
        "INTEGRITY_ERROR"
    );
    let blob = dir
        .path()
        .join("imports")
        .join(format!("{}.bin", file.sha256));
    fs::write(&blob, b"changed").unwrap();
    assert_eq!(
        reopened.analyze_file(&file.id).unwrap_err().code,
        "INTEGRITY_ERROR"
    );
    assert!(
        reopened
            .list_jobs(None)
            .unwrap()
            .iter()
            .any(|j| j.status == "failed")
    );
    reopened.remove_file(&file.id).unwrap();
    assert!(reopened.list_files(None).unwrap().is_empty());
    assert!(
        reopened
            .list_jobs(None)
            .unwrap()
            .iter()
            .all(|j| j.file_id.is_none())
    );
    assert!(std::env::current_exe().unwrap().exists());
}
#[test]
fn authorization_exclusions_identity_changes_and_cancellation_cannot_be_bypassed() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("db"))).unwrap();
    let p = core.create_project("policy", "").unwrap();
    let sample = dir.path().join("sample.bin");
    fs::write(&sample, b"owned benign string\0").unwrap();
    let file = core.import_file(&p.id, &sample, false).unwrap();
    assert_eq!(
        core.analyze_file(&file.id).unwrap_err().code,
        "BLOCKED_BY_POLICY"
    );
    let id = file.target_id.as_deref().unwrap();
    let mut t = core.target(id).unwrap().input;
    t.authorized = true;
    t.excluded_scope = vec![t.value.clone()];
    core.update_target(id, t.clone()).unwrap();
    assert_eq!(
        core.analyze_file(&file.id).unwrap_err().code,
        "BLOCKED_BY_POLICY"
    );
    t.excluded_scope.clear();
    core.update_target(id, t.clone()).unwrap();
    let job = core.begin_file_analysis(&file.id).unwrap();
    core.cancel_job(&job.id).unwrap();
    assert_eq!(
        core.run_file_analysis(&job.id).unwrap_err().code,
        "CANCELLED"
    );
    assert_eq!(core.job(&job.id).unwrap().status, "cancelled");
    assert!(core.file_analysis(&file.id).is_err());
    let next = core.begin_file_analysis(&file.id).unwrap();
    t.authorized = false;
    core.update_target(id, t.clone()).unwrap();
    assert_eq!(
        core.run_file_analysis(&next.id).unwrap_err().code,
        "BLOCKED_BY_POLICY"
    );
    t.kind = TargetKind::Domain;
    t.value = "changed.test".into();
    t.allowed_scope = vec![];
    core.update_target(id, t).unwrap();
    assert_eq!(
        core.analyze_file(&file.id).unwrap_err().code,
        "BLOCKED_BY_POLICY"
    );
}
#[test]
fn invalid_oversized_and_missing_files_do_not_create_import_records() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("db"))).unwrap();
    let p = core.create_project("input-errors", "").unwrap();
    let oversized = dir.path().join("large.bin");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    assert_eq!(
        core.import_file(&p.id, &oversized, true).unwrap_err().code,
        "INPUT_LIMIT"
    );
    assert_eq!(
        core.import_file(&p.id, &dir.path().join("missing"), true)
            .unwrap_err()
            .code,
        "FILE_IO"
    );
    assert_eq!(
        core.import_file(&p.id, dir.path(), true).unwrap_err().code,
        "INVALID_FILE"
    );
    assert!(core.list_files(None).unwrap().is_empty());
    assert!(core.list_targets(None).unwrap().is_empty());
    assert!(
        core.create_target(input(
            &p.id,
            TargetKind::Binary,
            &dir.path().to_string_lossy()
        ))
        .is_err()
    );
}
#[test]
fn folder_import_reports_failures_comparison_and_originals_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("db"))).unwrap();
    let p = core.create_project("folder", "").unwrap();
    let folder = dir.path().join("samples");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("a.bin"), b"benign fixture A\0").unwrap();
    fs::write(folder.join("b.bin"), b"benign fixture B\0").unwrap();
    fs::write(folder.join("bad.exe"), b"MZ").unwrap();
    let result = core.import_folder(&p.id, &folder, true).unwrap();
    assert_eq!(result.imported.len(), 2);
    assert_eq!(result.failures.len(), 1);
    assert_eq!(result.failures[0].error_code, "MALFORMED_BINARY");
    let diff = core
        .compare_files(&result.imported[0].id, &result.imported[1].id)
        .unwrap();
    assert!(!diff.identical);
    assert_eq!(diff.changed_bytes, 1);
    assert_eq!(fs::read(folder.join("bad.exe")).unwrap(), b"MZ");
}
#[test]
fn migration_preserves_version_one_projects_settings_and_providers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(include_str!(
        "../../reconx-storage/migrations/001_foundation.sql"
    ))
    .unwrap();
    conn.execute(
        "INSERT INTO projects VALUES('project','Legacy','notes','before','before',0)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO settings VALUES(1,?1)",
        [r#"{"security_mode":"passive_auto","default_provider_id":null,"theme":"light"}"#],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO providers VALUES('provider','Legacy provider','{}')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 1).unwrap();
    drop(conn);
    let db = Database::open(&path).unwrap();
    assert_eq!(
        db.connect()
            .unwrap()
            .query_row(
                "SELECT description FROM projects WHERE id='project'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "notes"
    );
    assert_eq!(
        db.connect()
            .unwrap()
            .query_row("SELECT json FROM providers WHERE id='provider'", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
        "{}"
    );
    assert_eq!(
        db.connect()
            .unwrap()
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        3
    );
    Database::open(&path).unwrap();
}
