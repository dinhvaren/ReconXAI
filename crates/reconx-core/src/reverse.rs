use super::*;
pub use reconx_analysis::{BinaryAnalysis, BinaryString, Section};
use reconx_analysis::{MAX_FILE_BYTES, hash, inspect};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedFile {
    pub id: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub name: String,
    pub sha256: String,
    pub size: u64,
    pub format: String,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisJob {
    pub id: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub file_id: Option<String>,
    pub kind: String,
    pub status: String,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub error_code: Option<String>,
    pub logs: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRecord {
    pub id: String,
    pub file_id: String,
    pub job_id: String,
    pub created_at: String,
    pub analysis: BinaryAnalysis,
}
#[derive(Debug, Serialize)]
pub struct ImportBatch {
    pub imported: Vec<ImportedFile>,
    pub failures: Vec<ImportFailure>,
}
#[derive(Debug, Serialize)]
pub struct ImportFailure {
    pub name: String,
    pub error_code: String,
}
#[derive(Debug, Serialize)]
pub struct BinaryComparison {
    pub identical: bool,
    pub a_sha256: String,
    pub b_sha256: String,
    pub a_size: u64,
    pub b_size: u64,
    pub changed_bytes: u64,
    pub sections_added: Vec<String>,
    pub sections_removed: Vec<String>,
    pub imports_added: Vec<String>,
    pub imports_removed: Vec<String>,
}
fn file_error() -> AppError {
    AppError::new(
        "FILE_IO",
        "Cannot access the selected local file or managed artifact",
    )
}
fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).map_err(|_| file_error())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(AppError::new(
            "INVALID_FILE",
            "Choose a regular local file, not a directory or symbolic link",
        ));
    }
    if metadata.len() > limit as u64 {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "File exceeds the configured input limit",
        ));
    }
    let file = File::open(path).map_err(|_| file_error())?;
    if !file.metadata().map_err(|_| file_error())?.is_file() {
        return Err(AppError::new("INVALID_FILE", "Input is not a regular file"));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| file_error())?;
    if bytes.len() > limit {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "File grew past the configured input limit",
        ));
    }
    Ok(bytes)
}
impl Core {
    fn artifact_directory(&self) -> Result<PathBuf> {
        let parent = self
            .db
            .path()
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let path = parent.join("imports");
        // Refuse a redirected managed root. Originals are never modified or executed.
        if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(AppError::new(
                "UNSAFE_PATH",
                "Managed import directory cannot be a symbolic link",
            ));
        }
        fs::create_dir_all(&path).map_err(|_| file_error())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
                .map_err(|_| file_error())?;
        }
        Ok(path)
    }
    fn blob_path(&self, sha: &str) -> Result<PathBuf> {
        if sha.len() != 64 || !sha.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::new("INVALID_DATA", "Invalid artifact identifier"));
        }
        Ok(self.artifact_directory()?.join(format!("{sha}.bin")))
    }
    pub fn import_file(
        &self,
        project: &str,
        path: &Path,
        authorized: bool,
    ) -> Result<ImportedFile> {
        let project = self.project(project)?;
        if project.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project before importing files",
            ));
        }
        let bytes = read_bounded(path, MAX_FILE_BYTES)?;
        let analysis = inspect(&bytes)?;
        let canonical = path
            .canonicalize()
            .map_err(|_| file_error())?
            .to_string_lossy()
            .into_owned();
        let target = match self
            .list_targets(Some(&project.id))?
            .into_iter()
            .find(|t| t.input.kind.local_file() && t.input.value == canonical)
        {
            Some(t) => t,
            None => self.create_target(TargetInput {
                project_id: project.id.clone(),
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .chars()
                    .take(120)
                    .collect(),
                kind: if analysis.format == "Android APK" {
                    TargetKind::Apk
                } else {
                    TargetKind::Binary
                },
                value: canonical.clone(),
                description: "Local file registered through Reverse Engineering import".into(),
                tags: vec!["local-import".into()],
                allowed_scope: vec![],
                excluded_scope: vec![],
                authorized,
                mode: SecurityMode::PassiveAuto,
            })?,
        };
        let destination = self.blob_path(&analysis.sha256)?;
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&destination)
        {
            Ok(mut f) => {
                f.write_all(&bytes).map_err(|_| file_error())?;
                f.sync_all().map_err(|_| file_error())?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if hash(&read_bounded(&destination, MAX_FILE_BYTES)?) != analysis.sha256 {
                    return Err(AppError::new(
                        "INTEGRITY_ERROR",
                        "Existing managed sample does not match its content hash",
                    ));
                }
            }
            Err(_) => return Err(file_error()),
        }
        let imported = ImportedFile {
            id: Uuid::new_v4().to_string(),
            project_id: project.id,
            target_id: Some(target.id),
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .chars()
                .take(255)
                .collect(),
            sha256: analysis.sha256,
            size: bytes.len() as u64,
            format: analysis.format,
            created_at: now(),
        };
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO imported_files(id,project_id,target_id,name,sha256,size,format,created_at,source_value) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![imported.id,imported.project_id,imported.target_id,imported.name,imported.sha256,imported.size as u32,imported.format,imported.created_at,canonical]).map_err(db_error)?;
        Self::audit(&tx, "file.imported", &imported.id, "completed")?;
        tx.commit().map_err(db_error)?;
        Ok(imported)
    }
    pub fn import_folder(
        &self,
        project: &str,
        path: &Path,
        authorized: bool,
    ) -> Result<ImportBatch> {
        let metadata = fs::symlink_metadata(path).map_err(|_| file_error())?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(AppError::new(
                "INVALID_FILE",
                "Choose a local directory, not a symbolic link",
            ));
        }
        let entries = fs::read_dir(path)
            .map_err(|_| file_error())?
            .take(101)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| file_error())?;
        if entries.len() > 100 {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "Folder import is limited to 100 direct entries; choose a smaller folder",
            ));
        }
        let mut result = ImportBatch {
            imported: vec![],
            failures: vec![],
        };
        for entry in entries {
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            match self.import_file(project, &entry.path(), authorized) {
                Ok(file) => result.imported.push(file),
                Err(e) => result.failures.push(ImportFailure {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    error_code: e.code,
                }),
            }
        }
        Ok(result)
    }
    pub fn list_files(&self, project: Option<&str>) -> Result<Vec<ImportedFile>> {
        let project = project.map(|p| self.project(p).map(|p| p.id)).transpose()?;
        let conn = self.db.connect()?;
        let mut stmt=conn.prepare("SELECT id,project_id,target_id,name,sha256,size,format,created_at FROM imported_files WHERE ?1 IS NULL OR project_id=?1 ORDER BY created_at DESC,id").map_err(db_error)?;
        stmt.query_map([project], |r| {
            Ok(ImportedFile {
                id: r.get(0)?,
                project_id: r.get(1)?,
                target_id: r.get(2)?,
                name: r.get(3)?,
                sha256: r.get(4)?,
                size: r.get::<_, u32>(5)? as u64,
                format: r.get(6)?,
                created_at: r.get(7)?,
            })
        })
        .map_err(db_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_error)
    }
    pub fn imported_file(&self, id: &str) -> Result<ImportedFile> {
        self.list_files(None)?
            .into_iter()
            .find(|f| f.id == id)
            .ok_or_else(|| AppError::new("NOT_FOUND", "Imported file does not exist"))
    }
    pub fn remove_file(&self, id: &str) -> Result<()> {
        self.imported_file(id)?;
        if self
            .list_jobs(None)?
            .iter()
            .any(|j| j.file_id.as_deref() == Some(id) && j.status == "running")
        {
            return Err(AppError::new(
                "JOB_RUNNING",
                "Cancel the running analysis before removing its file",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("DELETE FROM imported_files WHERE id=?1", [id])
            .map_err(db_error)?;
        Self::audit(&tx, "file.removed", id, "completed")?;
        tx.commit().map_err(db_error)
    }
    fn sample_bytes(&self, file: &ImportedFile) -> Result<Vec<u8>> {
        let bytes = read_bounded(&self.blob_path(&file.sha256)?, MAX_FILE_BYTES)?;
        if hash(&bytes) != file.sha256 {
            return Err(AppError::new(
                "INTEGRITY_ERROR",
                "Managed sample changed; analysis was refused",
            ));
        }
        Ok(bytes)
    }
    pub fn begin_file_analysis(&self, id: &str) -> Result<AnalysisJob> {
        let file = self.imported_file(id)?;
        self.require_file_authorization(&file)?;
        let job = AnalysisJob {
            id: Uuid::new_v4().to_string(),
            project_id: file.project_id,
            target_id: file.target_id,
            file_id: Some(id.into()),
            kind: "local_static_binary".into(),
            status: "running".into(),
            created_at: now(),
            finished_at: None,
            error_code: None,
            logs: vec![
                "Policy checked: read-only local static analysis; sample execution disabled".into(),
            ],
        };
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO analysis_jobs(id,project_id,target_id,file_id,kind,status,created_at,logs_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![job.id,job.project_id,job.target_id,job.file_id,job.kind,job.status,job.created_at,serde_json::to_string(&job.logs).map_err(json_error)?]).map_err(db_error)?;
        Self::audit(&tx, "analysis.started", &job.id, "running")?;
        tx.commit().map_err(db_error)?;
        Ok(job)
    }
    fn require_file_authorization(&self, file: &ImportedFile) -> Result<()> {
        let target = file.target_id.as_deref().ok_or_else(|| {
            AppError::new(
                "BLOCKED_BY_POLICY",
                "The imported file must be linked to an authorized target",
            )
        })?;
        let registered = self.target(target)?;
        let source: String = self
            .db
            .connect()?
            .query_row(
                "SELECT source_value FROM imported_files WHERE id=?1",
                [&file.id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if !registered.input.kind.local_file() || registered.input.value != source {
            return Err(AppError::new(
                "BLOCKED_BY_POLICY",
                "Target identity changed after import; import the file again under its correct target",
            ));
        }
        let decision = self.validate_scope(target, None)?;
        if decision.status != "allowed" {
            return Err(AppError::new("BLOCKED_BY_POLICY", &decision.reason));
        }
        if self.project(&file.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project before starting analysis",
            ));
        }
        Ok(())
    }
    fn job_checkpoint(&self, id: &str, message: &str) -> Result<()> {
        let mut job = self.job(id)?;
        if job.status != "running" {
            return Err(AppError::new(
                "CANCELLED",
                "Analysis job is no longer running",
            ));
        }
        let file = self.imported_file(
            job.file_id
                .as_deref()
                .ok_or_else(|| AppError::new("NOT_FOUND", "The job's file was removed"))?,
        )?;
        self.require_file_authorization(&file)?;
        job.logs.push(message.into());
        self.db
            .connect()?
            .execute(
                "UPDATE analysis_jobs SET logs_json=?1 WHERE id=?2 AND status='running'",
                params![serde_json::to_string(&job.logs).map_err(json_error)?, id],
            )
            .map_err(db_error)?;
        Ok(())
    }
    pub fn run_file_analysis(&self, job_id: &str) -> Result<AnalysisRecord> {
        let outcome = (|| {
            let job = self.job(job_id)?;
            if job.kind != "local_static_binary" {
                return Err(AppError::new("INVALID_JOB", "Unsupported analysis job"));
            }
            let file = self.imported_file(
                job.file_id
                    .as_deref()
                    .ok_or_else(|| AppError::new("NOT_FOUND", "Job file was removed"))?,
            )?;
            self.job_checkpoint(job_id, "Reading managed sample and verifying SHA-256")?;
            let bytes = self.sample_bytes(&file)?;
            self.job_checkpoint(
                job_id,
                "Parsing content signatures, sections, imports, exports and bounded strings",
            )?;
            let analysis = inspect(&bytes)?;
            self.job_checkpoint(
                job_id,
                "Static parsing completed; saving local analysis artifact",
            )?;
            let record = AnalysisRecord {
                id: Uuid::new_v4().to_string(),
                file_id: file.id.clone(),
                job_id: job_id.into(),
                created_at: now(),
                analysis,
            };
            let artifact_name = format!("{}.json", record.id);
            // Raw strings/hex stay in local artifact files, not ordinary SQLite fields.
            let mut artifact = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.artifact_directory()?.join(&artifact_name))
                .map_err(|_| file_error())?;
            let artifact_bytes = serde_json::to_vec(&record).map_err(json_error)?;
            artifact
                .write_all(&artifact_bytes)
                .map_err(|_| file_error())?;
            artifact.sync_all().map_err(|_| file_error())?;
            let mut conn = self.db.connect()?;
            let tx = conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(db_error)?;
            let changed=tx.execute("UPDATE analysis_jobs SET status='completed',finished_at=?1 WHERE id=?2 AND status='running'",params![now(),job_id]).map_err(db_error)?;
            if changed != 1 {
                return Err(AppError::new(
                    "CANCELLED",
                    "Analysis cancelled before saving",
                ));
            }
            tx.execute("INSERT INTO binary_analyses(id,file_id,job_id,created_at,artifact_name,artifact_sha256) VALUES(?1,?2,?3,?4,?5,?6)",params![record.id,record.file_id,record.job_id,record.created_at,artifact_name,hash(&artifact_bytes)]).map_err(db_error)?;
            if let Some(target) = file.target_id {
                tx.execute(
                    "UPDATE targets SET last_analyzed_at=?1 WHERE id=?2",
                    params![now(), target],
                )
                .map_err(db_error)?;
            }
            Self::audit(&tx, "analysis.completed", job_id, "completed")?;
            tx.commit().map_err(db_error)?;
            Ok(record)
        })();
        if let Err(error) = &outcome {
            let conn = self.db.connect()?;
            conn.execute("UPDATE analysis_jobs SET status='failed',error_code=?1,finished_at=?2 WHERE id=?3 AND status='running'",params![error.code,now(),job_id]).map_err(db_error)?;
            Self::audit(&conn, "analysis.failed", job_id, &error.code)?;
        }
        outcome
    }
    pub fn analyze_file(&self, id: &str) -> Result<AnalysisRecord> {
        let job = self.begin_file_analysis(id)?;
        self.run_file_analysis(&job.id)
    }
    pub fn file_analysis(&self, id: &str) -> Result<AnalysisRecord> {
        let conn = self.db.connect()?;
        let reference:Option<(String,String)>=conn.query_row("SELECT artifact_name,artifact_sha256 FROM binary_analyses WHERE file_id=?1 ORDER BY created_at DESC,id DESC LIMIT 1",[id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db_error)?;
        let (name, digest) = reference.ok_or_else(|| {
            AppError::new(
                "NOT_ANALYZED",
                "Start a static analysis to create the first result",
            )
        })?;
        if Path::new(&name).components().count() != 1 || !name.ends_with(".json") {
            return Err(AppError::new(
                "INVALID_DATA",
                "Invalid analysis artifact reference",
            ));
        }
        let artifact_bytes =
            read_bounded(&self.artifact_directory()?.join(name), 16 * 1024 * 1024)?;
        if hash(&artifact_bytes) != digest {
            return Err(AppError::new(
                "INTEGRITY_ERROR",
                "Analysis artifact changed after it was saved",
            ));
        }
        let record: AnalysisRecord = serde_json::from_slice(&artifact_bytes).map_err(json_error)?;
        let file = self.imported_file(id)?;
        if record.file_id != id || record.analysis.sha256 != file.sha256 {
            return Err(AppError::new(
                "INTEGRITY_ERROR",
                "Analysis artifact does not match its sample",
            ));
        }
        Ok(record)
    }
    pub fn list_jobs(&self, project: Option<&str>) -> Result<Vec<AnalysisJob>> {
        let project = project.map(|p| self.project(p).map(|p| p.id)).transpose()?;
        let conn = self.db.connect()?;
        let mut stmt=conn.prepare("SELECT id,project_id,target_id,file_id,kind,status,created_at,finished_at,error_code,logs_json FROM analysis_jobs WHERE ?1 IS NULL OR project_id=?1 ORDER BY created_at DESC,id").map_err(db_error)?;
        let rows = stmt
            .query_map([project], |r| {
                Ok((
                    AnalysisJob {
                        id: r.get(0)?,
                        project_id: r.get(1)?,
                        target_id: r.get(2)?,
                        file_id: r.get(3)?,
                        kind: r.get(4)?,
                        status: r.get(5)?,
                        created_at: r.get(6)?,
                        finished_at: r.get(7)?,
                        error_code: r.get(8)?,
                        logs: vec![],
                    },
                    r.get::<_, String>(9)?,
                ))
            })
            .map_err(db_error)?;
        rows.map(|r| {
            let (mut job, json) = r.map_err(db_error)?;
            job.logs = serde_json::from_str(&json).map_err(json_error)?;
            Ok(job)
        })
        .collect()
    }
    pub fn job(&self, id: &str) -> Result<AnalysisJob> {
        self.list_jobs(None)?
            .into_iter()
            .find(|j| j.id == id)
            .ok_or_else(|| AppError::new("NOT_FOUND", "Job does not exist"))
    }
    pub fn cancel_job(&self, id: &str) -> Result<()> {
        self.job(id)?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let changed=tx.execute("UPDATE analysis_jobs SET status='cancelled',finished_at=?1 WHERE id=?2 AND status='running'",params![now(),id]).map_err(db_error)?;
        if changed == 0 {
            return Err(AppError::new(
                "JOB_FINISHED",
                "Only running jobs can be cancelled",
            ));
        }
        Self::audit(&tx, "analysis.cancelled", id, "cancelled")?;
        tx.commit().map_err(db_error)
    }
    pub fn compare_files(&self, a: &str, b: &str) -> Result<BinaryComparison> {
        let a = self.imported_file(a)?;
        let b = self.imported_file(b)?;
        let bytes_a = self.sample_bytes(&a)?;
        let bytes_b = self.sample_bytes(&b)?;
        let analysis_a = inspect(&bytes_a)?;
        let analysis_b = inspect(&bytes_b)?;
        let sa: Vec<_> = analysis_a.sections.iter().map(|s| s.name.clone()).collect();
        let sb: Vec<_> = analysis_b.sections.iter().map(|s| s.name.clone()).collect();
        Ok(BinaryComparison {
            identical: a.sha256 == b.sha256,
            a_sha256: a.sha256,
            b_sha256: b.sha256,
            a_size: a.size,
            b_size: b.size,
            changed_bytes: bytes_a.iter().zip(&bytes_b).filter(|(a, b)| a != b).count() as u64
                + a.size.abs_diff(b.size),
            sections_added: sb.iter().filter(|s| !sa.contains(s)).cloned().collect(),
            sections_removed: sa.iter().filter(|s| !sb.contains(s)).cloned().collect(),
            imports_added: analysis_b
                .imports
                .iter()
                .filter(|s| !analysis_a.imports.contains(s))
                .cloned()
                .collect(),
            imports_removed: analysis_a
                .imports
                .iter()
                .filter(|s| !analysis_b.imports.contains(s))
                .cloned()
                .collect(),
        })
    }
}
