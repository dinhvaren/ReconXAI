//! Ingestion preserves complete bytes; deep-parser memory budgets are independent.
use super::*;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileImportLimits {
    pub max_bytes: u64,
}
impl Default for FileImportLimits {
    fn default() -> Self {
        Self {
            max_bytes: 8 * 1024 * 1024 * 1024,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct FileImportProgress {
    pub stage: String,
    pub completed: u64,
    pub total: u64,
}
impl Core {
    pub fn import_file_with_progress(
        &self,
        project: &str,
        path: &Path,
        authorized: bool,
        limits: FileImportLimits,
        mut update: impl FnMut(FileImportProgress) -> Result<()>,
    ) -> Result<ImportedFile> {
        if limits.max_bytes == 0 || limits.max_bytes > 64 * 1024 * 1024 * 1024 {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "Choose an import budget between 1 byte and 64 GiB",
            ));
        }
        let project = self.project(project)?;
        if project.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project before importing files",
            ));
        }
        let metadata = fs::symlink_metadata(path).map_err(|_| ingest_io())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(AppError::new(
                "INVALID_FILE",
                "Select a regular local file, not a directory or symbolic link",
            ));
        }
        if metadata.len() > limits.max_bytes {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "File exceeds the selected streaming import budget",
            ));
        }
        let canonical = path.canonicalize().map_err(|_| ingest_io())?;
        let mut source = File::open(&canonical).map_err(|_| ingest_io())?;
        let initial = source.metadata().map_err(|_| ingest_io())?;
        if !initial.is_file() || initial.len() != metadata.len() {
            return Err(AppError::new(
                "FILE_CHANGED",
                "Selected file changed before import",
            ));
        }
        let directory = self.artifact_directory()?;
        let reserve = initial
            .len()
            .checked_add(16 * 1024 * 1024)
            .ok_or_else(ingest_io)?;
        if reconx_tools::packages::free_space(&directory)? < reserve {
            return Err(AppError::new(
                "DISK_SPACE",
                "Insufficient free disk space for the complete artifact and safety reserve",
            ));
        }
        let mut staging = tempfile::Builder::new()
            .prefix(".import-")
            .tempfile_in(&directory)
            .map_err(|_| ingest_io())?;
        let prefix_budget = if initial.len() <= reconx_analysis::MAX_FILE_BYTES as u64 {
            reconx_analysis::MAX_FILE_BYTES
        } else {
            1024 * 1024
        };
        let mut prefix = Vec::with_capacity((initial.len().min(prefix_budget as u64)) as usize);
        let mut chunk = vec![0u8; 256 * 1024];
        let mut digest = Sha256::new();
        let mut count = 0u64;
        update(FileImportProgress {
            stage: "Copying and hashing selected file".into(),
            completed: 0,
            total: initial.len(),
        })?;
        loop {
            let n = source.read(&mut chunk).map_err(|_| ingest_io())?;
            if n == 0 {
                break;
            }
            count = count.checked_add(n as u64).ok_or_else(ingest_io)?;
            if count > limits.max_bytes || count > initial.len() {
                return Err(AppError::new(
                    "FILE_CHANGED",
                    "File grew beyond its approved import size",
                ));
            }
            staging.write_all(&chunk[..n]).map_err(|_| ingest_io())?;
            digest.update(&chunk[..n]);
            let keep = n.min(prefix_budget.saturating_sub(prefix.len()));
            prefix.extend_from_slice(&chunk[..keep]);
            update(FileImportProgress {
                stage: "Copying and hashing selected file".into(),
                completed: count,
                total: initial.len(),
            })?;
        }
        let after = source.metadata().map_err(|_| ingest_io())?;
        if count != initial.len()
            || after.len() != initial.len()
            || after.modified().ok() != initial.modified().ok()
        {
            return Err(AppError::new(
                "FILE_CHANGED",
                "File changed during ingestion; no imported record was committed",
            ));
        }
        let sha256: String = digest
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        update(FileImportProgress {
            stage: "Inspecting bounded content signatures".into(),
            completed: count,
            total: count,
        })?;
        let format = if count <= reconx_analysis::MAX_FILE_BYTES as u64 {
            reconx_analysis::inspect_with_progress(&prefix, |_, _| {
                update(FileImportProgress {
                    stage: "Inspecting bounded file structure".into(),
                    completed: count,
                    total: count,
                })
            })?
            .format
        } else {
            reconx_analysis::signature_label(&prefix)
        };
        staging.as_file_mut().sync_all().map_err(|_| ingest_io())?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if self.project(&project.id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Project was archived during ingestion",
            ));
        }
        update(FileImportProgress {
            stage: "Committing complete artifact".into(),
            completed: count,
            total: count,
        })?;
        let canonical = canonical.to_string_lossy().into_owned();
        let target = if let Some(t) = self
            .list_targets(Some(&project.id))?
            .into_iter()
            .find(|t| t.input.kind.local_file() && t.input.value == canonical)
        {
            t
        } else {
            let input = self.validate_target_input(TargetInput {
                project_id: project.id.clone(),
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .chars()
                    .take(120)
                    .collect(),
                kind: if format == "Android APK" {
                    TargetKind::Apk
                } else {
                    TargetKind::Binary
                },
                value: canonical.clone(),
                description: "Local file registered through bounded streaming import".into(),
                tags: vec!["local-import".into()],
                allowed_scope: vec![],
                excluded_scope: vec![],
                authorized,
                mode: SecurityMode::PassiveAuto,
            })?;
            let target = Target {
                id: Uuid::new_v4().to_string(),
                scope_status: if authorized {
                    "allowed"
                } else {
                    "authorization_required"
                }
                .into(),
                input,
                created_at: now(),
                updated_at: now(),
                last_analyzed_at: None,
            };
            Self::write_target(&tx, &target)?;
            target
        };
        let file = ImportedFile {
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
            source_path: canonical.clone(),
            sha256,
            size: count,
            format,
            created_at: now(),
        };
        let destination = self.blob_path(&file.sha256)?;
        match staging.persist_noclobber(&destination) {
            Ok(_) => {}
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                let actual = self.stream_sample_facts(&file, &mut |done, total| {
                    update(FileImportProgress {
                        stage: "Verifying existing complete artifact".into(),
                        completed: done,
                        total,
                    })
                })?;
                if actual.sha256 != file.sha256 {
                    return Err(AppError::new(
                        "INTEGRITY_ERROR",
                        "Existing managed artifact does not match its content hash",
                    ));
                }
            }
            Err(_) => return Err(ingest_io()),
        }
        tx.execute("INSERT INTO imported_files(id,project_id,target_id,name,sha256,size,format,created_at,source_value) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![file.id,file.project_id,file.target_id,file.name,file.sha256,file.size as i64,file.format,file.created_at,canonical]).map_err(db_error)?;
        Self::audit(&tx, "file.imported", &file.id, "completed")?;
        tx.commit().map_err(db_error)?;
        Ok(file)
    }
    pub(super) fn stream_sample_facts(
        &self,
        file: &ImportedFile,
        update: &mut dyn FnMut(u64, u64) -> Result<()>,
    ) -> Result<BinaryAnalysis> {
        let path = self.blob_path(&file.sha256)?;
        let meta = fs::symlink_metadata(&path).map_err(|_| ingest_io())?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() != file.size {
            return Err(AppError::new(
                "INTEGRITY_ERROR",
                "Managed file identity or size changed",
            ));
        }
        let mut input = File::open(path).map_err(|_| ingest_io())?;
        let mut hash = Sha256::new();
        let mut prefix = Vec::new();
        let mut counts = [0u64; 256];
        let mut done = 0u64;
        let mut chunk = vec![0; 256 * 1024];
        let mut last = std::time::Instant::now();
        let mut reported = 0u64;
        update(0, file.size)?;
        loop {
            let n = input.read(&mut chunk).map_err(|_| ingest_io())?;
            if n == 0 {
                break;
            }
            done += n as u64;
            if done > file.size {
                return Err(AppError::new(
                    "INTEGRITY_ERROR",
                    "Managed artifact grew during verification",
                ));
            }
            hash.update(&chunk[..n]);
            for b in &chunk[..n] {
                counts[*b as usize] += 1;
            }
            let keep = n.min((1024 * 1024usize).saturating_sub(prefix.len()));
            prefix.extend_from_slice(&chunk[..keep]);
            if done == file.size
                || done - reported >= file.size.div_ceil(100).max(256 * 1024)
                || last.elapsed() >= std::time::Duration::from_millis(50)
            {
                update(done, file.size)?;
                last = std::time::Instant::now();
                reported = done;
            }
        }
        let actual: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if done != file.size || actual != file.sha256 {
            return Err(AppError::new(
                "INTEGRITY_ERROR",
                "Complete artifact hash verification failed",
            ));
        }
        let entropy = counts
            .iter()
            .filter(|n| **n > 0)
            .map(|n| {
                let p = *n as f64 / done.max(1) as f64;
                -p * p.log2()
            })
            .sum();
        reconx_analysis::inspect_metadata_prefix(&prefix, file.size, &actual, entropy)
    }
}
fn ingest_io() -> AppError {
    AppError::new(
        "FILE_IO",
        "Cannot access the selected local file or managed artifact",
    )
}
