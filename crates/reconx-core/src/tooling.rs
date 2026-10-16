//! Registry metadata does not imply installation or execution support.
use super::*;
use serde_json::{Value, json};
const CANDIDATES: &[(&str, &str)] = &[
    ("subfinder", "recon"),
    ("amass", "recon"),
    ("httpx", "recon"),
    ("dnsx", "recon"),
    ("katana", "recon"),
    ("nuclei", "web"),
    ("naabu", "network"),
    ("zap", "web"),
    ("ffuf", "web"),
    ("sqlmap", "web"),
    ("nikto", "web"),
    ("nmap", "network"),
    ("masscan", "network"),
    ("tshark", "network"),
    ("ghidra", "reverse"),
    ("radare2", "reverse"),
    ("rizin", "reverse"),
    ("yara", "reverse"),
    ("binwalk", "reverse"),
    ("x64dbg", "reverse"),
    ("windbg", "reverse"),
    ("adb", "mobile"),
    ("jadx", "mobile"),
    ("apktool", "mobile"),
    ("frida", "mobile"),
    ("objection", "mobile"),
    ("trivy", "cloud"),
    ("checkov", "cloud"),
    ("semgrep", "cicd"),
    ("syft", "cloud"),
    ("grype", "cloud"),
    ("actionlint", "cicd"),
];
impl Core {
    pub fn tool_history(&self, id: &str) -> Result<Value> {
        self.tool_plan(id)?;
        let operations = self
            .audit_events()?
            .into_iter()
            .filter(|e| e.entity_id == id && e.action.starts_with("tool."))
            .collect::<Vec<_>>();
        let jobs = if id == "yara-x" {
            self.list_jobs(None)?
                .into_iter()
                .filter(|j| j.kind == "yara_signatures")
                .take(20)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        Ok(
            json!({"operations":operations,"executions":jobs,"note":"Most recent 100 audit events and 20 analysis executions; download byte counters are live only"}),
        )
    }
    pub fn tool_catalog(&self) -> Result<Vec<Value>> {
        let mut entries=self.tool_inventory()?.into_iter().map(|t|json!({"id":t.manifest.id,"name":t.manifest.name,"category":t.manifest.category,"integration":"implemented","status":t.status,"version":t.manifest.version,"integrity_verified":t.integrity_verified,"manifest":t.manifest})).collect::<Vec<_>>();
        for (id, category) in CANDIDATES {
            entries.push(json!({"id":id,"name":id,"category":category,"integration":"pending","status":"pending_integration","note":"No verified installer or execution adapter is available. Registration here is roadmap metadata only."}));
        }
        Ok(entries)
    }
    pub fn tool_profiles(&self) -> Value {
        json!([
            {"id":"core","tools":["ripgrep"],"installable":true},
            {"id":"reverse","tools":["ripgrep","yara-x"],"installable":true},
            {"id":"recon","tools":["subfinder","dnsx","httpx"],"installable":false},
            {"id":"web","tools":["zap","nuclei"],"installable":false},
            {"id":"api","tools":["zap"],"installable":false},
            {"id":"mobile","tools":["adb","jadx","apktool"],"installable":false},
            {"id":"cloud","tools":["trivy","checkov"],"installable":false},
            {"id":"full","tools":["ripgrep","yara-x","subfinder","zap","jadx","trivy"],"installable":false}
        ])
    }
    pub fn tool_profile_plan(&self, id: &str) -> Result<Vec<ToolManifest>> {
        let profiles = self.tool_profiles();
        let profile = profiles
            .as_array()
            .and_then(|a| a.iter().find(|p| p["id"] == id))
            .ok_or_else(|| AppError::new("UNKNOWN_PROFILE", "Unknown tool profile"))?;
        if profile["installable"] != true {
            return Err(AppError::new(
                "PROFILE_INCOMPLETE",
                "This profile contains pending adapters and cannot be installed yet",
            ));
        }
        profile["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| self.tool_plan(t.as_str().unwrap_or_default()))
            .collect()
    }
    pub fn tool_host(&self) -> Value {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let executable = |name: &str| {
            std::env::split_paths(&path).any(|p| {
                p.join(if cfg!(windows) {
                    format!("{name}.exe")
                } else {
                    name.into()
                })
                .is_file()
            })
        };
        let package_managers = ["winget", "scoop", "apt", "dnf", "pacman", "wsl"]
            .into_iter()
            .filter(|name| executable(name))
            .collect::<Vec<_>>();
        let distro = std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|s| {
                s.lines().find_map(|l| {
                    l.strip_prefix("PRETTY_NAME=")
                        .map(|s| s.trim_matches('"').to_owned())
                })
            });
        json!({"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"distribution":distro,"available_package_managers":package_managers,"portable_install":true,"automatic_elevation":false,"wsl_execution":"not_implemented"})
    }
    pub(super) fn persist_tool_state(&self, id: &str) -> Result<()> {
        let state = self
            .tool_inventory()?
            .into_iter()
            .find(|t| t.manifest.id == id)
            .ok_or_else(|| AppError::new("UNKNOWN_TOOL", "Tool not in registry"))?;
        self.db.connect()?.execute("INSERT INTO tool_installations(tool_id,state_json,updated_at) VALUES(?1,?2,?3) ON CONFLICT(tool_id) DO UPDATE SET state_json=excluded.state_json,updated_at=excluded.updated_at",params![id,serde_json::to_string(&state).map_err(json_error)?,now()]).map_err(db_error)?;
        Ok(())
    }
}
