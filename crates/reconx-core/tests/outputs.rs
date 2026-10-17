use reconx_core::*;
use std::fs;
fn setup() -> (tempfile::TempDir, Core, String) {
    let d = tempfile::tempdir().unwrap();
    let c = Core::open(Some(d.path().join("outputs.db"))).unwrap();
    let p = c.create_project("Local output fixtures", "").unwrap();
    (d, c, p.id)
}
fn input(project: &str, kind: EvidenceKind) -> EvidenceInput {
    EvidenceInput {
        project_id: project.into(),
        target_id: None,
        kind,
        title: "Actual fixture observation".into(),
        source_tool: "owned fixture".into(),
        sensitivity: Sensitivity::Confidential,
    }
}
fn finding(project: &str, evidence: &str) -> FindingInput {
    FindingInput {
        project_id: project.into(),
        title: "Local test observation".into(),
        evidence_ids: vec![evidence.into()],
        ..Default::default()
    }
}
#[test]
fn evidence_integrity_restart_and_redacted_export_preserve_original() {
    let (d, c, p) = setup();
    let e = c
        .evidence_add(
            input(&p, EvidenceKind::HttpResponse),
            "HTTP/1.1 200 OK\nSet-Cookie: session=fixture-private-cookie\n\nHello observed body",
        )
        .unwrap();
    assert_eq!(c.evidence_show(&e.id).unwrap().evidence.sha256, e.sha256);
    c.evidence_verify(&e.id).unwrap();
    let out = d.path().join("redacted.txt");
    c.evidence_export(&e.id, &out, false).unwrap();
    let exported = fs::read_to_string(&out).unwrap();
    assert!(!exported.contains("fixture-private-cookie"));
    assert!(exported.contains("[REDACTED]"));
    assert_eq!(
        c.evidence_export(&e.id, &out, false).unwrap_err().code,
        "OUTPUT_EXISTS"
    );
    let reopened = Core::open(Some(c.database_path().to_owned())).unwrap();
    assert!(
        reopened
            .evidence_show(&e.id)
            .unwrap()
            .text
            .unwrap()
            .contains("fixture-private-cookie")
    );
    fs::write(d.path().join(&e.storage_location), b"tampered").unwrap();
    assert_eq!(
        reopened.evidence_verify(&e.id).unwrap_err().code,
        "INTEGRITY_ERROR"
    );
}
#[test]
fn completed_real_jobs_automatically_create_one_observed_evidence_record() {
    let (d, c, p) = setup();
    let path = d.path().join("owned.bin");
    fs::write(&path, b"owned benign static research text").unwrap();
    let file = c.import_file(&p, &path, true).unwrap();
    let a = c.analyze_file(&file.id).unwrap();
    let e = c.evidence_list(Some(&p)).unwrap();
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].job_id.as_deref(), Some(a.job_id.as_str()));
    assert_eq!(e[0].verification_state, "observed");
    assert!(
        c.evidence_show(&e[0].id)
            .unwrap()
            .text
            .unwrap()
            .contains(&a.analysis.sha256)
    );
    assert_eq!(c.evidence_from_job(&a.job_id).unwrap().id, e[0].id);
    assert_eq!(c.evidence_list(None).unwrap().len(), 1);
    let job = c
        .enqueue_job(JobInput {
            file_id: Some(file.id),
            target_id: None,
            kind: JobKind::BinaryMetadata,
            timeout_ms: 1,
        })
        .unwrap();
    let done = c.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "failed");
    assert_eq!(c.evidence_list(None).unwrap().len(), 1);
}
#[test]
fn finding_verification_requires_manual_source_review_and_cross_project_links_fail() {
    let (_d, c, p) = setup();
    let e = c
        .evidence_add(
            input(&p, EvidenceKind::UserNote),
            "Observed benign local fixture condition",
        )
        .unwrap();
    let mut f = finding(&p, &e.id);
    f.status = FindingStatus::Verified;
    f.verification_note = "Reviewed actual source evidence".into();
    assert_eq!(
        c.finding_save(None, f.clone(), None, true)
            .unwrap_err()
            .code,
        "VERIFICATION_REQUIRED"
    );
    c.evidence_review(&e.id, "Compared with the actual local fixture")
        .unwrap();
    assert_eq!(
        c.finding_save(None, f.clone(), None, false)
            .unwrap_err()
            .code,
        "VERIFICATION_REQUIRED"
    );
    let saved = c.finding_save(None, f, None, true).unwrap();
    assert_eq!(saved.input.status, FindingStatus::Verified);
    let ai = c
        .evidence_add(input(&p, EvidenceKind::AiSummary), "Unverified model text")
        .unwrap();
    assert_eq!(
        c.evidence_review(&ai.id, "Claimed model confidence is insufficient")
            .unwrap_err()
            .code,
        "UNVERIFIED_AI"
    );
    let other = c.create_project("Other project", "").unwrap();
    assert_eq!(
        c.finding_save(None, finding(&other.id, &e.id), None, false)
            .unwrap_err()
            .code,
        "PROJECT_MISMATCH"
    );
}
#[test]
fn finding_lifecycle_stale_writers_compare_and_merge_keep_source_content() {
    let (_d, c, p) = setup();
    let e = c
        .evidence_add(input(&p, EvidenceKind::UserNote), "Local observation")
        .unwrap();
    let mut f = c
        .finding_save(None, finding(&p, &e.id), None, false)
        .unwrap();
    f.input.status = FindingStatus::FalsePositive;
    let updated = c
        .finding_save(Some(&f.id), f.input.clone(), Some(f.revision), false)
        .unwrap();
    assert_eq!(updated.revision, 2);
    assert_eq!(
        c.finding_save(Some(&f.id), f.input, Some(1), false)
            .unwrap_err()
            .code,
        "STALE_RECORD"
    );
    let b = c
        .finding_save(None, finding(&p, &e.id), None, false)
        .unwrap();
    c.finding_compare(&updated.id, &b.id).unwrap();
    c.finding_merge(&updated.id, &b.id).unwrap();
    let merged = c.finding(&b.id).unwrap();
    assert_eq!(merged.input.status, FindingStatus::Duplicate);
    assert_eq!(merged.duplicate_of.as_deref(), Some(updated.id.as_str()));
    assert_eq!(merged.input.description, b.input.description);
    assert_eq!(c.finding_remove(&b.id, 1).unwrap_err().code, "STALE_RECORD");
    c.finding_remove(&b.id, merged.revision).unwrap();
    c.evidence_verify(&e.id).unwrap();
}
#[test]
fn cvss_reference_scores_and_incomplete_vectors() {
    assert_eq!(
        calculate_cvss("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H")
            .unwrap()
            .score,
        9.8
    );
    assert_eq!(
        calculate_cvss("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:N")
            .unwrap()
            .score,
        0.0
    );
    assert_eq!(
        calculate_cvss("CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:H/SI:H/SA:H")
            .unwrap()
            .score,
        10.0
    );
    assert_eq!(
        calculate_cvss("CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:N/VI:N/VA:N/SC:N/SI:N/SA:N")
            .unwrap()
            .score,
        0.0
    );
    for v in [
        "CVSS:3.1/AV:N",
        "CVSS:4.0/AV:N",
        "CVSS:3.0/AV:N",
        "CVSS:4.0/AV:WRONG",
    ] {
        assert_eq!(calculate_cvss(v).unwrap_err().code, "INVALID_CVSS");
    }
}
#[test]
fn reports_are_immutable_redacted_reviewed_and_export_four_real_formats() {
    let (d, c, p) = setup();
    let e=c.evidence_add(input(&p,EvidenceKind::HttpRequest),"GET / HTTP/1.1\nAuthorization: Bearer fixture-private-access\nCookie: session=fixture-private-session\n\npassword=fixture-private-password\n<script>alert(1)</script>\nC:\\private\\local\\machine.txt").unwrap();
    let mut f = finding(&p, &e.id);
    f.description =
        "Fixture observation; not a vulnerability. Kiểm tra bằng chứng tiếng Việt.".into();
    f.technical_details = "No exploit or vulnerability claim is made".into();
    let f = c.finding_save(None, f, None, false).unwrap();
    let view = c
        .report_prepare(ReportInput {
            project_id: p.clone(),
            title: "Local evidence assessment".into(),
            template: ReportTemplate::HackerOne,
            summary: "Owned fixture output".into(),
            methodology: "Offline import only".into(),
            finding_ids: vec![f.id.clone()],
            evidence_ids: vec![],
        })
        .unwrap();
    assert!(view.document.redactions >= 4);
    let json = serde_json::to_string(&view.document).unwrap();
    for secret in [
        "fixture-private-access",
        "fixture-private-session",
        "fixture-private-password",
        "machine.txt",
    ] {
        assert!(!json.contains(secret), "{secret} leaked");
    }
    assert!(view.document.text.contains("hypothesis"));
    assert!(view.document.text.contains("Steps to reproduce"));
    assert!(
        view.document
            .warnings
            .iter()
            .any(|w| w.contains("Hypothesis"))
    );
    assert_eq!(
        c.report_export(
            &view.report.id,
            ReportFormat::Json,
            &d.path().join("bad.json"),
            "unreviewed"
        )
        .unwrap_err()
        .code,
        "PREVIEW_REQUIRED"
    );
    for (format, name) in [
        (ReportFormat::Markdown, "report.md"),
        (ReportFormat::Html, "report.html"),
        (ReportFormat::Json, "report.json"),
        (ReportFormat::Pdf, "report.pdf"),
    ] {
        let path = d.path().join(name);
        c.report_export(&view.report.id, format, &path, &view.report.sha256)
            .unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.len() > 100);
        if name.ends_with("pdf") {
            assert!(bytes.starts_with(b"%PDF-"));
            assert!(bytes.windows(5).any(|w| w == b"%%EOF"));
        }
        if name.ends_with("html") {
            let html = String::from_utf8(bytes).unwrap();
            assert!(!html.contains("<script>"));
            assert!(html.contains("&lt;script&gt;"));
            assert!(html.contains("default-src 'none'"));
        }
    }
    c.finding_remove(&f.id, f.revision).unwrap();
    assert_eq!(
        c.report_show(&view.report.id).unwrap().document.text,
        view.document.text
    );
    let reopened = Core::open(Some(c.database_path().to_owned())).unwrap();
    assert_eq!(reopened.reports(Some(&p)).unwrap().len(), 1);
    fs::write(d.path().join(&e.storage_location), b"tampered").unwrap();
    assert_eq!(
        reopened
            .report_export(
                &view.report.id,
                ReportFormat::Json,
                &d.path().join("later.json"),
                &view.report.sha256
            )
            .unwrap_err()
            .code,
        "INTEGRITY_ERROR"
    );
}
#[test]
fn file_attachment_limits_types_and_raw_export_are_explicit() {
    let (d, c, p) = setup();
    let path = d.path().join("attachment.bin");
    fs::write(&path, b"owned fixture bytes").unwrap();
    let e = c
        .evidence_import(input(&p, EvidenceKind::Attachment), &path)
        .unwrap();
    assert_eq!(
        c.evidence_export(&e.id, &d.path().join("copy.bin"), false)
            .unwrap_err()
            .code,
        "RAW_APPROVAL_REQUIRED"
    );
    c.evidence_export(&e.id, &d.path().join("copy.bin"), true)
        .unwrap();
    assert_eq!(
        fs::read(d.path().join("copy.bin")).unwrap(),
        fs::read(&path).unwrap()
    );
    assert_eq!(
        c.evidence_import(input(&p, EvidenceKind::Screenshot), &path)
            .unwrap_err()
            .code,
        "INVALID_IMAGE"
    );
    fs::File::create(&path)
        .unwrap()
        .set_len(MAX_EVIDENCE_BYTES as u64 + 1)
        .unwrap();
    assert_eq!(
        c.evidence_import(input(&p, EvidenceKind::Attachment), &path)
            .unwrap_err()
            .code,
        "INPUT_LIMIT"
    );
    assert_eq!(
        c.evidence_verify("../escape").unwrap_err().code,
        "NOT_FOUND"
    );
}

#[test]
fn structured_exports_redact_credential_fields_without_breaking_json() {
    let (d, c, p) = setup();
    let e=c.evidence_add(input(&p,EvidenceKind::StructuredJson),r#"{"Authorization":"Basic fixture-private-auth","cookie":"fixture-private-cookie","nested":{"api_key":"fixture-private-key"},"headers":[{"name":"Set-Cookie","value":"fixture-private-session"},{"name":"Api-Key","value":"fixture-private-header-key"}],"observation":"actual benign data","sk-fixtureSecretKeyForRedaction":true}"#).unwrap();
    let path = d.path().join("safe.json");
    c.evidence_export(&e.id, &path, false).unwrap();
    let text = fs::read_to_string(path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(!text.contains("fixture-private"));
    assert_eq!(json["observation"], "actual benign data");
    assert_eq!(json["nested"]["api_key"], "[REDACTED]");
    let manual = c
        .evidence_add(
            input(&p, EvidenceKind::HttpResponse),
            r#"{"headers":[{"name":"Set-Cookie","value":"fixture-private-manual-cookie"}]}"#,
        )
        .unwrap();
    let manual_path = d.path().join("safe-manual.json");
    c.evidence_export(&manual.id, &manual_path, false).unwrap();
    assert!(
        !fs::read_to_string(manual_path)
            .unwrap()
            .contains("fixture-private-manual-cookie")
    );
    let report = c
        .report_prepare(ReportInput {
            project_id: p,
            title: "Structured HTTP evidence regression".into(),
            template: ReportTemplate::Research,
            summary: "Owned fixture only".into(),
            methodology: "Offline review".into(),
            finding_ids: vec![],
            evidence_ids: vec![e.id],
        })
        .unwrap();
    assert!(!report.document.text.contains("fixture-private"));
    assert!(
        !serde_json::to_string(&report.document)
            .unwrap()
            .contains("fixture-private")
    );
    assert!(report.document.redactions >= 5);
    for (format, name) in [
        (ReportFormat::Json, "structured-report.json"),
        (ReportFormat::Html, "structured-report.html"),
        (ReportFormat::Markdown, "structured-report.md"),
    ] {
        let path = d.path().join(name);
        c.report_export(&report.report.id, format, &path, &report.report.sha256)
            .unwrap();
        assert!(
            !fs::read_to_string(path)
                .unwrap()
                .contains("fixture-private")
        );
    }
}
