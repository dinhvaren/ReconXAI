use chrono::Utc;
use reconx_security::{AppError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};
mod local;
mod recon;
pub use recon::{ReconOutput, ReconToolRequest};
pub mod packages;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub category: String,
    pub platform: String,
    pub version: String,
    pub source: String,
    pub url: String,
    pub sha256: String,
    pub binary_sha256: String,
    pub archive: String,
    pub entry: String,
    pub executable: String,
    pub requires_admin: bool,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub version: String,
    pub installed_at: String,
    pub last_health_check: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Inventory {
    pub manifest: ToolManifest,
    pub status: String,
    pub integrity_verified: bool,
    pub receipt: Option<Receipt>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ToolProgress {
    pub phase: String,
    pub downloaded: u64,
    pub total: Option<u64>,
}
#[derive(Clone)]
pub struct ToolManager {
    root: PathBuf,
}
fn io_error() -> AppError {
    AppError::new(
        "TOOL_IO",
        "Managed tool operation failed; check workspace permissions",
    )
}
pub fn validate_manifest(m: &ToolManifest) -> Result<()> {
    let safe = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-._".contains(&c))
            && s != "."
            && s != ".."
    };
    if !safe(&m.id)
        || !safe(&m.version)
        || !safe(&m.executable)
        || !m.url.starts_with("https://github.com/")
        || m.requires_admin
        || !["zip", "tar_gz"].contains(&m.archive.as_str())
        || [&m.sha256, &m.binary_sha256]
            .iter()
            .any(|h| h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(AppError::new(
            "INVALID_MANIFEST",
            "Manifest paths, source or integrity fields are invalid",
        ));
    }
    Ok(())
}
pub fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn verify(bytes: &[u8], expected: &str) -> Result<()> {
    if expected.len() != 64
        || !expected.chars().all(|c| c.is_ascii_hexdigit())
        || hash(bytes) != expected.to_ascii_lowercase()
    {
        return Err(AppError::new(
            "INTEGRITY_FAILURE",
            "SHA-256 verification failed; nothing will be executed",
        ));
    }
    Ok(())
}
fn manifest(id: &str) -> Result<ToolManifest> {
    let manifests: Vec<ToolManifest> = serde_json::from_str(match id {
        "ripgrep" => include_str!("../../../manifests/tools/ripgrep.json"),
        "yara-x" => include_str!("../../../manifests/tools/yara-x.json"),
        "httpx" => include_str!("../../../manifests/tools/httpx.json"),
        "subfinder" => include_str!("../../../manifests/tools/subfinder.json"),
        _ => {
            return Err(AppError::new(
                "UNKNOWN_TOOL",
                "No verified adapter for this tool",
            ));
        }
    })
    .map_err(|_| AppError::new("INVALID_MANIFEST", "Embedded tool manifest is invalid"))?;
    let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    manifests
        .into_iter()
        .find(|m| m.platform == platform)
        .ok_or_else(|| {
            AppError::new(
                "UNSUPPORTED_PLATFORM",
                "No pinned tool artifact exists for this platform",
            )
        })
}
impl ToolManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn plan(&self, id: &str) -> Result<ToolManifest> {
        let m = manifest(id)?;
        validate_manifest(&m)?;
        Ok(m)
    }
    fn directory(&self, m: &ToolManifest) -> PathBuf {
        self.root.join(&m.id).join(&m.version)
    }
    fn validate_directory(&self, directory: &Path) -> Result<()> {
        let root = std::fs::canonicalize(&self.root).map_err(|_| io_error())?;
        let resolved = std::fs::canonicalize(directory).map_err(|_| io_error())?;
        if !resolved.starts_with(&root)
            || std::fs::symlink_metadata(directory)
                .map_err(|_| io_error())?
                .file_type()
                .is_symlink()
        {
            return Err(AppError::new(
                "TOOL_PATH_POLICY",
                "Managed version directory resolves outside the tool workspace or is a symbolic link",
            ));
        }
        Ok(())
    }
    fn receipt(&self, m: &ToolManifest) -> Option<Receipt> {
        let bytes = std::fs::read(self.directory(m).join("receipt.json")).ok()?;
        if bytes.len() > 8192 {
            return None;
        }
        serde_json::from_slice(&bytes).ok()
    }
    fn verify_installed(&self, m: &ToolManifest) -> Result<PathBuf> {
        self.validate_directory(&self.directory(m))?;
        let path = self.directory(m).join(&m.executable);
        let meta = std::fs::symlink_metadata(&path)
            .map_err(|_| AppError::new("TOOL_NOT_INSTALLED", "Managed tool is not installed"))?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 128 * 1024 * 1024 {
            return Err(AppError::new(
                "INTEGRITY_FAILURE",
                "Managed executable is not a regular bounded file",
            ));
        }
        verify(
            &std::fs::read(&path).map_err(|_| io_error())?,
            &m.binary_sha256,
        )?;
        std::fs::canonicalize(path).map_err(|_| io_error())
    }
    pub fn inventory(&self) -> Result<Vec<Inventory>> {
        ["ripgrep", "yara-x", "httpx", "subfinder"]
            .iter()
            .map(|id| self.inventory_one(id))
            .collect()
    }
    fn inventory_one(&self, id: &str) -> Result<Inventory> {
        let m = self.plan(id)?;
        let receipt = self.receipt(&m);
        let installed = self.directory(&m).join(&m.executable).exists();
        let integrity_verified = installed && self.verify_installed(&m).is_ok();
        let status = if !installed {
            "not_installed"
        } else if !integrity_verified {
            "integrity_failed"
        } else if receipt
            .as_ref()
            .is_some_and(|r| r.last_health_check.is_some())
        {
            "tested"
        } else {
            "installed"
        };
        Ok(Inventory {
            manifest: m,
            status: status.into(),
            integrity_verified,
            receipt,
        })
    }
    pub async fn install<F>(&self, id: &str, mut progress: F) -> Result<Inventory>
    where
        F: FnMut(ToolProgress) + Send,
    {
        let m = self.plan(id)?;
        if self.directory(&m).join(&m.executable).exists() {
            return Err(AppError::new(
                "ALREADY_INSTALLED",
                "Managed executable already exists; run tools doctor or uninstall it first",
            ));
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                let u = attempt.url();
                if attempt.previous().len() < 5
                    && u.scheme() == "https"
                    && matches!(
                        u.host_str(),
                        Some(
                            "github.com"
                                | "release-assets.githubusercontent.com"
                                | "objects.githubusercontent.com"
                        )
                    )
                {
                    attempt.follow()
                } else {
                    attempt.stop()
                }
            }))
            .build()
            .map_err(|_| AppError::new("TOOL_DOWNLOAD", "Cannot initialize download client"))?;
        let mut response = client
            .get(&m.url)
            .send()
            .await
            .map_err(|_| AppError::new("TOOL_DOWNLOAD", "Official tool download failed"))?;
        if !response.status().is_success() {
            return Err(AppError::new(
                "TOOL_DOWNLOAD",
                "Official download returned an unsuccessful status",
            ));
        }
        let total = response.content_length();
        let mut bytes = Vec::new();
        progress(ToolProgress {
            phase: "downloading".into(),
            downloaded: 0,
            total,
        });
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| AppError::new("TOOL_DOWNLOAD", "Official download was interrupted"))?
        {
            if bytes.len() + chunk.len() > 32 * 1024 * 1024 {
                return Err(AppError::new(
                    "TOOL_DOWNLOAD_LIMIT",
                    "Tool archive exceeded 32 MiB",
                ));
            }
            bytes.extend_from_slice(&chunk);
            progress(ToolProgress {
                phase: "downloading".into(),
                downloaded: bytes.len() as u64,
                total,
            });
        }
        verify(&bytes, &m.sha256)?;
        progress(ToolProgress {
            phase: "verified".into(),
            downloaded: bytes.len() as u64,
            total,
        });
        let binary = extract(&m, &bytes)?;
        verify(&binary, &m.binary_sha256)?;
        let dir = self.directory(&m);
        std::fs::create_dir_all(&dir).map_err(|_| io_error())?;
        self.validate_directory(&dir)?;
        let mut temp = tempfile::NamedTempFile::new_in(&dir).map_err(|_| io_error())?;
        temp.write_all(&binary).map_err(|_| io_error())?;
        temp.as_file().sync_all().map_err(|_| io_error())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            temp.as_file()
                .set_permissions(std::fs::Permissions::from_mode(0o700))
                .map_err(|_| io_error())?;
        }
        temp.persist_noclobber(dir.join(&m.executable))
            .map_err(|_| io_error())?;
        let receipt = Receipt {
            id: m.id.clone(),
            version: m.version.clone(),
            installed_at: Utc::now().to_rfc3339(),
            last_health_check: None,
        };
        self.write_receipt(&m, &receipt)?;
        progress(ToolProgress {
            phase: "installed".into(),
            downloaded: bytes.len() as u64,
            total,
        });
        self.inventory_one(id)
    }
    fn write_receipt(&self, m: &ToolManifest, receipt: &Receipt) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(receipt).map_err(|_| io_error())?;
        let mut temp =
            tempfile::NamedTempFile::new_in(self.directory(m)).map_err(|_| io_error())?;
        temp.write_all(&bytes).map_err(|_| io_error())?;
        temp.as_file().sync_all().map_err(|_| io_error())?;
        temp.persist(self.directory(m).join("receipt.json"))
            .map_err(|_| io_error())?;
        Ok(())
    }
    pub async fn doctor(&self, id: &str) -> Result<Inventory> {
        let m = self.plan(id)?;
        let executable = self.verify_installed(&m)?;
        let mut command = tokio::process::Command::new(executable);
        // Version commands can initialize upstream configuration too. Keep
        // their cwd/home private just like a real recon invocation.
        let version_home = tempfile::tempdir().map_err(|_| io_error())?;
        command.current_dir(version_home.path());
        command.env_clear();
        for key in [
            "HOME",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
        ] {
            command.env(key, version_home.path());
        }
        for key in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
            if let Some(v) = std::env::var_os(key) {
                command.env(key, v);
            }
        }
        command
            .arg(if matches!(id, "httpx" | "subfinder") {
                "-version"
            } else {
                "--version"
            })
            .kill_on_drop(true);
        #[cfg(windows)]
        {
            command.creation_flags(0x08000000);
        }
        let output = tokio::time::timeout(Duration::from_secs(5), command.output())
            .await
            .map_err(|_| AppError::new("TOOL_TIMEOUT", "Tool version check timed out"))?
            .map_err(|_| AppError::new("TOOL_EXECUTION", "Cannot execute verified tool"))?;
        let expected = format!(
            "{} {}",
            if id == "yara-x" {
                "yara-x-cli"
            } else {
                "ripgrep"
            },
            m.version
        );
        let pd_version = matches!(id, "httpx" | "subfinder")
            && String::from_utf8_lossy(&output.stderr)
                .contains(&format!("Current Version: v{}", m.version));
        if !output.status.success()
            || output.stdout.len() > 8192
            || output.stderr.len() > 8192
            || (!pd_version && !String::from_utf8_lossy(&output.stdout).starts_with(&expected))
        {
            return Err(AppError::new(
                "TOOL_HEALTH_FAILURE",
                "Verified tool did not report the expected version",
            ));
        }
        if matches!(id, "httpx" | "subfinder") {
            let manager = self.clone();
            let tool = id.to_string();
            tokio::task::spawn_blocking(move || manager.recon_health(&tool))
                .await
                .map_err(|_| {
                    AppError::new("TOOL_HEALTH_FAILURE", "Recon health worker failed")
                })??;
        }
        let mut receipt = self
            .receipt(&m)
            .ok_or_else(|| AppError::new("TOOL_NOT_MANAGED", "Installation receipt is missing"))?;
        receipt.last_health_check = Some(Utc::now().to_rfc3339());
        self.write_receipt(&m, &receipt)?;
        self.inventory_one(id)
    }
    pub fn uninstall(&self, id: &str) -> Result<()> {
        let m = self.plan(id)?;
        let dir = self.directory(&m);
        if dir.exists() {
            self.validate_directory(&dir)?;
        }
        // Remove only explicitly named managed files, never recurse into user data.
        for filename in [&m.executable, "receipt.json"] {
            let path = dir.join(filename);
            if path.exists() {
                std::fs::remove_file(path).map_err(|_| io_error())?;
            }
        }
        if dir.exists() {
            std::fs::remove_dir(dir).map_err(|_|AppError::new("TOOL_DIRECTORY_NOT_EMPTY","Managed files were removed; unexpected files in the version directory were preserved"))?;
        }
        Ok(())
    }
}
fn bounded_read(mut reader: impl Read) -> Result<Vec<u8>> {
    let mut binary = Vec::new();
    reader
        .by_ref()
        .take(128 * 1024 * 1024 + 1)
        .read_to_end(&mut binary)
        .map_err(|_| AppError::new("TOOL_ARCHIVE", "Cannot read tool archive entry"))?;
    if binary.len() > 128 * 1024 * 1024 {
        return Err(AppError::new(
            "TOOL_ARCHIVE",
            "Executable exceeds size limit",
        ));
    }
    Ok(binary)
}
fn extract(m: &ToolManifest, bytes: &[u8]) -> Result<Vec<u8>> {
    // Never unpack paths supplied by an archive: copy only the pinned regular entry.
    if m.archive == "zip" {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .map_err(|_| AppError::new("TOOL_ARCHIVE", "Invalid ZIP archive"))?;
        let file = archive
            .by_name(&m.entry)
            .map_err(|_| AppError::new("TOOL_ARCHIVE", "Pinned executable is absent"))?;
        if !file.is_file()
            || file
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(AppError::new(
                "TOOL_ARCHIVE",
                "Pinned entry is not a regular file",
            ));
        }
        bounded_read(file)
    } else if m.archive == "tar_gz" {
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
        for entry in archive
            .entries()
            .map_err(|_| AppError::new("TOOL_ARCHIVE", "Invalid TAR archive"))?
        {
            let entry = entry.map_err(|_| AppError::new("TOOL_ARCHIVE", "Invalid TAR entry"))?;
            if entry
                .path()
                .map_err(|_| AppError::new("TOOL_ARCHIVE", "Invalid archive path"))?
                == Path::new(&m.entry)
                && entry.header().entry_type().is_file()
            {
                return bounded_read(entry);
            }
        }
        Err(AppError::new("TOOL_ARCHIVE", "Pinned executable is absent"))
    } else {
        Err(AppError::new("TOOL_ARCHIVE", "Unsupported archive type"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yara_manifests_are_pinned_for_both_platforms_and_reject_unsafe_fields() {
        let manifests: Vec<ToolManifest> =
            serde_json::from_str(include_str!("../../../manifests/tools/yara-x.json")).unwrap();
        assert_eq!(manifests.len(), 2);
        for m in manifests {
            validate_manifest(&m).unwrap();
            assert_eq!(m.version, "1.21.0");
            assert!(m.dependencies.is_empty());
            let mut corrupt = m.clone();
            corrupt.id = "../outside".into();
            assert!(validate_manifest(&corrupt).is_err());
            corrupt = m.clone();
            corrupt.url = "http://untrusted.invalid/tool".into();
            assert!(validate_manifest(&corrupt).is_err());
            corrupt = m;
            corrupt.sha256 = "fabricated".into();
            assert!(validate_manifest(&corrupt).is_err());
        }
    }
    #[test]
    fn integrity_rejects_modified_downloads() {
        let digest = hash(b"trusted");
        verify(b"trusted", &digest).unwrap();
        assert_eq!(
            verify(b"modified", &digest).unwrap_err().code,
            "INTEGRITY_FAILURE"
        );
        assert!(verify(b"trusted", "fake").is_err());
    }
    #[test]
    fn manifests_are_pinned_and_unknown_tools_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(dir.path().into());
        let m = manager.plan("ripgrep").unwrap();
        assert_eq!(m.version, "15.2.0");
        assert_eq!(m.sha256.len(), 64);
        assert_eq!(m.binary_sha256.len(), 64);
        assert!(!m.requires_admin);
        assert!(manager.plan("../other").is_err());
    }
    #[test]
    fn uninstaller_preserves_unexpected_files() {
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(temp.path().into());
        let m = manager.plan("ripgrep").unwrap();
        let dir = manager.directory(&m);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("user-notes.txt"), b"preserve").unwrap();
        assert!(manager.uninstall("ripgrep").is_err());
        assert_eq!(
            std::fs::read(dir.join("user-notes.txt")).unwrap(),
            b"preserve"
        );
    }
}
