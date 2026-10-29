//! Compiled ProjectDiscovery arguments; no shell, ambient config or provider keys.
use super::*;
use std::{
    io::{Seek, SeekFrom},
    process::{Command, Stdio},
    thread,
    time::Instant,
};

pub enum ReconToolRequest {
    Httpx {
        transport_url: String,
        host: String,
        timeout_secs: u32,
        technology: bool,
    },
    Subfinder {
        domain: String,
        sources: Vec<String>,
        timeout_secs: u32,
    },
    Sources,
}
pub struct ReconOutput {
    pub stdout: String,
    pub stderr: String,
}
impl ToolManager {
    pub fn discover_recon(&self, id: &str) -> Result<Option<PathBuf>> {
        let m = self.plan(id)?;
        for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
            let path = dir.join(&m.executable);
            if let Ok(meta) = std::fs::symlink_metadata(&path)
                && meta.is_file()
                && !meta.file_type().is_symlink()
                && meta.len() <= 128 * 1024 * 1024
                && let Ok(bytes) = std::fs::read(&path)
                && verify(&bytes, &m.binary_sha256).is_ok()
            {
                return std::fs::canonicalize(path)
                    .map(Some)
                    .map_err(|_| io_error());
            }
        }
        Ok(None)
    }
    pub fn adopt_recon(&self, id: &str, path: &std::path::Path) -> Result<()> {
        let m = self.plan(id)?;
        let meta = std::fs::symlink_metadata(path).map_err(|_| io_error())?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 128 * 1024 * 1024 {
            return Err(AppError::new(
                "INTEGRITY_FAILURE",
                "Existing tool is not a bounded regular executable",
            ));
        }
        let bytes = std::fs::read(path).map_err(|_| io_error())?;
        verify(&bytes, &m.binary_sha256)?;
        let directory = self.directory(&m);
        std::fs::create_dir_all(&directory).map_err(|_| io_error())?;
        self.validate_directory(&directory)?;
        let mut tmp = tempfile::NamedTempFile::new_in(&directory).map_err(|_| io_error())?;
        tmp.write_all(&bytes).map_err(|_| io_error())?;
        tmp.as_file().sync_all().map_err(|_| io_error())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tmp.as_file()
                .set_permissions(std::fs::Permissions::from_mode(0o700))
                .map_err(|_| io_error())?;
        }
        let destination = directory.join(&m.executable);
        match std::fs::symlink_metadata(&destination) {
            Ok(metadata) => {
                // Repair may replace only our receipt-owned regular binary.
                // Preserve unexpected files and symlinks; never delete first.
                let owned = self
                    .receipt(&m)
                    .is_some_and(|r| r.id == m.id && r.version == m.version);
                if !metadata.is_file() || metadata.file_type().is_symlink() || !owned {
                    return Err(AppError::new(
                        "TOOL_DESTINATION_UNSAFE",
                        "Existing executable is not a receipt-owned regular managed file",
                    ));
                }
                tmp.persist(&destination).map_err(|_| io_error())?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                tmp.persist_noclobber(&destination)
                    .map_err(|_| io_error())?;
            }
            Err(_) => return Err(io_error()),
        }
        self.write_receipt(
            &m,
            &Receipt {
                id: m.id.clone(),
                version: m.version.clone(),
                installed_at: Utc::now().to_rfc3339(),
                last_health_check: None,
            },
        )
    }
    pub fn recon_health(&self, id: &str) -> Result<()> {
        if id == "subfinder" {
            let out = self.recon_execute(ReconToolRequest::Sources, || Ok(()))?;
            if out.stdout.contains("crtsh") || out.stderr.contains("crtsh") {
                return Ok(());
            }
            return Err(AppError::new(
                "TOOL_HEALTH_FAILURE",
                "Subfinder source inventory lacks the reviewed source",
            ));
        }
        use std::{
            net::TcpListener,
            sync::{
                Arc,
                atomic::{AtomicBool, AtomicUsize, Ordering},
            },
        };
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| io_error())?;
        listener.set_nonblocking(true).map_err(|_| io_error())?;
        let address = listener.local_addr().map_err(|_| io_error())?;
        let stop = Arc::new(AtomicBool::new(false));
        let count = Arc::new(AtomicUsize::new(0));
        let (quit, calls) = (stop.clone(), count.clone());
        let worker = thread::spawn(move || {
            while !quit.load(Ordering::Acquire) {
                if let Ok((mut stream, _)) = listener.accept() {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                    let mut b = [0; 8192];
                    if let Ok(n) = stream.read(&mut b)
                        && b[..n].starts_with(b"GET ")
                    {
                        calls.fetch_add(1, Ordering::SeqCst);
                        let body = b"<html><title>ReconX HTTPX health</title></html>";
                        let _ = write!(
                            stream,
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(body);
                    }
                } else {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });
        let out = self.recon_execute(
            ReconToolRequest::Httpx {
                transport_url: format!("http://{address}/"),
                host: address.to_string(),
                timeout_secs: 5,
                technology: true,
            },
            || Ok(()),
        );
        stop.store(true, Ordering::Release);
        let _ = worker.join();
        let out = out?;
        let value: serde_json::Value = serde_json::from_str(out.stdout.trim()).map_err(|_| {
            AppError::new("TOOL_HEALTH_FAILURE", "HTTPX failed real local JSON probe")
        })?;
        if value["status_code"] != 200
            || value["title"] != "ReconX HTTPX health"
            || count.load(Ordering::Acquire) != 1
        {
            return Err(AppError::new(
                "TOOL_HEALTH_FAILURE",
                "HTTPX did not perform exactly the expected owned-loopback GET",
            ));
        }
        Ok(())
    }
    pub fn recon_execute(
        &self,
        request: ReconToolRequest,
        mut check: impl FnMut() -> Result<()>,
    ) -> Result<ReconOutput> {
        let id = if matches!(request, ReconToolRequest::Httpx { .. }) {
            "httpx"
        } else {
            "subfinder"
        };
        let executable = self.verify_installed(&self.plan(id)?)?;
        let temp = tempfile::tempdir().map_err(|_| io_error())?;
        let config = temp.path().join("config.yaml");
        std::fs::write(&config, b"{}\n").map_err(|_| io_error())?;
        let providers = temp.path().join("providers.yaml");
        std::fs::write(&providers, b"{}\n").map_err(|_| io_error())?;
        let mut args = vec!["-duc".into(), "-nc".into()];
        let mut timeout = 15;
        match request {
            ReconToolRequest::Httpx {
                transport_url,
                host,
                timeout_secs,
                technology,
            } => {
                let u = reqwest::Url::parse(&transport_url)
                    .map_err(|_| AppError::new("INVALID_INPUT", "Invalid pinned probe URL"))?;
                if u.host_str()
                    .and_then(|s| s.trim_matches(['[', ']']).parse::<std::net::IpAddr>().ok())
                    .is_none()
                    || !matches!(u.scheme(), "http" | "https")
                    || host.bytes().any(|c| c == b'\r' || c == b'\n')
                {
                    return Err(AppError::new(
                        "INVALID_INPUT",
                        "Probe transport must be a pinned numeric IP",
                    ));
                }
                args.extend([
                    "-u".into(),
                    transport_url,
                    "-H".into(),
                    format!("Host: {host}"),
                    "-json".into(),
                    "-silent".into(),
                    "-sc".into(),
                    "-title".into(),
                    "-ct".into(),
                    "-rt".into(),
                    "-ip".into(),
                    "-tls-grab".into(),
                    "-nfs".into(),
                    "-no-stdin".into(),
                    "-t".into(),
                    "1".into(),
                    "-rl".into(),
                    "1".into(),
                    "-retries".into(),
                    "0".into(),
                    "-timeout".into(),
                    timeout_secs.clamp(1, 15).to_string(),
                    "-rstr".into(),
                    "65536".into(),
                    "-rsts".into(),
                    "0".into(),
                    "-cdn=false".into(),
                    "-random-agent=false".into(),
                    "-auth=false".into(),
                    "-config".into(),
                    config.to_string_lossy().into(),
                ]);
                if technology {
                    args.push("-td".into());
                }
                if let Ok(authority) = reqwest::Url::parse(&format!("http://{host}/"))
                    && let Some(name) = authority.host_str()
                    && name
                        .trim_matches(['[', ']'])
                        .parse::<std::net::IpAddr>()
                        .is_err()
                {
                    args.extend(["-sni".into(), name.into()]);
                }
            }
            ReconToolRequest::Subfinder {
                domain,
                sources,
                timeout_secs,
            } => {
                if domain.starts_with('-')
                    || domain.len() > 253
                    || !domain
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b".-".contains(&c))
                    || sources.is_empty()
                    || sources
                        .iter()
                        .any(|s| !["crtsh", "hackertarget", "certspotter"].contains(&s.as_str()))
                {
                    return Err(AppError::new(
                        "INVALID_INPUT",
                        "Invalid passive domain or source selection",
                    ));
                }
                timeout = timeout_secs.clamp(1, 120);
                args.extend([
                    "-d".into(),
                    domain,
                    "-s".into(),
                    sources.join(","),
                    "-oJ".into(),
                    "-cs".into(),
                    "-rl".into(),
                    "2".into(),
                    "-timeout".into(),
                    "10".into(),
                    "-max-time".into(),
                    "2".into(),
                    "-mr".into(),
                    "100".into(),
                    "-rsr".into(),
                    "1048576".into(),
                    "-config".into(),
                    config.to_string_lossy().into(),
                    "-pc".into(),
                    providers.to_string_lossy().into(),
                ]);
            }
            ReconToolRequest::Sources => args.extend(["-ls".into(), "-oJ".into()]),
        }
        let stdout = tempfile::tempfile().map_err(|_| io_error())?;
        let stderr = tempfile::tempfile().map_err(|_| io_error())?;
        let mut command = Command::new(executable);
        command.env_clear();
        for key in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
            if let Some(v) = std::env::var_os(key) {
                command.env(key, v);
            }
        }
        for key in [
            "HOME",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "XDG_CONFIG_HOME",
        ] {
            command.env(key, temp.path());
        }
        command
            .current_dir(temp.path())
            .args(args)
            .stdin(Stdio::null())
            .stdout(stdout.try_clone().map_err(|_| io_error())?)
            .stderr(stderr.try_clone().map_err(|_| io_error())?);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        check()?;
        let mut child = command
            .spawn()
            .map_err(|_| AppError::new("TOOL_EXECUTION", "Cannot start verified recon tool"))?;
        let start = Instant::now();
        let status = loop {
            let state = check().and_then(|_| {
                if start.elapsed() > Duration::from_secs(timeout as u64 + 5) {
                    Err(AppError::new(
                        "TOOL_TIMEOUT",
                        "Recon tool deadline exceeded",
                    ))
                } else if stdout.metadata().map_err(|_| io_error())?.len()
                    + stderr.metadata().map_err(|_| io_error())?.len()
                    > 1024 * 1024
                {
                    Err(AppError::new("OUTPUT_LIMIT", "Recon output exceeded 1 MiB"))
                } else {
                    Ok(())
                }
            });
            if let Err(e) = state {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
            match child.try_wait() {
                Ok(Some(s)) => break s,
                Ok(None) => thread::sleep(Duration::from_millis(40)),
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(AppError::new("TOOL_EXECUTION", "Recon child failed"));
                }
            }
        };
        fn capture(mut file: std::fs::File) -> Result<String> {
            file.seek(SeekFrom::Start(0)).map_err(|_| io_error())?;
            let mut s = String::new();
            file.take(1024 * 1024 + 1)
                .read_to_string(&mut s)
                .map_err(|_| io_error())?;
            if s.len() > 1024 * 1024 {
                return Err(AppError::new("OUTPUT_LIMIT", "Recon output limit exceeded"));
            }
            Ok(s)
        }
        let output = ReconOutput {
            stdout: capture(stdout)?,
            stderr: capture(stderr)?,
        };
        if !status.success() {
            return Err(AppError::new(
                "TOOL_EXECUTION",
                "Verified recon tool failed; no successful empty result recorded",
            ));
        }
        check()?;
        Ok(output)
    }
    pub fn pinned_recon_path(&self, id: &str) -> Result<PathBuf> {
        self.verify_installed(&self.plan(id)?)
    }
}
