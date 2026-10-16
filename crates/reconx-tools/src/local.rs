//! Fixed, offline file signatures. No caller-controlled shell or scanner arguments.
use super::*;
use std::{
    process::{Command, Stdio},
    thread,
};

pub const SIGNATURE_RULES: &str = r#"
rule PE_Header { condition: uint16(0) == 0x5a4d }
rule ELF_Header { condition: uint32(0) == 0x464c457f }
rule ReconX_Test_Marker { strings: $a = "RECONX_BENIGN_FIXTURE" condition: $a }
"#;

impl ToolManager {
    pub fn signatures(
        &self,
        bytes: &[u8],
        mut check: impl FnMut() -> Result<()>,
    ) -> Result<serde_json::Value> {
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(AppError::new("INPUT_LIMIT", "Sample exceeds 64 MiB"));
        }
        let m = self.plan("yara-x")?;
        let executable = self.verify_installed(&m)?;
        let temp = tempfile::tempdir().map_err(|_| io_error())?;
        let rules = temp.path().join("signatures.yar");
        let sample = temp.path().join("sample.bin");
        std::fs::write(&rules, SIGNATURE_RULES).map_err(|_| io_error())?;
        std::fs::write(&sample, bytes).map_err(|_| io_error())?;
        let stdout = tempfile::tempfile().map_err(|_| io_error())?;
        let stderr = tempfile::tempfile().map_err(|_| io_error())?;
        let mut cmd = Command::new(executable);
        cmd.env_clear();
        for key in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
            if let Some(v) = std::env::var_os(key) {
                cmd.env(key, v);
            }
        }
        cmd.current_dir(temp.path())
            .args([
                "scan",
                "--output-format=json",
                "--threads=1",
                "--timeout=30",
                "--disable-console-logs",
            ])
            .arg(&rules)
            .arg(&sample)
            .stdin(Stdio::null())
            .stdout(stdout.try_clone().map_err(|_| io_error())?)
            .stderr(stderr.try_clone().map_err(|_| io_error())?);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        check()?;
        let mut child = cmd
            .spawn()
            .map_err(|_| AppError::new("TOOL_EXECUTION", "Cannot start verified YARA-X"))?;
        let start = std::time::Instant::now();
        let status = loop {
            let limit = stdout.metadata().map_err(|_| io_error())?.len()
                + stderr.metadata().map_err(|_| io_error())?.len();
            let state = check().and_then(|_| {
                if limit > 256 * 1024 {
                    Err(AppError::new(
                        "OUTPUT_LIMIT",
                        "Tool output exceeded 256 KiB",
                    ))
                } else if start.elapsed() > Duration::from_secs(35) {
                    Err(AppError::new("TOOL_TIMEOUT", "YARA-X timeout"))
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
                Ok(None) => thread::sleep(Duration::from_millis(30)),
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(AppError::new(
                        "TOOL_EXECUTION",
                        "Cannot wait for analysis process",
                    ));
                }
            }
        };
        if !status.success() {
            return Err(AppError::new(
                "TOOL_EXECUTION",
                "YARA-X rejected the fixed local analysis",
            ));
        }
        use std::io::{Seek, SeekFrom};
        let mut stdout = stdout;
        stdout.seek(SeekFrom::Start(0)).map_err(|_| io_error())?;
        let mut output = String::new();
        stdout
            .take(256 * 1024)
            .read_to_string(&mut output)
            .map_err(|_| io_error())?;
        let mut result: serde_json::Value = serde_json::from_str(&output)
            .map_err(|_| AppError::new("TOOL_OUTPUT", "YARA-X returned invalid JSON"))?;
        // Never retain temporary machine paths in shareable structured results.
        fn strip(v: &mut serde_json::Value) {
            match v {
                serde_json::Value::Object(o) => {
                    for (k, v) in o {
                        if matches!(k.as_str(), "path" | "file" | "target") && v.is_string() {
                            *v = serde_json::json!("managed-sample")
                        } else {
                            strip(v)
                        }
                    }
                }
                serde_json::Value::Array(a) => a.iter_mut().for_each(strip),
                _ => {}
            }
        }
        strip(&mut result);
        Ok(
            serde_json::json!({"tool":"yara-x","version":m.version,"rules_sha256":hash(SIGNATURE_RULES.as_bytes()),"sample_sha256":hash(bytes),"observations":result,"verification":"File signatures only; not a vulnerability verdict"}),
        )
    }
}
