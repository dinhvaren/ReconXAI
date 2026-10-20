//! Reviewed npm tarballs, not package-generated commands. No npm lifecycle scripts.
use super::*;
use base64::Engine;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::process::{Command, Stdio};
const NODE_VERSION: &str = "24.22.0";
const MAX_DOWNLOAD: u64 = 128 * 1024 * 1024;
const MAX_EXTRACT: u64 = 512 * 1024 * 1024;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDependency {
    pub name: String,
    pub version: String,
    pub executable: String,
    pub sha256: String,
    pub managed: bool,
    pub source: Option<String>,
    pub archive_sha256: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageDependency {
    pub path: String,
    pub version: String,
    pub source: String,
    pub integrity: String,
    pub dependencies: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpmPlan {
    pub id: String,
    pub version: String,
    pub lock_sha256: String,
    pub runtime: RuntimeDependency,
    pub dependencies: Vec<PackageDependency>,
    pub minimum_free_bytes: u64,
    pub os: String,
    pub architecture: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageReceipt {
    pub version: String,
    pub lock_sha256: String,
    pub runtime: RuntimeDependency,
    pub directory: String,
    pub files: BTreeMap<String, String>,
}
#[derive(Clone)]
pub struct PackageManager {
    root: PathBuf,
}
fn error(code: &str, message: &str) -> AppError {
    AppError::new(code, message)
}
fn redirected(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| {
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
fn protect_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|_| io_error())?;
    if redirected(path) {
        return Err(error("PACKAGE_PATH", "Managed directory cannot be a link"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| io_error())?;
    }
    Ok(())
}
fn safe_relative(path: &str) -> Result<PathBuf> {
    if path.is_empty()
        || path.len() > 1024
        || path.split('/').count() > 32
        || path.contains(['\\', ':', '\0'])
    {
        return Err(error("PACKAGE_PATH", "Unsafe archive or dependency path"));
    }
    let mut result = PathBuf::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." || part.ends_with(['.', ' ']) {
            return Err(error(
                "PACKAGE_PATH",
                "Archive traversal or ambiguous path refused",
            ));
        }
        if [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ]
        .contains(
            &part
                .split('.')
                .next()
                .unwrap_or("")
                .to_ascii_uppercase()
                .as_str(),
        ) {
            return Err(error("PACKAGE_PATH", "Reserved device path refused"));
        }
        result.push(part);
    }
    Ok(result)
}
fn lock(version: &str) -> Result<Value> {
    let source = match version {
        "2.0.0" => include_str!("../../../manifests/mcp/everything-2.0.0.lock.json"),
        "1.0.0" => include_str!("../../../manifests/mcp/everything-1.0.0.lock.json"),
        _ => {
            return Err(error(
                "PACKAGE_VERSION",
                "This package version has no reviewed lockfile",
            ));
        }
    };
    serde_json::from_str(source)
        .map_err(|_| error("PACKAGE_MANIFEST", "Reviewed package lock is invalid"))
}
fn sri(bytes: &[u8], integrity: &str) -> Result<()> {
    use sha2::Sha512;
    let expected = integrity
        .strip_prefix("sha512-")
        .and_then(|s| base64::engine::general_purpose::STANDARD.decode(s).ok())
        .filter(|b| b.len() == 64)
        .ok_or_else(|| {
            error(
                "PACKAGE_INTEGRITY",
                "Only pinned SHA-512 npm integrity is accepted",
            )
        })?;
    if Sha512::digest(bytes).as_slice() != expected {
        return Err(error(
            "PACKAGE_INTEGRITY",
            "Package archive integrity verification failed",
        ));
    }
    Ok(())
}
fn graph(version: &str) -> Result<(String, Vec<PackageDependency>)> {
    let lock = lock(version)?;
    graph_value(lock)
}
fn graph_value(lock: Value) -> Result<(String, Vec<PackageDependency>)> {
    let packages = lock["packages"].as_object().ok_or_else(io_error)?;
    let mut result = Vec::new();
    for (path, package) in packages.iter().filter(|(p, _)| !p.is_empty()) {
        safe_relative(path)?;
        if !path.starts_with("node_modules/")
            || package["hasInstallScript"] == true
            || package.get("link") == Some(&json!(true))
        {
            return Err(error(
                "PACKAGE_POLICY",
                "Scripts, linked and non-registry packages require manual setup",
            ));
        }
        let source = package["resolved"].as_str().ok_or_else(io_error)?;
        let integrity = package["integrity"].as_str().ok_or_else(io_error)?;
        let url = reqwest::Url::parse(source).map_err(|_| io_error())?;
        if url.scheme() != "https"
            || url.host_str() != Some("registry.npmjs.org")
            || url.port().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(error(
                "PACKAGE_SOURCE",
                "Only the reviewed official npm HTTPS registry is permitted",
            ));
        }
        if !integrity.starts_with("sha512-") {
            return Err(error(
                "PACKAGE_INTEGRITY",
                "Every transitive dependency requires SHA-512 integrity",
            ));
        }
        let mut dependencies = Vec::new();
        if let Some(deps) = package["dependencies"].as_object() {
            for name in deps.keys() {
                safe_relative(name)?;
                let mut current = path.as_str();
                let resolved = loop {
                    let candidate = format!("{current}/node_modules/{name}");
                    if packages.contains_key(&candidate) {
                        break Some(candidate);
                    }
                    match current.rsplit_once("/node_modules/") {
                        Some((parent, _)) => current = parent,
                        None => {
                            let candidate = format!("node_modules/{name}");
                            break packages.contains_key(&candidate).then_some(candidate);
                        }
                    }
                };
                dependencies.push(resolved.ok_or_else(|| {
                    error(
                        "DEPENDENCY_MISSING",
                        "A reviewed transitive dependency cannot be resolved",
                    )
                })?);
            }
        }
        result.push(PackageDependency {
            path: path.clone(),
            version: package["version"].as_str().ok_or_else(io_error)?.into(),
            source: source.into(),
            integrity: integrity.into(),
            dependencies,
        });
    }
    if result.len() > 256 {
        return Err(error(
            "DEPENDENCY_LIMIT",
            "Dependency graph exceeds the reviewed package limit",
        ));
    }
    fn visit<'a>(
        id: &'a str,
        graph: &'a [PackageDependency],
        active: &mut BTreeSet<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> Result<()> {
        if done.contains(id) {
            return Ok(());
        }
        if !active.insert(id) {
            return Err(error(
                "DEPENDENCY_CYCLE",
                "Circular dependency metadata is unsupported by this adapter",
            ));
        }
        let node = graph.iter().find(|n| n.path == id).ok_or_else(io_error)?;
        for edge in &node.dependencies {
            visit(edge, graph, active, done)?;
        }
        active.remove(id);
        done.insert(id);
        Ok(())
    }
    let mut done = BTreeSet::new();
    for node in &result {
        visit(&node.path, &result, &mut BTreeSet::new(), &mut done)?;
    }
    Ok((
        hash(&serde_json::to_vec(&lock).map_err(|_| io_error())?),
        result,
    ))
}
fn file_hash(path: &Path) -> Result<String> {
    let meta = std::fs::symlink_metadata(path).map_err(|_| io_error())?;
    if !meta.is_file() || redirected(path) || meta.len() > MAX_DOWNLOAD {
        return Err(error(
            "PACKAGE_PATH",
            "Expected a bounded regular managed file",
        ));
    }
    Ok(hash(&std::fs::read(path).map_err(|_| io_error())?))
}
fn node_version(path: &Path) -> Result<String> {
    let mut command = Command::new(path);
    command
        .arg("--version")
        .env_clear()
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped());
    for key in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|_| {
        error(
            "RUNTIME_UNAVAILABLE",
            "Cannot execute the detected Node runtime",
        )
    })?;
    let started = std::time::Instant::now();
    loop {
        match child.try_wait().map_err(|_| io_error())? {
            Some(status) => {
                if !status.success() {
                    return Err(error("RUNTIME_UNAVAILABLE", "Node version check failed"));
                }
                let mut value = String::new();
                child
                    .stdout
                    .take()
                    .ok_or_else(io_error)?
                    .take(8193)
                    .read_to_string(&mut value)
                    .map_err(|_| io_error())?;
                if value.len() > 8192 {
                    return Err(error(
                        "RUNTIME_LIMIT",
                        "Runtime version output exceeds limit",
                    ));
                }
                return Ok(value.trim().trim_start_matches('v').into());
            }
            None if started.elapsed() < Duration::from_secs(3) => {
                std::thread::sleep(Duration::from_millis(10))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error("RUNTIME_TIMEOUT", "Node version check timed out"));
            }
        }
    }
}
fn compatible_node(version: &str) -> bool {
    version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .is_some_and(|v| v >= 24)
}
pub fn free_space(path: &Path) -> Result<u64> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let name = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let mut available = 0u64;
        // SAFETY: the path is NUL terminated, the output pointer is writable,
        // and unused optional outputs are null for this synchronous Windows API.
        if unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                name.as_ptr(),
                &mut available,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(io_error());
        }
        Ok(available)
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let name = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|_| io_error())?;
        let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: valid C path and writable statvfs storage; initialized only on success.
        if unsafe { libc::statvfs(name.as_ptr(), stats.as_mut_ptr()) } != 0 {
            return Err(io_error());
        }
        let stats = unsafe { stats.assume_init() };
        u64::try_from(u128::from(stats.f_bavail) * u128::from(stats.f_frsize))
            .map_err(|_| io_error())
    }
}
impl PackageManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn plan(&self, version: &str, managed: bool) -> Result<NpmPlan> {
        if std::env::consts::ARCH != "x86_64"
            || !["windows", "linux"].contains(&std::env::consts::OS)
        {
            return Err(error(
                "UNSUPPORTED_PLATFORM",
                "Reviewed npm/Node runtime currently supports Windows/Linux x64 only",
            ));
        }
        let (lock_sha256, dependencies) = graph(version)?;
        let existing = (!managed)
            .then(|| std::env::var_os("PATH"))
            .flatten()
            .and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|p| p.join(if cfg!(windows) { "node.exe" } else { "node" }))
                    .filter(|p| p.is_file() && !redirected(p))
                    .find_map(|p| {
                        let version = node_version(&p).ok()?;
                        if !compatible_node(&version) {
                            return None;
                        }
                        Some((std::fs::canonicalize(p).ok()?, version))
                    })
            });
        let runtime = if let Some((path, version)) = existing {
            RuntimeDependency {
                name: "node".into(),
                version,
                sha256: file_hash(&path)?,
                executable: path.to_string_lossy().into(),
                managed: false,
                source: None,
                archive_sha256: None,
            }
        } else {
            let (platform, archive, checksum, executable) = if cfg!(windows) {
                (
                    "win-x64",
                    "zip",
                    "f9683a19877e12a9b167305c8db71040d812876359de21f0d2e97c3f321f80c3",
                    "node.exe",
                )
            } else {
                (
                    "linux-x64",
                    "tar.gz",
                    "28ea0a438de9a13bd7891fd758eee6c7819ee8af5442dfd8d45bd65e56d4228c",
                    "bin/node",
                )
            };
            let executable = self
                .root
                .join("runtimes")
                .join(format!("node-{NODE_VERSION}-{platform}"))
                .join(executable);
            RuntimeDependency {
                name: "node".into(),
                version: NODE_VERSION.into(),
                executable: executable.to_string_lossy().into(),
                sha256: if cfg!(windows) {
                    "ba93f9a70bd86fce3ebbb39ec758501458453467aa92a34a0a3602d1e3fc2b9c"
                } else {
                    "67ca5783febb8bf85736c79fcdd92c8d6d465a0eb2a082672bcdfa76a506751c"
                }
                .into(),
                managed: true,
                source: Some(format!(
                    "https://nodejs.org/dist/v{NODE_VERSION}/node-v{NODE_VERSION}-{platform}.{archive}"
                )),
                archive_sha256: Some(checksum.into()),
            }
        };
        Ok(NpmPlan {
            id: "reconx.mcp-everything".into(),
            version: version.into(),
            lock_sha256,
            runtime,
            dependencies,
            minimum_free_bytes: MAX_EXTRACT,
            os: std::env::consts::OS.into(),
            architecture: std::env::consts::ARCH.into(),
        })
    }
    pub fn directory(&self, plan: &NpmPlan) -> PathBuf {
        self.root.join("packages").join(format!(
            "everything-{}-{}",
            plan.version,
            &plan.lock_sha256[..16]
        ))
    }
    pub async fn install(
        &self,
        plan: &NpmPlan,
        mut check: impl FnMut(&str, u64, Option<u64>) -> Result<()>,
    ) -> Result<PackageReceipt> {
        let (expected, dependencies) = graph(&plan.version)?;
        if plan.id != "reconx.mcp-everything"
            || plan.lock_sha256 != expected
            || serde_json::to_value(&dependencies).map_err(|_| io_error())?
                != serde_json::to_value(&plan.dependencies).map_err(|_| io_error())?
            || plan.os != std::env::consts::OS
            || plan.architecture != std::env::consts::ARCH
            || plan.minimum_free_bytes != MAX_EXTRACT
        {
            return Err(error(
                "PACKAGE_POLICY",
                "Package plan differs from the compiled reviewed lock",
            ));
        }
        protect_dir(&self.root)?;
        if free_space(&self.root)? < plan.minimum_free_bytes {
            return Err(error(
                "PACKAGE_DISK_SPACE",
                "At least 512 MiB free space is required before staging this package",
            ));
        }
        let runtime = if plan.runtime.managed {
            self.install_node(&plan.runtime, &mut check).await?
        } else {
            let path = Path::new(&plan.runtime.executable);
            if file_hash(path)? != plan.runtime.sha256 || !compatible_node(&node_version(path)?) {
                return Err(error(
                    "RUNTIME_CHANGED",
                    "Approved runtime changed or no longer satisfies Node >=24",
                ));
            }
            plan.runtime.clone()
        };
        let directory = self.directory(plan);
        let parent = directory.parent().ok_or_else(io_error)?;
        protect_dir(parent)?;
        if directory.exists() {
            let mut receipt = self.receipt(&directory)?;
            // A cached package tree may have been launched with another Node
            // runtime. Bind this operation to the exact newly approved runtime.
            receipt.runtime = runtime;
            self.verify(&receipt)?;
            let mut updated = tempfile::NamedTempFile::new_in(parent).map_err(|_| io_error())?;
            updated
                .write_all(&serde_json::to_vec(&receipt).map_err(|_| io_error())?)
                .map_err(|_| io_error())?;
            updated.as_file().sync_all().map_err(|_| io_error())?;
            updated
                .persist(directory.join(".reconx-receipt.json"))
                .map_err(|_| io_error())?;
            return Ok(receipt);
        }
        let stage = tempfile::Builder::new()
            .prefix(".package-staging-")
            .tempdir_in(parent)
            .map_err(|_| io_error())?;
        let mut total = 0;
        let mut file_count = 0usize;
        for dependency in &plan.dependencies {
            check(
                &format!("Downloading {} {}", dependency.path, dependency.version),
                0,
                None,
            )?;
            let archive = self
                .download(
                    &dependency.source,
                    Some(&dependency.integrity),
                    None,
                    &mut check,
                )
                .await?;
            let target = stage.path().join(safe_relative(&dependency.path)?);
            protect_dir(&target)?;
            extract_tar(
                &archive,
                &target,
                "package",
                false,
                &mut total,
                &mut |stage, bytes, size| {
                    file_count += 1;
                    if file_count > 50000 {
                        return Err(error(
                            "PACKAGE_LIMIT",
                            "Combined dependency file count exceeds limit",
                        ));
                    }
                    check(stage, bytes, size)
                },
            )?;
        }
        let mut files = BTreeMap::new();
        collect_files(stage.path(), stage.path(), &mut files)?;
        let receipt = PackageReceipt {
            version: plan.version.clone(),
            lock_sha256: plan.lock_sha256.clone(),
            runtime,
            directory: directory.to_string_lossy().into(),
            files,
        };
        std::fs::write(
            stage.path().join(".reconx-receipt.json"),
            serde_json::to_vec(&receipt).map_err(|_| io_error())?,
        )
        .map_err(|_| io_error())?;
        check("Committing verified package files", total, Some(total))?;
        std::fs::rename(stage.path(), &directory).map_err(|_| io_error())?;
        self.verify(&receipt)?;
        Ok(receipt)
    }
    async fn download(
        &self,
        source: &str,
        integrity: Option<&str>,
        sha256: Option<&str>,
        check: &mut impl FnMut(&str, u64, Option<u64>) -> Result<()>,
    ) -> Result<PathBuf> {
        let url = reqwest::Url::parse(source).map_err(|_| io_error())?;
        if url.scheme() != "https"
            || ![Some("registry.npmjs.org"), Some("nodejs.org")].contains(&url.host_str())
            || url.port().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(error(
                "PACKAGE_SOURCE",
                "Download URL is outside the approved HTTPS source policy",
            ));
        }
        let cache = self.root.join("download-cache");
        protect_dir(&cache)?;
        let path = cache.join(hash(
            format!("{source}|{integrity:?}|{sha256:?}").as_bytes(),
        ));
        let verify_archive = |bytes: &[u8]| -> Result<()> {
            if let Some(sri_hash) = integrity {
                sri(bytes, sri_hash)?;
            }
            if let Some(sha) = sha256 {
                verify(bytes, sha)?;
            }
            Ok(())
        };
        if path.exists() && (redirected(&path) || !path.is_file()) {
            return Err(error(
                "PACKAGE_PATH",
                "Download cache entry is not an owned regular file",
            ));
        }
        if path.is_file() {
            if std::fs::metadata(&path).map_err(|_| io_error())?.len() > MAX_DOWNLOAD {
                return Err(error(
                    "PACKAGE_LIMIT",
                    "Cached archive exceeds download limit",
                ));
            }
            let bytes = std::fs::read(&path).map_err(|_| io_error())?;
            if bytes.len() <= MAX_DOWNLOAD as usize && verify_archive(&bytes).is_ok() {
                return Ok(path);
            }
            std::fs::remove_file(&path).map_err(|_| io_error())?;
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|_| io_error())?;
        let mut response = client
            .get(url)
            .send()
            .await
            .map_err(|_| error("PACKAGE_DOWNLOAD", "Package download failed"))?;
        if !response.status().is_success()
            || response.content_length().is_some_and(|n| n > MAX_DOWNLOAD)
        {
            return Err(error(
                "PACKAGE_DOWNLOAD",
                "Package source rejected the request or exceeds download limits",
            ));
        }
        let mut temp = tempfile::NamedTempFile::new_in(&cache).map_err(|_| io_error())?;
        let download_length = response.content_length();
        let mut downloaded = 0;
        while let Some(bytes) = response
            .chunk()
            .await
            .map_err(|_| error("PACKAGE_DOWNLOAD", "Package download was interrupted"))?
        {
            downloaded += bytes.len() as u64;
            if downloaded > MAX_DOWNLOAD {
                return Err(error("PACKAGE_LIMIT", "Download exceeds 128 MiB"));
            }
            check(
                "Downloading verified dependency",
                downloaded,
                download_length,
            )?;
            temp.write_all(&bytes).map_err(|_| {
                error(
                    "PACKAGE_IO",
                    "Cannot write download; check free space and permissions",
                )
            })?;
        }
        verify_archive(&std::fs::read(temp.path()).map_err(|_| io_error())?)?;
        temp.as_file().sync_all().map_err(|_| io_error())?;
        match temp.persist_noclobber(&path) {
            Ok(_) => {}
            Err(e) if path.exists() => {
                let bytes = std::fs::read(&path).map_err(|_| io_error())?;
                verify_archive(&bytes)?;
                drop(e)
            }
            Err(_) => return Err(io_error()),
        };
        Ok(path)
    }
    async fn install_node(
        &self,
        runtime: &RuntimeDependency,
        check: &mut impl FnMut(&str, u64, Option<u64>) -> Result<()>,
    ) -> Result<RuntimeDependency> {
        let approved = self.plan("2.0.0", true)?.runtime;
        if runtime.source != approved.source
            || runtime.archive_sha256 != approved.archive_sha256
            || runtime.executable != approved.executable
            || runtime.sha256 != approved.sha256
            || runtime.version != approved.version
        {
            return Err(error(
                "RUNTIME_POLICY",
                "Managed runtime differs from the compiled Node artifact",
            ));
        }
        let executable = PathBuf::from(&runtime.executable);
        let directory = if cfg!(windows) {
            executable.parent().ok_or_else(io_error)?.to_path_buf()
        } else {
            executable
                .parent()
                .and_then(Path::parent)
                .ok_or_else(io_error)?
                .to_path_buf()
        };
        if !directory.exists() {
            let archive = self
                .download(
                    runtime.source.as_deref().ok_or_else(io_error)?,
                    None,
                    runtime.archive_sha256.as_deref(),
                    check,
                )
                .await?;
            let parent = directory.parent().ok_or_else(io_error)?;
            protect_dir(parent)?;
            let stage = tempfile::Builder::new()
                .prefix(".node-staging-")
                .tempdir_in(parent)
                .map_err(|_| io_error())?;
            let mut total = 0;
            let prefix = format!(
                "node-v{NODE_VERSION}-{}",
                if cfg!(windows) {
                    "win-x64"
                } else {
                    "linux-x64"
                }
            );
            if cfg!(windows) {
                extract_zip(&archive, stage.path(), &prefix, &mut total, check)?;
            } else {
                extract_tar(&archive, stage.path(), &prefix, true, &mut total, check)?;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(
                    stage.path().join("bin/node"),
                    std::fs::Permissions::from_mode(0o700),
                )
                .map_err(|_| io_error())?;
            }
            check("Committing managed Node runtime", total, Some(total))?;
            std::fs::rename(stage.path(), &directory).map_err(|_| io_error())?;
        }
        if file_hash(&executable)? != runtime.sha256 || node_version(&executable)? != NODE_VERSION {
            return Err(error(
                "RUNTIME_VERSION",
                "Managed Node version verification failed",
            ));
        }
        let mut runtime = runtime.clone();
        runtime.sha256 = file_hash(&executable)?;
        Ok(runtime)
    }
    fn receipt(&self, directory: &Path) -> Result<PackageReceipt> {
        let path = directory.join(".reconx-receipt.json");
        if redirected(directory) || redirected(&path) {
            return Err(error(
                "PACKAGE_PATH",
                "Managed package receipt cannot be a link",
            ));
        }
        let bytes = std::fs::read(path).map_err(|_| io_error())?;
        if bytes.len() > 8 * 1024 * 1024 {
            return Err(error("PACKAGE_LIMIT", "Package receipt exceeds limit"));
        }
        serde_json::from_slice(&bytes).map_err(|_| io_error())
    }
    pub fn verify(&self, receipt: &PackageReceipt) -> Result<()> {
        let (lock_hash, _) = graph(&receipt.version)?;
        let plan = self.plan(&receipt.version, true)?;
        let expected = self.directory(&plan);
        if receipt.lock_sha256 != lock_hash
            || Path::new(&receipt.directory) != expected
            || redirected(&expected)
            || file_hash(Path::new(&receipt.runtime.executable))? != receipt.runtime.sha256
        {
            return Err(error(
                "PACKAGE_INTEGRITY",
                "Installed package/runtime identity or integrity changed",
            ));
        }
        // One complete safe traversal detects changed, missing and extra files.
        // Hashing the same tree twice made Windows MCP calls exceed their budget.
        let mut actual = BTreeMap::new();
        collect_files(&expected, &expected, &mut actual)?;
        actual.remove(".reconx-receipt.json");
        if actual != receipt.files {
            return Err(error(
                "PACKAGE_INTEGRITY",
                "Unexpected package files require reconciliation",
            ));
        }
        Ok(())
    }
    pub fn remove(&self, receipt: &PackageReceipt) -> Result<()> {
        self.verify(receipt)?;
        let root = Path::new(&receipt.directory);
        let mut paths = receipt.files.keys().cloned().collect::<Vec<_>>();
        paths.push(".reconx-receipt.json".into());
        for path in paths {
            std::fs::remove_file(root.join(safe_relative(&path)?)).map_err(|_| io_error())?;
        }
        fn empty(path: &Path) -> Result<()> {
            if redirected(path) {
                return Err(error("PACKAGE_PATH", "Package directory became a link"));
            }
            for entry in std::fs::read_dir(path).map_err(|_| io_error())? {
                let entry = entry.map_err(|_| io_error())?;
                if entry.file_type().map_err(|_| io_error())?.is_dir() {
                    empty(&entry.path())?;
                }
            }
            std::fs::remove_dir(path).map_err(|_| {
                error(
                    "PACKAGE_RETAINED",
                    "Unexpected files were retained; package directory was not recursively deleted",
                )
            })
        }
        empty(root)
    }
}
fn extract_file(
    mut reader: impl Read,
    path: &Path,
    size: u64,
    total: &mut u64,
    check: &mut impl FnMut(&str, u64, Option<u64>) -> Result<()>,
) -> Result<()> {
    if size > MAX_DOWNLOAD || total.saturating_add(size) > MAX_EXTRACT {
        return Err(error(
            "PACKAGE_LIMIT",
            "Archive expands beyond resource limits",
        ));
    }
    *total += size;
    if let Some(parent) = path.parent() {
        protect_dir(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|_| io_error())?;
    let copied =
        std::io::copy(&mut reader.by_ref().take(size + 1), &mut file).map_err(|_| io_error())?;
    if copied != size {
        return Err(error("PACKAGE_ARCHIVE", "Archive size mismatch"));
    }
    check("Extracting verified dependency files", *total, None)?;
    Ok(())
}
fn relative_entry(name: &str, prefix: &str) -> Result<Option<PathBuf>> {
    let name = name.trim_end_matches('/');
    if name == prefix {
        return Ok(None);
    }
    let rest = name
        .strip_prefix(&format!("{prefix}/"))
        .ok_or_else(|| error("PACKAGE_PATH", "Unexpected archive root"))?;
    Ok(Some(safe_relative(rest)?))
}
fn extract_tar(
    archive: &Path,
    dest: &Path,
    prefix: &str,
    skip_links: bool,
    total: &mut u64,
    check: &mut impl FnMut(&str, u64, Option<u64>) -> Result<()>,
) -> Result<()> {
    let reader =
        flate2::read::GzDecoder::new(std::fs::File::open(archive).map_err(|_| io_error())?);
    let mut archive = tar::Archive::new(reader);
    let mut count = 0;
    for entry in archive.entries().map_err(|_| io_error())? {
        let mut entry = entry.map_err(|_| io_error())?;
        count += 1;
        if count > 50000 {
            return Err(error("PACKAGE_LIMIT", "Archive file count exceeds limit"));
        }
        let name = entry
            .path()
            .map_err(|_| io_error())?
            .to_string_lossy()
            .into_owned();
        let Some(relative) = relative_entry(&name, prefix)? else {
            continue;
        };
        // Managed runtime needs only Node itself and its license. npm is not
        // executed: the reviewed package graph is extracted directly by Rust.
        if skip_links && relative != Path::new("bin/node") && relative != Path::new("LICENSE") {
            continue;
        }
        let kind = entry.header().entry_type();
        if kind.is_symlink() && skip_links {
            continue;
        }
        if kind.is_dir() {
            protect_dir(&dest.join(relative))?;
            continue;
        }
        if !kind.is_file() {
            return Err(error(
                "PACKAGE_ARCHIVE",
                "Archive links and special files are not permitted",
            ));
        }
        let size = entry.size();
        extract_file(&mut entry, &dest.join(relative), size, total, check)?;
    }
    Ok(())
}
fn extract_zip(
    archive: &Path,
    dest: &Path,
    prefix: &str,
    total: &mut u64,
    check: &mut impl FnMut(&str, u64, Option<u64>) -> Result<()>,
) -> Result<()> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(archive).map_err(|_| io_error())?)
        .map_err(|_| io_error())?;
    if archive.len() > 50000 {
        return Err(error("PACKAGE_LIMIT", "Archive file count exceeds limit"));
    }
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|_| io_error())?;
        let name = entry.name().map_err(|_| io_error())?.into_owned();
        let Some(relative) = relative_entry(&name, prefix)? else {
            continue;
        };
        if relative != Path::new("node.exe") && relative != Path::new("LICENSE") {
            continue;
        }
        if entry.is_dir() {
            protect_dir(&dest.join(relative))?;
            continue;
        }
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err(error("PACKAGE_ARCHIVE", "ZIP links are not permitted"));
        }
        let size = entry.size();
        extract_file(&mut entry, &dest.join(relative), size, total, check)?;
    }
    Ok(())
}
fn collect_files(root: &Path, path: &Path, files: &mut BTreeMap<String, String>) -> Result<()> {
    if redirected(path)
        || !path.starts_with(root)
        || path
            .strip_prefix(root)
            .map_or(true, |p| p.components().count() > 40)
    {
        return Err(error("PACKAGE_PATH", "Managed file tree escapes its root"));
    }
    for entry in std::fs::read_dir(path).map_err(|_| io_error())? {
        let entry = entry.map_err(|_| io_error())?;
        if redirected(&entry.path()) {
            return Err(error("PACKAGE_PATH", "Managed file tree contains a link"));
        }
        if entry.file_type().map_err(|_| io_error())?.is_dir() {
            collect_files(root, &entry.path(), files)?;
        } else {
            let key = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| io_error())?
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(key, file_hash(&entry.path())?);
            if files.len() > 50000 {
                return Err(error(
                    "PACKAGE_LIMIT",
                    "Managed tree exceeds file count limit",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod package_tests {
    use super::*;
    #[test]
    fn reviewed_graph_resolves_transitive_dependencies_and_rejects_unsafe_metadata() {
        assert_eq!(graph("2.0.0").unwrap().1.len(), 94);
        assert_eq!(graph("1.0.0").unwrap().1.len(), 108);
        assert!(graph("latest").is_err());
        assert!(compatible_node("24.22.0"));
        assert!(!compatible_node("20.19.2"));
        for field in ["resolved", "hasInstallScript", "integrity"] {
            let mut value = lock("2.0.0").unwrap();
            let package = value["packages"]
                .as_object_mut()
                .unwrap()
                .iter_mut()
                .find(|(key, _)| !key.is_empty())
                .unwrap()
                .1;
            package[field] = match field {
                "resolved" => json!("https://untrusted.invalid/package.tgz"),
                "hasInstallScript" => json!(true),
                _ => json!("sha1-insecure"),
            };
            assert!(graph_value(value).is_err(), "{field}");
        }
        let mut value = lock("2.0.0").unwrap();
        let package = value["packages"]
            .as_object_mut()
            .unwrap()
            .iter_mut()
            .find(|(key, _)| !key.is_empty())
            .unwrap()
            .1;
        package["dependencies"] = json!({"nonexistent-dependency":"1"});
        assert_eq!(graph_value(value).unwrap_err().code, "DEPENDENCY_MISSING");
    }
    #[test]
    fn archive_paths_integrity_links_and_resource_limits_are_enforced() {
        for path in [
            "../escape",
            "/absolute",
            "a/../../b",
            "a\\b",
            "a:b",
            "NUL.txt",
            "a./b",
            "a//b",
        ] {
            assert!(safe_relative(path).is_err(), "{path}");
        }
        assert!(safe_relative(&vec!["a"; 34].join("/")).is_err());
        let digest = format!(
            "sha512-{}",
            base64::engine::general_purpose::STANDARD.encode(sha2::Sha512::digest(b"payload"))
        );
        assert!(sri(b"payload", &digest).is_ok());
        assert_eq!(
            sri(b"corrupt", &digest).unwrap_err().code,
            "PACKAGE_INTEGRITY"
        );
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("unsafe.tgz");
        let gzip = flate2::write::GzEncoder::new(
            std::fs::File::create(&archive).unwrap(),
            flate2::Compression::default(),
        );
        let mut tar = tar::Builder::new(gzip);
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        header.set_mode(0o777);
        header.set_link_name("../../outside").unwrap();
        header.set_cksum();
        tar.append_data(&mut header, "package/link", std::io::empty())
            .unwrap();
        tar.into_inner().unwrap().finish().unwrap();
        let dest = dir.path().join("dest");
        protect_dir(&dest).unwrap();
        assert_eq!(
            extract_tar(
                &archive,
                &dest,
                "package",
                false,
                &mut 0,
                &mut |_, _, _| Ok(())
            )
            .unwrap_err()
            .code,
            "PACKAGE_ARCHIVE"
        );
        assert!(!dest.join("link").exists());
        assert_eq!(
            extract_file(
                std::io::empty(),
                &dest.join("huge"),
                MAX_DOWNLOAD + 1,
                &mut 0,
                &mut |_, _, _| Ok(())
            )
            .unwrap_err()
            .code,
            "PACKAGE_LIMIT"
        );
        assert_eq!(
            extract_file(
                &b"data"[..],
                &dest.join("cancel"),
                4,
                &mut 0,
                &mut |_, _, _| Err(error("CANCELLED", "Stopped"))
            )
            .unwrap_err()
            .code,
            "CANCELLED"
        );
        assert!(free_space(dir.path()).unwrap() > 0);
    }
    #[tokio::test]
    async fn altered_install_plan_is_rejected_before_network_or_file_changes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("extensions");
        let manager = PackageManager::new(root.clone());
        let mut plan = manager.plan("2.0.0", true).unwrap();
        plan.minimum_free_bytes = 0;
        assert_eq!(
            manager
                .install(&plan, |_, _, _| Ok(()))
                .await
                .unwrap_err()
                .code,
            "PACKAGE_POLICY"
        );
        assert!(!root.exists());
        plan = manager.plan("2.0.0", true).unwrap();
        plan.dependencies[0].integrity = "sha512-false".into();
        assert_eq!(
            manager
                .install(&plan, |_, _, _| Ok(()))
                .await
                .unwrap_err()
                .code,
            "PACKAGE_POLICY"
        );
        assert!(!root.exists());
    }
}
