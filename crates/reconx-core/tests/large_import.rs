use reconx_core::*;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};
fn setup() -> (tempfile::TempDir, Core, String) {
    let d = tempfile::tempdir().unwrap();
    let c = Core::open(Some(d.path().join("test.db"))).unwrap();
    let p = c.create_project("Streaming fixture", "").unwrap().id;
    (d, c, p)
}
fn sample(path: &Path, size: u64) {
    let mut f = std::fs::File::create(path).unwrap();
    f.write_all(b"BENIGN_LARGE_IMPORT_FIXTURE\0").unwrap();
    f.set_len(size).unwrap();
}
fn hash(path: &Path) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    let mut h = Sha256::new();
    let mut b = vec![0; 1024 * 1024];
    loop {
        let n = f.read(&mut b).unwrap();
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
fn no_partial(c: &Core, d: &Path) {
    assert!(c.list_files(None).unwrap().is_empty());
    assert!(c.list_targets(None).unwrap().is_empty());
    if d.join("imports").exists() {
        assert_eq!(fs::read_dir(d.join("imports")).unwrap().count(), 0);
    }
}
#[test]
fn large_import_is_streamed_hashed_persistent_and_analyzed_with_explicit_partial_metadata() {
    let (d, c, p) = setup();
    let path = d.path().join("benign.raw");
    sample(&path, 80 * 1024 * 1024);
    let expected = hash(&path);
    let mut ticks = 0;
    let mut maximum = 0;
    let f = c
        .import_file_with_progress(&p, &path, true, FileImportLimits::default(), |u| {
            ticks += 1;
            maximum = maximum.max(u.completed);
            assert!(u.completed <= u.total);
            Ok(())
        })
        .unwrap();
    assert_eq!(f.sha256, expected);
    assert_eq!(f.size, 80 * 1024 * 1024);
    assert!(ticks > 100);
    assert_eq!(maximum, f.size);
    let analysis = c.analyze_file(&f.id).unwrap();
    let result = c.job(&analysis.job_id).unwrap().result.unwrap();
    assert!(result["sections"].is_null());
    assert_eq!(result["analysis_limited"], true);
    let index = c
        .enqueue_job(JobInput {
            kind: JobKind::ArtifactIndex,
            file_id: Some(f.id.clone()),
            target_id: None,
            timeout_ms: 60_000,
        })
        .unwrap();
    assert!(c.execute_job(&index.id).unwrap().result.unwrap()["sections"].is_null());
    assert_eq!(analysis.analysis.sha256, expected);
    assert_eq!(analysis.analysis.size, f.size);
    assert!(analysis.analysis.sections.is_empty());
    assert!(
        analysis
            .analysis
            .warnings
            .iter()
            .any(|s| s.contains("first 1 MiB"))
    );
    assert!(
        analysis
            .analysis
            .strings
            .iter()
            .any(|s| s.text.contains("BENIGN_LARGE_IMPORT_FIXTURE"))
    );
    let reopened = Core::open(Some(d.path().join("test.db"))).unwrap();
    assert_eq!(reopened.imported_file(&f.id).unwrap().size, f.size);
    assert_eq!(reopened.file_analysis(&f.id).unwrap().id, analysis.id);
    reopened
        .evidence_verify(&reopened.evidence_list(Some(&p)).unwrap()[0].id)
        .unwrap();
    assert_eq!(hash(&path), expected);
}
#[test]
fn cancellation_during_copy_and_before_commit_removes_staging_without_creating_targets() {
    for commit in [false, true] {
        let (d, c, p) = setup();
        let path = d.path().join("cancel.raw");
        sample(&path, 80 * 1024 * 1024);
        let e = c
            .import_file_with_progress(&p, &path, true, FileImportLimits::default(), |u| {
                if (commit && u.stage == "Committing complete artifact")
                    || (!commit && u.completed >= 1024 * 1024)
                {
                    return Err(AppError::new("CANCELLED", "Controlled cancellation"));
                }
                Ok(())
            })
            .unwrap_err();
        assert_eq!(e.code, "CANCELLED");
        no_partial(&c, d.path());
        assert_eq!(fs::metadata(path).unwrap().len(), 80 * 1024 * 1024);
    }
}
#[test]
fn changed_source_and_archived_project_abort_the_import_transaction() {
    for archive in [false, true] {
        let (d, c, p) = setup();
        let path = d.path().join("changed.raw");
        sample(&path, 4 * 1024 * 1024);
        let mut changed = false;
        let e = c
            .import_file_with_progress(&p, &path, true, FileImportLimits::default(), |u| {
                if !changed && u.completed > 0 {
                    changed = true;
                    if archive {
                        c.archive_project(&p, true)?;
                    } else {
                        OpenOptions::new()
                            .append(true)
                            .open(&path)
                            .unwrap()
                            .write_all(b"changed")
                            .unwrap();
                    }
                }
                Ok(())
            })
            .unwrap_err();
        assert_eq!(
            e.code,
            if archive {
                "PROJECT_ARCHIVED"
            } else {
                "FILE_CHANGED"
            }
        );
        no_partial(&c, d.path());
    }
}
#[test]
fn full_artifact_deduplication_verifies_existing_bytes_and_keeps_existing_authorization() {
    let (d, c, p) = setup();
    let path = d.path().join("benign.raw");
    sample(&path, 80 * 1024 * 1024);
    let first = c.import_file(&p, &path, false).unwrap();
    let second = c.import_file(&p, &path, true).unwrap();
    assert_eq!(first.sha256, second.sha256);
    assert!(
        !c.target(first.target_id.as_ref().unwrap())
            .unwrap()
            .input
            .authorized
    );
    assert_eq!(fs::read_dir(d.path().join("imports")).unwrap().count(), 1);
    let blob = d
        .path()
        .join("imports")
        .join(format!("{}.bin", first.sha256));
    let mut f = OpenOptions::new().write(true).open(blob).unwrap();
    f.write_all(b"tampered").unwrap();
    drop(f);
    assert_eq!(
        c.import_file(&p, &path, true).unwrap_err().code,
        "INTEGRITY_ERROR"
    );
    assert_eq!(c.list_files(None).unwrap().len(), 2);
}
#[test]
fn size_storage_and_limits_do_not_truncate_at_four_gibibytes() {
    let (d, c, p) = setup();
    let path = d.path().join("small.raw");
    sample(&path, 4096);
    let f = c.import_file(&p, &path, true).unwrap();
    let db = reconx_storage::rusqlite::Connection::open(c.database_path()).unwrap();
    db.execute(
        "UPDATE imported_files SET size=?1 WHERE id=?2",
        reconx_storage::rusqlite::params![5_i64 * 1024 * 1024 * 1024 + 7, f.id],
    )
    .unwrap();
    assert_eq!(
        c.imported_file(&f.id).unwrap().size,
        5 * 1024 * 1024 * 1024 + 7
    );
    assert_eq!(
        c.import_file_with_progress(
            &p,
            &path,
            true,
            FileImportLimits { max_bytes: 4095 },
            |_| Ok(())
        )
        .unwrap_err()
        .code,
        "INPUT_LIMIT"
    );
    assert_eq!(c.list_files(None).unwrap().len(), 1);
}
