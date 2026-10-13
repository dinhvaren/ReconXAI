use reconx_core::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use zeroize::Zeroizing;
#[derive(Default)]
struct MemoryStore(Mutex<HashMap<String, String>>);
impl SecretStore for MemoryStore {
    fn set(&self, id: &str, key: &str) -> Result<()> {
        self.0.lock().unwrap().insert(id.into(), key.into());
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Option<Zeroizing<String>>> {
        Ok(self.0.lock().unwrap().get(id).cloned().map(Zeroizing::new))
    }
    fn delete(&self, id: &str) -> Result<()> {
        self.0.lock().unwrap().remove(id);
        Ok(())
    }
}
fn setup() -> (tempfile::TempDir, Core) {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_secrets(
        Some(dir.path().join("workspace.db")),
        Arc::new(MemoryStore::default()),
    )
    .unwrap();
    (dir, core)
}
fn provider() -> ProviderConfig {
    ProviderConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Private model".into(),
        kind: ProviderKind::Compatible,
        base_url: "http://127.0.0.1:8080/v1".into(),
        model: "fixture".into(),
        enabled: true,
        timeout_secs: 30,
        max_output_tokens: 512,
        requires_key: true,
    }
}
#[test]
fn persistence_import_export_archive_and_audit() {
    let (dir, core) = setup();
    let p = core.create_project("Research", "Local notes").unwrap();
    assert_eq!(
        core.create_project("research", "").unwrap_err().code,
        "DUPLICATE_NAME"
    );
    let core = Core::with_secrets(
        Some(dir.path().join("workspace.db")),
        Arc::new(MemoryStore::default()),
    )
    .unwrap();
    assert_eq!(core.project(&p.id).unwrap().description, "Local notes");
    core.update_project(&p.id, "Renamed", "Updated notes")
        .unwrap();
    core.archive_project(&p.id, true).unwrap();
    assert!(core.list_projects(false).unwrap().is_empty());
    assert_eq!(core.list_projects(true).unwrap().len(), 1);
    core.archive_project(&p.id, false).unwrap();
    let export = core.export_project(&p.id).unwrap();
    let (_destination_dir, destination) = setup();
    let imported = destination.import_project(export).unwrap();
    assert_ne!(p.id, imported.id);
    assert_eq!(imported.name, "Renamed");
    assert_eq!(core.audit_events().unwrap().len(), 4);
    assert_eq!(
        core.import_project(ProjectExport {
            format_version: 99,
            name: "Bad".into(),
            description: "".into()
        })
        .unwrap_err()
        .code,
        "UNSUPPORTED_FORMAT"
    );
}
#[test]
fn settings_and_provider_credential_isolation() {
    let (_dir, core) = setup();
    let mut config = provider();
    core.save_provider(&config).unwrap();
    core.set_key(&config.id, "fixture-secret-never-persist")
        .unwrap();
    assert!(core.credential_status(&config.id).unwrap());
    let dump = serde_json::to_string(&core.snapshot().unwrap()).unwrap();
    assert!(!dump.contains("fixture-secret"));
    core.save_settings(&Settings {
        security_mode: SecurityMode::LabResearch,
        default_provider_id: Some(config.id.clone()),
        fallback_provider_id: None,
        theme: ThemePreference::Dark,
    })
    .unwrap();
    config.base_url = "https://another.example/v1".into();
    assert_eq!(
        core.save_provider(&config).unwrap_err().code,
        "GATEWAY_CREDENTIAL_BOUND"
    );
    core.remove_key(&config.id).unwrap();
    core.save_provider(&config).unwrap();
    core.delete_provider(&config.id).unwrap();
    assert!(core.providers().unwrap().is_empty());
    assert!(core.settings().unwrap().default_provider_id.is_none());
}

#[test]
fn provider_id_reuse_cannot_redirect_a_stored_credential() {
    let store = Arc::new(MemoryStore::default());
    let first_dir = tempfile::tempdir().unwrap();
    let second_dir = tempfile::tempdir().unwrap();
    let first = Core::with_secrets(Some(first_dir.path().join("db")), store.clone()).unwrap();
    let second = Core::with_secrets(Some(second_dir.path().join("db")), store).unwrap();
    let mut config = provider();
    first.save_provider(&config).unwrap();
    first.set_key(&config.id, "owned-secret").unwrap();
    config.base_url = "https://different.example/v1".into();
    second.save_provider(&config).unwrap();
    assert!(!second.credential_status(&config.id).unwrap());
    assert!(first.credential_status(&config.id).unwrap());
}
#[test]
fn concurrent_writers_keep_all_records() {
    let (_dir, core) = setup();
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let core = core.clone();
            std::thread::spawn(move || core.create_project(&format!("Project {i}"), "").unwrap())
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(core.list_projects(false).unwrap().len(), 8);
    assert_eq!(core.audit_events().unwrap().len(), 8);
}
#[test]
fn validation_does_not_modify_database() {
    let (_dir, core) = setup();
    assert!(core.create_project("   ", "").is_err());
    assert!(core.create_project("multiple\nlines", "").is_err());
    assert!(
        core.save_settings(&Settings {
            security_mode: SecurityMode::PassiveAuto,
            default_provider_id: Some("missing".into()),
            fallback_provider_id: None,
            theme: ThemePreference::Dark,
        })
        .is_err()
    );
    assert!(core.snapshot().unwrap().audit.is_empty());
}

#[test]
fn appearance_survives_restart_and_legacy_settings_default_to_dark() {
    let (_dir, core) = setup();
    let db = reconx_storage::Database::open(core.database_path()).unwrap();
    db.connect()
        .unwrap()
        .execute(
            "INSERT INTO settings(id,json) VALUES(1,?1)",
            [r#"{"security_mode":"passive_auto","default_provider_id":null}"#],
        )
        .unwrap();
    assert_eq!(core.settings().unwrap().theme, ThemePreference::Dark);
    for theme in [
        ThemePreference::Light,
        ThemePreference::System,
        ThemePreference::Dark,
    ] {
        core.save_settings(&Settings {
            theme,
            ..Default::default()
        })
        .unwrap();
        let reopened = Core::with_secrets(
            Some(core.database_path().to_path_buf()),
            Arc::new(MemoryStore::default()),
        )
        .unwrap();
        assert_eq!(reopened.settings().unwrap().theme, theme);
    }
}
#[test]
fn unavailable_credential_store_has_no_plaintext_fallback() {
    struct Unavailable;
    impl SecretStore for Unavailable {
        fn set(&self, _: &str, _: &str) -> Result<()> {
            Err(AppError::new("SECRET_STORE_UNAVAILABLE", "Unavailable"))
        }
        fn get(&self, _: &str) -> Result<Option<Zeroizing<String>>> {
            Err(AppError::new("SECRET_STORE_UNAVAILABLE", "Unavailable"))
        }
        fn delete(&self, _: &str) -> Result<()> {
            Err(AppError::new("SECRET_STORE_UNAVAILABLE", "Unavailable"))
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_secrets(Some(dir.path().join("db")), Arc::new(Unavailable)).unwrap();
    let config = provider();
    core.save_provider(&config).unwrap();
    assert_eq!(
        core.set_key(&config.id, "do-not-save").unwrap_err().code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert!(
        !serde_json::to_string(&core.snapshot().unwrap())
            .unwrap()
            .contains("do-not-save")
    );
}
