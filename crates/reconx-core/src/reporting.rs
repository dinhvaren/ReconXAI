//! Local immutable report snapshots. No raw attachment or active HTML rendering.
use super::evidence::{digest, export_new, read_regular};
use super::*;
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path, sync::OnceLock};
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportTemplate {
    Pentest,
    BugBounty,
    HackerOne,
    Executive,
    Technical,
    ReverseEngineering,
    Research,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportInput {
    pub project_id: String,
    pub title: String,
    pub template: ReportTemplate,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub methodology: String,
    #[serde(default)]
    pub finding_ids: Vec<String>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub template: ReportTemplate,
    pub created_at: String,
    pub sha256: String,
    pub size: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDocument {
    pub snapshot: Value,
    pub text: String,
    pub redactions: usize,
    pub warnings: Vec<String>,
    pub evidence_ids: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct ReportView {
    pub report: ReportRecord,
    pub document: ReportDocument,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    Markdown,
    Html,
    Pdf,
    Json,
}

pub(super) fn redact_text(text: &str) -> (String, usize) {
    static RULES: OnceLock<Vec<regex::Regex>> = OnceLock::new();
    let rules=RULES.get_or_init(||[
  r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
  r"(?im)(?:authorization|proxy-authorization|cookie|set-cookie)\s*:[^\r\n]+",
  r#"(?i)["'](?:authorization|proxy-authorization|cookie|set-cookie|private[_-]?key)["']\s*:\s*(?:"(?:\\.|[^"\\])*"|'[^']*')"#,
  r#"(?i)["']?(?:api[_-]?key|access[_-]?token|refresh[_-]?token|session[_-]?(?:id|token)|password|passwd|client[_-]?secret|secret|token)["']?\s*[:=]\s*(?:"[^"\r\n]*"|'[^'\r\n]*'|[^\s&,;\r\n}]+)"#,
  r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]+",
  r"\b(?:sk-[A-Za-z0-9_-]{12,}|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|AIza[A-Za-z0-9_-]{20,}|AKIA[A-Z0-9]{16})\b",
  r"(?i)[a-z][a-z0-9+.-]*://[^\s/@:]+:[^\s/@]+@",
  r#"(?im)(?P<prefix>^|[\s"'`(=])(?:\\\\\?\\)?[A-Z]:[\\/][^\r\n\t"<>]+"#,
  r#"(?m)(?P<prefix>^|[\s"'`(=])\\\\[^\r\n\t"<>]+"#,
  r#"(?m)(?P<prefix>^|[\s"'`(=])/(?:home|Users|private|tmp|var/tmp)/[^\r\n\t"<>]+"#,
 ].into_iter().map(|p|regex::Regex::new(p).expect("constant redaction pattern")).collect());
    let mut result = text.to_owned();
    let mut count = 0;
    for rule in rules {
        count += rule.find_iter(&result).count();
        result = rule
            .replace_all(&result, "${prefix}[REDACTED]")
            .into_owned();
    }
    (result, count)
}
pub(super) fn redact_value(value: &mut Value) -> usize {
    redact_inner(value, 0)
}
fn redact_inner(value: &mut Value, embedded_depth: u8) -> usize {
    match value {
        Value::String(s) => {
            let (new, count) = redact_content_depth(s, embedded_depth);
            *s = new;
            count
        }
        Value::Array(a) => a.iter_mut().map(|v| redact_inner(v, embedded_depth)).sum(),
        Value::Object(o) => {
            let paired_secret = o.get("name").and_then(Value::as_str).is_some_and(|name| {
                [
                    "authorization",
                    "proxy-authorization",
                    "cookie",
                    "set-cookie",
                    "x-api-key",
                    "api-key",
                    "api_key",
                    "apikey",
                    "x-api-token",
                    "x-auth-token",
                ]
                .contains(&name.to_ascii_lowercase().as_str())
            });
            let original = std::mem::take(o);
            let mut total = 0;
            for (key, mut value) in original {
                let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
                let count = if [
                    "authorization",
                    "proxyauthorization",
                    "cookie",
                    "setcookie",
                    "apikey",
                    "accesstoken",
                    "refreshtoken",
                    "sessiontoken",
                    "sessionid",
                    "password",
                    "passwd",
                    "clientsecret",
                    "secret",
                    "token",
                    "privatekey",
                    "xapikey",
                    "xapitoken",
                    "xauthtoken",
                    "awssecretaccesskey",
                    "awsaccesskeyid",
                ]
                .contains(&normalized.as_str())
                    || (paired_secret && matches!(key.as_str(), "value" | "value_base64"))
                {
                    let changed = value != Value::String("[REDACTED]".into());
                    value = Value::String("[REDACTED]".into());
                    usize::from(changed)
                } else {
                    redact_inner(&mut value, embedded_depth)
                };
                let (mut safe_key, key_redactions) = redact_text(&key);
                if key_redactions > 0 {
                    safe_key = format!("[REDACTED key {}]", o.len() + 1);
                }
                total += count + key_redactions;
                o.insert(safe_key, value);
            }
            total
        }
        _ => 0,
    }
}
// A JSON artifact must be redacted as structured data before it is embedded in
// the text report. Regex-only treatment misses name/value HTTP header pairs.
pub(super) fn redact_content(text: &str) -> (String, usize) {
    redact_content_depth(text, 0)
}
fn redact_content_depth(text: &str, embedded_depth: u8) -> (String, usize) {
    if matches!(text.trim_start().as_bytes().first(), Some(b'{' | b'['))
        && let Ok(mut value @ (Value::Object(_) | Value::Array(_))) = serde_json::from_str(text)
    {
        // Bound nested serialized JSON separately from serde_json's structural
        // recursion limit. Retain the original privately; show the omission.
        if embedded_depth >= 4 {
            return ("[REDACTED: nested JSON requires separate review]".into(), 1);
        }
        let count = redact_inner(&mut value, embedded_depth + 1);
        return (pretty(&value), count);
    }
    redact_text(text)
}

fn require_current_redaction(snapshot: &Value) -> Result<()> {
    if let Some(evidence) = snapshot["evidence"].as_array() {
        for record in evidence {
            if let Some(text) = record["content"].as_str()
                && let Ok(original @ (Value::Object(_) | Value::Array(_))) =
                    serde_json::from_str(text)
            {
                let mut safe = original.clone();
                redact_value(&mut safe);
                if safe != original {
                    return Err(AppError::new(
                        "REPORT_REDACTION_REQUIRED",
                        "This saved snapshot needs updated structured redaction. Prepare and review a new report before export; the original is preserved",
                    ));
                }
            }
        }
    }
    Ok(())
}
fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn markdown_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if "\\`*_{}[]()#+-.!|>".contains(c) {
            out.push('\\');
        }
        match c {
            '<' => out.push_str("&lt;"),
            '&' => out.push_str("&amp;"),
            _ => out.push(c),
        }
    }
    out
}
fn report_text(snapshot: &Value) -> String {
    fn section(out: &mut String, title: &str, body: &str) {
        out.push_str(&format!(
            "{title}\n{}\n{body}\n\n",
            "-".repeat(title.len().min(80))
        ));
    }
    let value = |v: &Value| v.as_str().unwrap_or("").to_owned();
    let template = snapshot["template"].as_str().unwrap_or("research");
    let bounty = matches!(template, "bug_bounty" | "hacker_one");
    let mut out = format!(
        "{}\nReconX AI | {}\nSnapshot: {}\n\n",
        value(&snapshot["title"]),
        template.replace('_', " "),
        value(&snapshot["created_at"])
    );
    section(&mut out, "Executive summary", &value(&snapshot["summary"]));
    let mut scope = String::new();
    if let Some(assets) = snapshot["scope"].as_array() {
        for a in assets {
            scope.push_str(&format!(
                "{} | {} | {}\nAuthorization declared: {}\nAllow: {}\nExclude: {}\n\n",
                value(&a["name"]),
                value(&a["kind"]),
                value(&a["value"]),
                a["authorized"],
                pretty(&a["allowed_scope"]),
                pretty(&a["excluded_scope"])
            ));
        }
    }
    section(
        &mut out,
        "Scope / registered assets (registration is not proof of testing)",
        &scope,
    );
    section(&mut out, "Methodology", &value(&snapshot["methodology"]));
    section(
        &mut out,
        "Tools used / evidence sources",
        &pretty(&snapshot["tools"]),
    );
    let findings = snapshot["findings"].as_array().cloned().unwrap_or_default();
    let summary = findings
        .iter()
        .map(|f| {
            format!(
                "{} | {} | {} | {}",
                value(&f["id"]),
                value(&f["input"]["severity"]),
                value(&f["input"]["status"]),
                value(&f["input"]["title"])
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    section(
        &mut out,
        "Findings summary",
        if summary.is_empty() {
            "No findings selected; no assurance of absence of vulnerabilities is implied."
        } else {
            &summary
        },
    );
    for finding in findings {
        let f = &finding["input"];
        section(
            &mut out,
            &format!("Finding: {}", value(&f["title"])),
            &format!(
                "ID: {} | Severity: {} | Status: {} | Confidence: {}\nAI-generated: {} | Revision: {}\nCVSS: {}\nCWE: {} | CVE: {}",
                value(&finding["id"]),
                value(&f["severity"]),
                value(&f["status"]),
                value(&f["confidence"]),
                f["ai_generated"],
                finding["revision"],
                pretty(&finding["cvss"]),
                value(&f["cwe"]),
                value(&f["cve"])
            ),
        );
        for (title, key) in [
            ("Asset", "affected_asset"),
            ("Weakness / category", "category"),
            ("Summary", "description"),
            (
                if bounty {
                    "Proof of concept / observed technical evidence"
                } else {
                    "Technical details"
                },
                "technical_details",
            ),
            ("Steps to reproduce", "reproduction_steps"),
            ("Impact", "impact"),
            ("Suggested remediation", "remediation"),
            ("Research notes", "notes"),
            ("Manual verification note", "verification_note"),
        ] {
            section(&mut out, title, &value(&f[key]));
        }
        section(
            &mut out,
            "Supporting evidence IDs",
            &pretty(&f["evidence_ids"]),
        );
        section(&mut out, "References", &pretty(&f["references"]));
    }
    if let Some(records) = snapshot["evidence"].as_array() {
        for e in records {
            section(
                &mut out,
                &format!("Evidence: {}", value(&e["title"])),
                &format!(
                    "ID: {}\nType: {} | Source: {}\nTarget: {} | Job: {}\nCreated: {} | Review: {} | Sensitivity: {}\nSHA-256: {} | Bytes: {}\nReview note: {}\n\n{}",
                    value(&e["id"]),
                    value(&e["kind"]),
                    value(&e["source_tool"]),
                    value(&e["target_id"]),
                    value(&e["job_id"]),
                    value(&e["created_at"]),
                    value(&e["verification_state"]),
                    value(&e["sensitivity"]),
                    value(&e["sha256"]),
                    e["size"],
                    value(&e["review_note"]),
                    e["content"]
                        .as_str()
                        .unwrap_or("Content omitted; original retained separately.")
                ),
            );
        }
    }
    section(
        &mut out,
        "Appendix / interpretation limits",
        "All findings retain their recorded verification status. AI-generated content is unverified until separately reviewed. Signatures are observations, not proof of vulnerability. Redactions appear as [REDACTED]. Raw binary/image attachments are excluded and require separate review. Unknown secrets may evade automatic redaction; review before sharing.",
    );
    out
}
fn pdf(text: &str) -> Result<Vec<u8>> {
    use krilla::{
        Document,
        geom::Point,
        page::PageSettings,
        text::{Font, Tag, TextDirection},
    };
    const FONT: &[u8] = include_bytes!(
        "../../../apps/desktop/src/assets/fonts/sourcesans3/SourceSans3VF-Upright.ttf"
    );
    let mut face = ttf_parser::Face::parse(FONT, 0)
        .map_err(|_| AppError::new("PDF_FONT", "Cannot parse bundled report font"))?;
    face.set_variation(ttf_parser::Tag::from_bytes(b"wght"), 400.0)
        .ok_or_else(|| AppError::new("PDF_FONT", "Cannot select regular report font weight"))?;
    let font = Font::new_variable(FONT.to_vec().into(), 0, &[(Tag::new(b"wght"), 400.0)])
        .ok_or_else(|| AppError::new("PDF_FONT", "Cannot load bundled report font"))?;
    let mut lines = Vec::new();
    for raw in text.lines() {
        let mut line = String::new();
        let mut width = 0.0;
        for c in raw.chars() {
            let c = if c == '\t' { ' ' } else { c };
            if c.is_control() {
                continue;
            }
            let glyph=face.glyph_index(c).ok_or_else(||AppError::new("PDF_GLYPH","Report contains a character unsupported by the bundled font; use HTML/JSON or edit the preview source"))?;
            let advance = face.glyph_hor_advance(glyph).unwrap_or(0) as f32
                / face.units_per_em() as f32
                * 10.5;
            if width + advance > 480.0 && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                width = 0.0;
            }
            line.push(c);
            width += advance;
        }
        lines.push(line);
    }
    if lines.len() > 20_000 {
        return Err(AppError::new(
            "REPORT_LIMIT",
            "PDF exceeds 400 pages; select fewer records",
        ));
    }
    let mut doc = Document::new();
    for (index, chunk) in lines.chunks(49).enumerate() {
        let mut page = doc.start_page_with(PageSettings::from_wh(595.0, 842.0).unwrap());
        let mut surface = page.surface();
        surface.draw_text(
            Point::from_xy(48.0, 36.0),
            font.clone(),
            9.0,
            "ReconX AI - research report",
            false,
            TextDirection::LeftToRight,
        );
        for (i, line) in chunk.iter().enumerate() {
            surface.draw_text(
                Point::from_xy(48.0, 66.0 + i as f32 * 14.0),
                font.clone(),
                10.5,
                line,
                false,
                TextDirection::LeftToRight,
            );
        }
        surface.draw_text(
            Point::from_xy(48.0, 800.0),
            font.clone(),
            9.0,
            &format!(
                "Page {} | Redacted snapshot; verify evidence references",
                index + 1
            ),
            false,
            TextDirection::LeftToRight,
        );
        surface.finish();
        page.finish();
    }
    doc.finish()
        .map_err(|_| AppError::new("PDF_RENDER", "Cannot render report PDF"))
}
impl Core {
    pub fn report_prepare(&self, mut input: ReportInput) -> Result<ReportView> {
        let project = self.project(&input.project_id)?;
        input.project_id = project.id.clone();
        input.title = validate_text(&input.title, 200, "Report title")?;
        if input.summary.len() > 16 * 1024
            || input.methodology.len() > 16 * 1024
            || input.finding_ids.len() > 100
            || input.evidence_ids.len() > 100
        {
            return Err(AppError::new(
                "REPORT_LIMIT",
                "Select at most 100 findings/evidence and use summaries under 16 KiB",
            ));
        }
        input.finding_ids.sort();
        input.finding_ids.dedup();
        let mut findings = Vec::new();
        let mut evidence_ids: BTreeSet<String> = input.evidence_ids.into_iter().collect();
        let mut warnings = Vec::new();
        for id in &input.finding_ids {
            let f = self.finding(id)?;
            if f.input.project_id != project.id {
                return Err(AppError::new(
                    "PROJECT_MISMATCH",
                    "Finding belongs to another project",
                ));
            }
            if f.input.status != FindingStatus::Verified {
                warnings.push(format!(
                    "Finding {} is {:?}; no verified vulnerability is implied",
                    f.id, f.input.status
                ));
            }
            if f.input.status == FindingStatus::Verified
                && !f.input.evidence_ids.iter().any(|id| {
                    self.evidence(id).is_ok_and(|e| {
                        e.verification_state == "manually_reviewed"
                            && e.kind != EvidenceKind::AiSummary
                    })
                })
            {
                return Err(AppError::new(
                    "EVIDENCE_REQUIRED",
                    "Verified finding has no reviewed source evidence",
                ));
            }
            evidence_ids.extend(f.input.evidence_ids.iter().cloned());
            findings.push(f);
        }
        if evidence_ids.len() > 100 {
            return Err(AppError::new(
                "REPORT_LIMIT",
                "Report may contain at most 100 distinct evidence records",
            ));
        }
        let mut evidence = Vec::new();
        let mut tools = BTreeSet::new();
        let mut content_redactions = 0;
        for id in &evidence_ids {
            let e = self.evidence(id)?;
            if e.project_id != project.id {
                return Err(AppError::new(
                    "PROJECT_MISMATCH",
                    "Evidence belongs to another project",
                ));
            }
            let bytes = self.evidence_bytes(&e)?;
            tools.insert(e.source_tool.clone());
            let content = if ["text/plain", "application/json"].contains(&e.media_type.as_str()) {
                if bytes.len() > 64 * 1024 {
                    warnings.push(format!("Evidence {id}: text omitted from report because it exceeds 64 KiB; metadata/hash retained"));
                    None
                } else {
                    let text = String::from_utf8(bytes)
                        .map_err(|_| AppError::new("INVALID_TEXT", "Evidence is not UTF-8"))?;
                    let (safe, count) = redact_content(&text);
                    content_redactions += count;
                    Some(safe)
                }
            } else {
                warnings.push(format!(
                    "Evidence {id}: raw binary/image omitted; review and export separately"
                ));
                None
            };
            evidence.push(json!({"id":e.id,"title":e.title,"kind":e.kind,"target_id":e.target_id,"job_id":e.job_id,"source_tool":e.source_tool,"created_at":e.created_at,"sha256":e.sha256,"size":e.size,"verification_state":e.verification_state,"review_note":e.review_note,"sensitivity":e.sensitivity,"content":content}));
        }
        if findings.is_empty() {
            warnings.push("No findings selected. This report does not establish that an assessment found no vulnerabilities.".into());
        }
        let scope:Vec<_>=self.list_targets(Some(&project.id))?.into_iter().map(|t|json!({"id":t.id,"name":t.input.name,"kind":t.input.kind,"value":t.input.value,"allowed_scope":t.input.allowed_scope,"excluded_scope":t.input.excluded_scope,"authorized":t.input.authorized})).collect();
        let stamp = now();
        let mut snapshot = json!({"format_version":1,"title":input.title,"template":input.template,"created_at":stamp,"project":{"id":project.id,"name":project.name,"description":project.description},"summary":input.summary,"methodology":input.methodology,"scope":scope,"tools":tools,"findings":findings,"evidence":evidence});
        let redactions = content_redactions + redact_value(&mut snapshot);
        if redactions > 0 {
            warnings.push(format!("{redactions} credential/path redactions applied. Review all content before sharing; automatic detection cannot identify every secret."));
        }
        let mut text = report_text(&snapshot);
        text.push_str(&format!(
            "\nExport quality / redactions: {redactions}\n{}\n",
            warnings.join("\n")
        ));
        let document = ReportDocument {
            snapshot,
            text,
            redactions,
            warnings,
            evidence_ids: evidence_ids.into_iter().collect(),
        };
        let bytes = serde_json::to_vec(&document).map_err(json_error)?;
        if bytes.len() > 8 * 1024 * 1024 {
            return Err(AppError::new(
                "REPORT_LIMIT",
                "Report exceeds 8 MiB; select fewer records",
            ));
        }
        let record = ReportRecord {
            id: Uuid::new_v4().to_string(),
            project_id: project.id,
            title: document.snapshot["title"]
                .as_str()
                .unwrap_or("Research report")
                .into(),
            template: input.template,
            created_at: stamp,
            sha256: digest(&bytes),
            size: bytes.len() as u64,
        };
        let path = self.output_path("reports", &record.id)?;
        export_new(&path, &bytes)?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let template = serde_json::to_value(record.template).map_err(json_error)?;
        tx.execute("INSERT INTO reports(id,project_id,title,template,created_at,sha256,size) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![record.id,record.project_id,record.title,template.as_str(),record.created_at,record.sha256,record.size as i64]).map_err(db_error)?;
        Self::audit(&tx, "report.prepared", &record.id, "redacted_snapshot")?;
        tx.commit().map_err(db_error)?;
        Ok(ReportView {
            report: record,
            document,
        })
    }
    pub fn reports(&self, project: Option<&str>) -> Result<Vec<ReportRecord>> {
        let project = project.map(|p| self.project(p).map(|p| p.id)).transpose()?;
        let conn = self.db.connect()?;
        let mut stmt=conn.prepare("SELECT id,project_id,title,template,created_at,sha256,size FROM reports WHERE ?1 IS NULL OR project_id=?1 ORDER BY created_at DESC,id LIMIT 1000").map_err(db_error)?;
        let rows = stmt
            .query_map([project], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, u32>(6)?,
                ))
            })
            .map_err(db_error)?;
        rows.map(|r| {
            let (id, project_id, title, template, created_at, sha256, size) =
                r.map_err(db_error)?;
            Ok(ReportRecord {
                id,
                project_id,
                title,
                template: serde_json::from_value(Value::String(template)).map_err(json_error)?,
                created_at,
                sha256,
                size: size as u64,
            })
        })
        .collect()
    }
    pub fn report_show(&self, id: &str) -> Result<ReportView> {
        let conn = self.db.connect()?;
        let record = conn
            .query_row(
                "SELECT project_id,title,template,created_at,sha256,size FROM reports WHERE id=?1",
                [id],
                |r| {
                    let template: String = r.get(2)?;
                    let template =
                        serde_json::from_value(Value::String(template)).map_err(|e| {
                            reconx_storage::rusqlite::Error::FromSqlConversionFailure(
                                2,
                                reconx_storage::rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;
                    Ok(ReportRecord {
                        id: id.into(),
                        project_id: r.get(0)?,
                        title: r.get(1)?,
                        template,
                        created_at: r.get(3)?,
                        sha256: r.get(4)?,
                        size: r.get::<_, u32>(5)? as u64,
                    })
                },
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| AppError::new("NOT_FOUND", "Report does not exist"))?;
        let bytes = read_regular(&self.output_path("reports", id)?, 8 * 1024 * 1024)?;
        if digest(&bytes) != record.sha256 || bytes.len() as u64 != record.size {
            return Err(AppError::new("INTEGRITY_ERROR", "Report snapshot changed"));
        }
        Ok(ReportView {
            report: record,
            document: serde_json::from_slice(&bytes).map_err(json_error)?,
        })
    }
    pub fn report_export(
        &self,
        id: &str,
        format: ReportFormat,
        path: &Path,
        reviewed_sha256: &str,
    ) -> Result<Value> {
        let view = self.report_show(id)?;
        if reviewed_sha256 != view.report.sha256 {
            return Err(AppError::new(
                "PREVIEW_REQUIRED",
                "Review the exact saved report preview and supply its SHA-256 before export",
            ));
        }
        for e in &view.document.evidence_ids {
            self.evidence_verify(e)?;
        }
        require_current_redaction(&view.document.snapshot)?;
        let bytes=match format {
   ReportFormat::Json=>serde_json::to_vec_pretty(&view.document).map_err(json_error)?,
   ReportFormat::Markdown=>markdown_escape(&view.document.text).into_bytes(),
   ReportFormat::Html=>format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'\"><title>{}</title><style>body{{font:15px/1.55 system-ui,sans-serif;max-width:1000px;margin:3rem auto;padding:0 2rem;color:#18202a}}pre{{white-space:pre-wrap;overflow-wrap:anywhere;font:inherit}}@media print{{body{{margin:0}}}}</style></head><body><pre>{}</pre></body></html>",html_escape(&view.report.title),html_escape(&view.document.text)).into_bytes(),
   ReportFormat::Pdf=>pdf(&view.document.text)?,
  };
        export_new(path, &bytes)?;
        let format = serde_json::to_value(format).map_err(json_error)?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO report_exports(id,report_id,format,sha256,created_at) VALUES(?1,?2,?3,?4,?5)",params![Uuid::new_v4().to_string(),id,format.as_str(),digest(&bytes),now()]).map_err(db_error)?;
        Self::audit(
            &tx,
            "report.exported",
            id,
            format.as_str().unwrap_or_default(),
        )?;
        tx.commit().map_err(db_error)?;
        Ok(
            json!({"report_id":id,"format":format,"sha256":digest(&bytes),"bytes":bytes.len(),"redactions":view.document.redactions}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_report_with_raw_json_header_pairs_cannot_be_exported() {
        let unsafe_content = r#"{"response":{"headers":[{"name":"Set-Cookie","value":"fixture-private-cookie"},{"name":"Authorization","value_base64":"Zml4dHVyZS1wcml2YXRlLWF1dGg="}]}}"#;
        let unsafe_snapshot = json!({"evidence":[{"content":unsafe_content}]});
        assert_eq!(
            require_current_redaction(&unsafe_snapshot)
                .unwrap_err()
                .code,
            "REPORT_REDACTION_REQUIRED"
        );
        let (safe, count) = redact_content(unsafe_content);
        assert_eq!(count, 2);
        assert!(!safe.contains("fixture-private-cookie"));
        assert!(!safe.contains("Zml4dHVyZS1wcml2YXRlLWF1dGg="));
        require_current_redaction(&json!({"evidence":[{"content":safe}]})).unwrap();
        let mut nested = unsafe_content.to_owned();
        for _ in 0..8 {
            nested = serde_json::to_string(&json!({"embedded": nested})).unwrap();
        }
        let (safe, _) = redact_content(&nested);
        assert!(!safe.contains("fixture-private-cookie"));
        assert!(safe.contains("nested JSON requires separate review"));
        for url in [
            "http://127.0.0.1:31234/",
            "https://example.test/tmp/artifact",
        ] {
            assert_eq!(redact_text(url).0, url);
        }
        for path in [
            r"C:\Users\owned\private.txt",
            "/home/owned/private.txt",
            r"\\owned\private\file",
        ] {
            assert_eq!(redact_text(path).0, "[REDACTED]");
        }
    }
}
