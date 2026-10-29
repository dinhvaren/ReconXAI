//! Immutable local artifacts. Integrity is not vulnerability verification.
use super::*;
use std::{
    fs,
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

pub const MAX_EVIDENCE_BYTES: usize = 32 * 1024 * 1024;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    HttpRequest,
    HttpResponse,
    ToolOutput,
    StructuredJson,
    TerminalLog,
    Screenshot,
    BinaryMetadata,
    StaticAnalysis,
    YaraMatches,
    SourceReference,
    McpInvocation,
    AiSummary,
    UserNote,
    Attachment,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    Public,
    Internal,
    #[default]
    Confidential,
    Restricted,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceInput {
    pub project_id: String,
    pub target_id: Option<String>,
    pub kind: EvidenceKind,
    pub title: String,
    #[serde(default)]
    pub source_tool: String,
    #[serde(default)]
    pub sensitivity: Sensitivity,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub job_id: Option<String>,
    pub kind: EvidenceKind,
    pub title: String,
    pub source_tool: String,
    pub created_at: String,
    pub sha256: String,
    pub size: u64,
    pub storage_location: String,
    pub media_type: String,
    pub verification_state: String,
    pub review_note: String,
    pub sensitivity: Sensitivity,
}
#[derive(Debug, Serialize)]
pub struct EvidenceView {
    pub evidence: Evidence,
    pub text: Option<String>,
    pub image_data: Option<String>,
    pub preview_omitted: bool,
}

fn evidence_io() -> AppError {
    AppError::new("ARTIFACT_IO", "Cannot access private research artifact")
}
pub(super) fn redirected(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            m.file_attributes() & 0x400 != 0
        }
        #[cfg(not(windows))]
        {
            m.file_type().is_symlink()
        }
    })
}
pub(super) fn read_regular(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let m = fs::symlink_metadata(path).map_err(|_| evidence_io())?;
    if !m.is_file() || redirected(path) {
        return Err(AppError::new(
            "UNSAFE_PATH",
            "Choose a regular file, not a symbolic link or reparse point",
        ));
    }
    if m.len() > limit as u64 {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "Artifact exceeds the supported byte limit",
        ));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| evidence_io())?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| evidence_io())?;
    if bytes.len() > limit {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "Artifact grew beyond its byte limit",
        ));
    }
    Ok(bytes)
}
pub(super) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub(super) fn export_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|_| {
        AppError::new(
            "OUTPUT_EXISTS",
            "Choose a writable new output filename; existing files are never overwritten",
        )
    })?;
    if file.write_all(bytes).and_then(|_| file.sync_all()).is_err() {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(evidence_io());
    }
    Ok(())
}
impl Core {
    pub(super) fn output_path(&self, category: &str, id: &str) -> Result<PathBuf> {
        Uuid::parse_str(id)
            .map_err(|_| AppError::new("INVALID_INPUT", "Artifact ID must be a UUID"))?;
        if !["evidence", "reports"].contains(&category) {
            return Err(AppError::new("UNSAFE_PATH", "Invalid artifact category"));
        }
        let parent = self
            .db
            .path()
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let root = parent.join(category);
        if redirected(&root) {
            return Err(AppError::new(
                "UNSAFE_PATH",
                "Private artifact directory cannot be redirected",
            ));
        }
        fs::create_dir_all(&root).map_err(|_| evidence_io())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .map_err(|_| evidence_io())?;
        }
        Ok(root.join(format!("{id}.bin")))
    }
    fn validate_evidence_input(&self, input: &mut EvidenceInput) -> Result<()> {
        let project = self.project(&input.project_id)?;
        input.project_id = project.id;
        if project.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore project before adding evidence",
            ));
        }
        input.title = validate_text(&input.title, 180, "Evidence title")?;
        if input.source_tool.len() > 120 {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "Source tool label is too long",
            ));
        }
        if let Some(id) = &input.target_id
            && self.target(id)?.input.project_id != input.project_id
        {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Target belongs to another project",
            ));
        }
        Ok(())
    }
    pub fn evidence_add(&self, mut input: EvidenceInput, text: &str) -> Result<Evidence> {
        self.validate_evidence_input(&mut input)?;
        if [EvidenceKind::Screenshot, EvidenceKind::Attachment].contains(&input.kind) {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Use file import for image/binary attachments",
            ));
        }
        if text.len() > 1024 * 1024 {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "Text evidence is limited to 1 MiB",
            ));
        }
        if input.kind == EvidenceKind::StructuredJson {
            let _: serde_json::Value = serde_json::from_str(text).map_err(json_error)?;
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let record = self.insert_evidence(
            &tx,
            input,
            None,
            text.as_bytes(),
            "text/plain",
            "unverified",
        )?;
        tx.commit().map_err(db_error)?;
        Ok(record)
    }
    pub fn evidence_import(&self, mut input: EvidenceInput, path: &Path) -> Result<Evidence> {
        self.validate_evidence_input(&mut input)?;
        let bytes = read_regular(path, MAX_EVIDENCE_BYTES)?;
        let media = if input.kind == EvidenceKind::Screenshot {
            if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                "image/png"
            } else if bytes.starts_with(b"\xff\xd8\xff") {
                "image/jpeg"
            } else {
                return Err(AppError::new(
                    "INVALID_IMAGE",
                    "Screenshot must contain PNG or JPEG data",
                ));
            }
        } else if input.kind == EvidenceKind::Attachment {
            "application/octet-stream"
        } else {
            std::str::from_utf8(&bytes).map_err(|_| {
                AppError::new("INVALID_TEXT", "Selected evidence type requires UTF-8 text")
            })?;
            if input.kind == EvidenceKind::StructuredJson {
                let _: serde_json::Value = serde_json::from_slice(&bytes).map_err(json_error)?;
            }
            "text/plain"
        };
        if media.starts_with("image/") {
            let size = imagesize::blob_size(&bytes)
                .map_err(|_| AppError::new("INVALID_IMAGE", "Cannot read screenshot dimensions"))?;
            if size.width == 0
                || size.height == 0
                || size.width > 8192
                || size.height > 8192
                || size.width.saturating_mul(size.height) > 16_000_000
            {
                return Err(AppError::new(
                    "INPUT_LIMIT",
                    "Screenshot is limited to 8192 pixels per side and 16 megapixels",
                ));
            }
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let record = self.insert_evidence(&tx, input, None, &bytes, media, "unverified")?;
        tx.commit().map_err(db_error)?;
        Ok(record)
    }
    fn insert_evidence(
        &self,
        conn: &Connection,
        input: EvidenceInput,
        job_id: Option<String>,
        bytes: &[u8],
        media: &str,
        state: &str,
    ) -> Result<Evidence> {
        if bytes.is_empty() || bytes.len() > MAX_EVIDENCE_BYTES {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "Evidence must contain 1 byte to 32 MiB",
            ));
        }
        let id = Uuid::new_v4().to_string();
        let path = self.output_path("evidence", &id)?;
        export_new(&path, bytes)?;
        let e = Evidence {
            id: id.clone(),
            project_id: input.project_id,
            target_id: input.target_id,
            job_id,
            kind: input.kind,
            title: input.title,
            source_tool: input.source_tool,
            created_at: now(),
            sha256: digest(bytes),
            size: bytes.len() as u64,
            storage_location: format!("evidence/{id}.bin"),
            media_type: media.into(),
            verification_state: state.into(),
            review_note: String::new(),
            sensitivity: input.sensitivity,
        };
        let kind = serde_json::to_value(e.kind)
            .map_err(json_error)?
            .as_str()
            .unwrap()
            .to_owned();
        if let Err(error)=conn.execute("INSERT INTO evidence(id,project_id,target_id,job_id,kind,title,created_at,metadata_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![e.id,e.project_id,e.target_id,e.job_id,kind,e.title,e.created_at,serde_json::to_string(&e).map_err(json_error)?]) {let _=fs::remove_file(path);return Err(db_error(error));}
        Self::audit(conn, "evidence.created", &e.id, "completed")?;
        Ok(e)
    }
    pub(super) fn insert_observed_evidence(
        &self,
        conn: &Connection,
        input: EvidenceInput,
        job: Option<String>,
        bytes: &[u8],
        media: &str,
    ) -> Result<Evidence> {
        self.insert_evidence(conn, input, job, bytes, media, "observed")
    }
    pub(super) fn record_job_evidence(
        &self,
        conn: &Connection,
        job: &AnalysisJob,
        result: &serde_json::Value,
        analysis: Option<&AnalysisRecord>,
    ) -> Result<Evidence> {
        let kind = match job.kind.as_str() {
            // Raw scanner output and normalized job results have distinct
            // provenance, and UNIQUE(job_id,kind) must retain both records.
            "recon_httpx" | "recon_subfinder" => EvidenceKind::StructuredJson,
            "mcp_tool" => EvidenceKind::McpInvocation,
            "yara_signatures" => EvidenceKind::YaraMatches,
            "binary_metadata" => EvidenceKind::BinaryMetadata,
            "local_static_binary" | "strings_extraction" => EvidenceKind::StaticAnalysis,
            _ => EvidenceKind::ToolOutput,
        };
        let content=serde_json::to_vec_pretty(&serde_json::json!({"job_id":job.id,"job_kind":job.kind,"observed_result":result,"analysis":analysis,"execution_log":job.logs,"interpretation":"Observed tool output; no vulnerability verification implied"})).map_err(json_error)?;
        self.insert_evidence(
            conn,
            EvidenceInput {
                project_id: job.project_id.clone(),
                target_id: job.target_id.clone(),
                kind,
                title: format!("{} result", job.kind),
                source_tool: job.kind.clone(),
                sensitivity: Sensitivity::Confidential,
            },
            Some(job.id.clone()),
            &content,
            "application/json",
            "observed",
        )
    }
    pub fn evidence_from_job(&self, id: &str) -> Result<Evidence> {
        let job = self.job(id)?;
        if job.status != "completed" {
            return Err(AppError::new(
                "JOB_NOT_COMPLETED",
                "Only completed jobs can become result evidence",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT id FROM evidence WHERE job_id=?1 LIMIT 1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        if let Some(existing) = existing {
            drop(tx);
            return self.evidence(&existing);
        }
        let result = job
            .result
            .as_ref()
            .ok_or_else(|| AppError::new("NO_RESULT", "Job has no stored result"))?;
        let analysis = if result.get("analysis_id").is_some() {
            Some(self.analysis_for_job(id)?)
        } else {
            None
        };
        let e = self.record_job_evidence(&tx, &job, result, analysis.as_ref())?;
        tx.commit().map_err(db_error)?;
        Ok(e)
    }
    pub fn evidence_list(&self, project: Option<&str>) -> Result<Vec<Evidence>> {
        let project = project.map(|p| self.project(p).map(|p| p.id)).transpose()?;
        let conn = self.db.connect()?;
        let mut stmt=conn.prepare("SELECT metadata_json,target_id,job_id FROM evidence WHERE ?1 IS NULL OR project_id=?1 ORDER BY created_at DESC,id LIMIT 1000").map_err(db_error)?;
        let rows = stmt
            .query_map([project], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(db_error)?;
        rows.map(|r| {
            let (json, target, job) = r.map_err(db_error)?;
            let mut e: Evidence = serde_json::from_str(&json).map_err(json_error)?;
            e.target_id = target;
            e.job_id = job;
            Ok(e)
        })
        .collect()
    }
    pub fn evidence(&self, id: &str) -> Result<Evidence> {
        let conn = self.db.connect()?;
        let row: Option<(String, Option<String>, Option<String>)> = conn
            .query_row(
                "SELECT metadata_json,target_id,job_id FROM evidence WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (json, target, job) =
            row.ok_or_else(|| AppError::new("NOT_FOUND", "Evidence does not exist"))?;
        let mut e: Evidence = serde_json::from_str(&json).map_err(json_error)?;
        e.target_id = target;
        e.job_id = job;
        Ok(e)
    }
    pub(super) fn evidence_bytes(&self, e: &Evidence) -> Result<Vec<u8>> {
        let bytes = read_regular(&self.output_path("evidence", &e.id)?, MAX_EVIDENCE_BYTES)?;
        if bytes.len() as u64 != e.size || digest(&bytes) != e.sha256 {
            return Err(AppError::new(
                "INTEGRITY_ERROR",
                "Evidence content changed; restore the original before using it",
            ));
        }
        Ok(bytes)
    }
    pub fn evidence_verify(&self, id: &str) -> Result<serde_json::Value> {
        let e = self.evidence(id)?;
        self.evidence_bytes(&e)?;
        Ok(
            serde_json::json!({"id":id,"integrity":"matched","sha256":e.sha256,"verification_state":e.verification_state,"message":"Hash integrity does not verify a vulnerability"}),
        )
    }
    pub fn evidence_show(&self, id: &str) -> Result<EvidenceView> {
        let e = self.evidence(id)?;
        let bytes = self.evidence_bytes(&e)?;
        let image_data = if e.media_type.starts_with("image/") && bytes.len() <= 8 * 1024 * 1024 {
            use base64::Engine;
            Some(format!(
                "data:{};base64,{}",
                e.media_type,
                base64::engine::general_purpose::STANDARD.encode(&bytes)
            ))
        } else {
            None
        };
        let text = if ["text/plain", "application/json"].contains(&e.media_type.as_str())
            && bytes.len() <= 1024 * 1024
        {
            Some(
                String::from_utf8(bytes)
                    .map_err(|_| AppError::new("INVALID_TEXT", "Evidence is not UTF-8"))?,
            )
        } else {
            None
        };
        let omitted = text.is_none() && image_data.is_none();
        Ok(EvidenceView {
            evidence: e,
            text,
            image_data,
            preview_omitted: omitted,
        })
    }
    pub fn evidence_review(&self, id: &str, note: &str) -> Result<Evidence> {
        let mut e = self.evidence(id)?;
        self.evidence_bytes(&e)?;
        if e.kind == EvidenceKind::AiSummary {
            return Err(AppError::new(
                "UNVERIFIED_AI",
                "AI summaries remain unverified; review supporting source evidence instead",
            ));
        }
        e.review_note = validate_text(note, 4096, "Manual review note")?;
        e.verification_state = "manually_reviewed".into();
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute(
            "UPDATE evidence SET metadata_json=?1 WHERE id=?2",
            params![serde_json::to_string(&e).map_err(json_error)?, id],
        )
        .map_err(db_error)?;
        Self::audit(&tx, "evidence.reviewed", id, "manual")?;
        tx.commit().map_err(db_error)?;
        Ok(e)
    }
    pub fn evidence_export(
        &self,
        id: &str,
        path: &Path,
        raw_approved: bool,
    ) -> Result<serde_json::Value> {
        let e = self.evidence(id)?;
        let bytes = self.evidence_bytes(&e)?;
        let (bytes, redactions) = if raw_approved {
            (bytes, 0)
        } else if ["text/plain", "application/json"].contains(&e.media_type.as_str()) {
            let text = std::str::from_utf8(&bytes)
                .map_err(|_| AppError::new("INVALID_TEXT", "Cannot redact non-text evidence"))?;
            if e.kind == EvidenceKind::StructuredJson || e.media_type == "application/json" {
                let mut json: serde_json::Value = serde_json::from_str(text).map_err(json_error)?;
                let count = super::reporting::redact_value(&mut json);
                (serde_json::to_vec_pretty(&json).map_err(json_error)?, count)
            } else {
                let (text, count) = super::reporting::redact_content(text);
                (text.into_bytes(), count)
            }
        } else {
            return Err(AppError::new(
                "RAW_APPROVAL_REQUIRED",
                "Images and binary attachments require explicit approval; they cannot be automatically redacted",
            ));
        };
        export_new(path, &bytes)?;
        Self::audit(
            &self.db.connect()?,
            "evidence.exported",
            id,
            if raw_approved {
                "raw_explicitly_approved"
            } else {
                "redacted"
            },
        )?;
        Ok(
            serde_json::json!({"id":id,"sha256":digest(&bytes),"redactions":redactions,"raw":raw_approved}),
        )
    }
}
