use super::*;
use ipnet::IpNet;
use std::{
    net::IpAddr,
    path::{Component, Path},
};
use url::Url;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Domain,
    Subdomain,
    Url,
    Ipv4,
    Ipv6,
    Cidr,
    Hostname,
    RestApi,
    Graphql,
    GitRepository,
    SourceDirectory,
    Binary,
    SharedLibrary,
    Firmware,
    Apk,
    ContainerImage,
    IacDirectory,
}
impl TargetKind {
    pub fn local_file(self) -> bool {
        matches!(
            self,
            Self::Binary | Self::SharedLibrary | Self::Firmware | Self::Apk
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetInput {
    pub project_id: String,
    pub name: String,
    pub kind: TargetKind,
    pub value: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub allowed_scope: Vec<String>,
    #[serde(default)]
    pub excluded_scope: Vec<String>,
    #[serde(default)]
    pub authorized: bool,
    #[serde(default)]
    pub mode: SecurityMode,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub scope_status: String,
    #[serde(flatten)]
    pub input: TargetInput,
    pub created_at: String,
    pub updated_at: String,
    pub last_analyzed_at: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct ScopeDecision {
    pub target_id: String,
    pub candidate: String,
    pub authorized: bool,
    pub in_scope: bool,
    pub status: String,
    pub reason: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetExport {
    pub format_version: u32,
    pub targets: Vec<TargetInput>,
}

fn invalid(message: &str) -> AppError {
    AppError::new("INVALID_TARGET", message)
}
fn hostname(raw: &str, domain: bool) -> Result<String> {
    let value = raw.trim_end_matches('.').to_ascii_lowercase();
    if value.len() > 253
        || (domain && !value.contains('.'))
        || value.parse::<IpAddr>().is_ok()
        || value.split('.').any(|l| {
            l.is_empty()
                || l.len() > 63
                || l.starts_with('-')
                || l.ends_with('-')
                || !l.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
    {
        return Err(invalid(
            "Enter a valid ASCII hostname (use punycode for international domains)",
        ));
    }
    Ok(value)
}
fn web_url(raw: &str) -> Result<String> {
    let mut url = Url::parse(raw).map_err(|_| invalid("Enter an absolute HTTP or HTTPS URL"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid(
            "Use HTTP(S) without embedded credentials, query parameters or fragments",
        ));
    }
    if url.path().is_empty() {
        url.set_path("/");
    }
    Ok(url.to_string())
}
fn local_path(raw: &str) -> Result<String> {
    let path = Path::new(raw);
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(invalid(
            "Local targets require an absolute path without parent traversal",
        ));
    }
    Ok(path
        .canonicalize()
        .unwrap_or_else(|_| path.to_owned())
        .to_string_lossy()
        .into_owned())
}
pub fn normalize_target_value(kind: TargetKind, raw: &str) -> Result<String> {
    let raw = validate_text(raw, 4096, "Target value")?;
    if raw.contains(['\n', '\t']) {
        return Err(invalid("Target values must occupy one line"));
    }
    match kind {
        TargetKind::Domain | TargetKind::Subdomain => hostname(&raw, true),
        TargetKind::Hostname => hostname(&raw, false),
        TargetKind::Url | TargetKind::RestApi | TargetKind::Graphql => web_url(&raw),
        TargetKind::Ipv4 => raw
            .parse::<std::net::Ipv4Addr>()
            .map(|v| v.to_string())
            .map_err(|_| invalid("Invalid IPv4 address")),
        TargetKind::Ipv6 => raw
            .parse::<std::net::Ipv6Addr>()
            .map(|v| v.to_string())
            .map_err(|_| invalid("Invalid IPv6 address")),
        TargetKind::Cidr => raw
            .parse::<IpNet>()
            .map(|v| v.trunc().to_string())
            .map_err(|_| invalid("Invalid CIDR network")),
        TargetKind::GitRepository => {
            if Path::new(&raw).is_absolute() {
                local_path(&raw)
            } else {
                web_url(&raw)
            }
        }
        TargetKind::SourceDirectory
        | TargetKind::Binary
        | TargetKind::SharedLibrary
        | TargetKind::Firmware
        | TargetKind::Apk
        | TargetKind::IacDirectory => {
            if let Ok(metadata) = std::fs::metadata(&raw)
                && ((kind.local_file() && !metadata.is_file())
                    || (!kind.local_file() && !metadata.is_dir()))
            {
                return Err(invalid(
                    "Local path does not match the selected file or directory target type",
                ));
            }
            local_path(&raw)
        }
        TargetKind::ContainerImage => {
            if raw.len() > 512
                || !raw
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"/._-:@".contains(&c))
                || raw.starts_with('-')
            {
                return Err(invalid("Invalid container image reference"));
            }
            Ok(raw)
        }
    }
}
fn normalize_rule(raw: &str) -> Result<String> {
    let raw = validate_text(raw, 4096, "Scope rule")?;
    if let Some(suffix) = raw.strip_prefix("*.") {
        return Ok(format!("*.{}", hostname(suffix, true)?));
    }
    if raw.contains("://") {
        return web_url(&raw);
    }
    if Path::new(&raw).is_absolute() {
        return local_path(&raw);
    }
    if let Ok(net) = raw.parse::<IpNet>() {
        return Ok(net.trunc().to_string());
    }
    if let Ok(ip) = raw.parse::<IpAddr>() {
        return Ok(ip.to_string());
    }
    hostname(&raw, false)
}
fn rule_matches(rule: &str, candidate: &str) -> bool {
    if rule == candidate {
        return true;
    }
    if let Some(suffix) = rule.strip_prefix("*.") {
        let host = Url::parse(candidate)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned))
            .unwrap_or_else(|| candidate.into());
        return host.ends_with(&format!(".{suffix}"));
    }
    if let Ok(net) = rule.parse::<IpNet>() {
        if let Ok(ip) = candidate.parse::<IpAddr>() {
            return net.contains(&ip);
        }
        if let Ok(other) = candidate.parse::<IpNet>() {
            return net.prefix_len() <= other.prefix_len() && net.contains(&other.network());
        }
    }
    if let (Ok(a), Ok(b)) = (Url::parse(rule), Url::parse(candidate)) {
        let prefix = a.path().trim_end_matches('/');
        return a.origin() == b.origin()
            && (b.path() == prefix || b.path().starts_with(&format!("{prefix}/")));
    }
    if Path::new(rule).is_absolute() && Path::new(candidate).is_absolute() {
        return Path::new(candidate).starts_with(rule);
    }
    // A hostname scope covers that host's URLs, never suffix lookalikes.
    Url::parse(candidate)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .is_some_and(|h| h == rule)
}
fn scope_for(input: &TargetInput, candidate: &str) -> (&'static str, &'static str, bool) {
    let excluded = input
        .excluded_scope
        .iter()
        .any(|r| rule_matches(r, candidate));
    let included = input
        .allowed_scope
        .iter()
        .any(|r| rule_matches(r, candidate));
    let (status, reason) = if excluded {
        ("excluded", "An exclusion rule takes precedence")
    } else if !included {
        ("out_of_scope", "No allowed rule matches this value")
    } else if !input.authorized {
        (
            "awaiting_authorization",
            "Authorization has not been confirmed",
        )
    } else {
        (
            "allowed",
            "Registered authorization and scope match; execution permissions are separate",
        )
    };
    (status, reason, included && !excluded)
}
impl Core {
    pub(super) fn validate_target_input(&self, mut input: TargetInput) -> Result<TargetInput> {
        let project = self.project(&input.project_id)?;
        if project.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project before adding or editing targets",
            ));
        }
        input.project_id = project.id;
        input.name = validate_text(&input.name, 120, "Target name")?;
        if input.name.contains(['\n', '\t'])
            || input.description.len() > 8192
            || input.tags.len() > 32
        {
            return Err(invalid(
                "Target name, description or tag count exceeds its limit",
            ));
        }
        input.value = normalize_target_value(input.kind, &input.value)?;
        input.tags = input
            .tags
            .iter()
            .map(|t| validate_text(t, 64, "Tag"))
            .collect::<Result<_>>()?;
        input.tags.sort();
        input.tags.dedup();
        if input.allowed_scope.is_empty() {
            input.allowed_scope.push(input.value.clone());
        }
        for rules in [&mut input.allowed_scope, &mut input.excluded_scope] {
            if rules.len() > 100 {
                return Err(invalid("Scope is limited to 100 rules per list"));
            }
            *rules = rules
                .iter()
                .map(|r| {
                    if r == &input.value {
                        Ok(r.clone())
                    } else {
                        normalize_rule(r)
                    }
                })
                .collect::<Result<_>>()?;
            rules.sort();
            rules.dedup();
        }
        Ok(input)
    }
    pub(super) fn write_target(tx: &Connection, target: &Target) -> Result<()> {
        tx.execute("INSERT INTO targets(id,project_id,name,kind,value,json,created_at,updated_at,last_analyzed_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![target.id,target.input.project_id,target.input.name,serde_json::to_string(&target.input.kind).map_err(json_error)?,target.input.value,serde_json::to_string(&target.input).map_err(json_error)?,target.created_at,target.updated_at,target.last_analyzed_at])
            .map_err(|e| if matches!(e, reconx_storage::rusqlite::Error::SqliteFailure(ref f,_) if f.code==reconx_storage::rusqlite::ErrorCode::ConstraintViolation) {
                AppError::new("DUPLICATE_TARGET","This target type and value already exist in the project")
            } else {db_error(e)})?;
        Self::audit(tx, "target.created", &target.id, "completed")
    }
    pub fn create_target(&self, input: TargetInput) -> Result<Target> {
        let export = TargetExport {
            format_version: 1,
            targets: vec![input],
        };
        Ok(self.import_targets(export)?.remove(0))
    }
    pub fn import_targets(&self, export: TargetExport) -> Result<Vec<Target>> {
        if export.format_version != 1 || export.targets.is_empty() || export.targets.len() > 500 {
            return Err(AppError::new(
                "INVALID_IMPORT",
                "Expected version 1 with 1–500 targets",
            ));
        }
        let mut targets = Vec::new();
        for input in export.targets {
            let input = self.validate_target_input(input)?;
            targets.push(Target {
                id: Uuid::new_v4().to_string(),
                scope_status: scope_for(&input, &input.value).0.into(),
                input,
                created_at: now(),
                updated_at: now(),
                last_analyzed_at: None,
            });
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        for target in &targets {
            Self::write_target(&tx, target)?;
        }
        tx.commit().map_err(db_error)?;
        Ok(targets)
    }
    pub fn list_targets(&self, project: Option<&str>) -> Result<Vec<Target>> {
        let project = project.map(|p| self.project(p).map(|p| p.id)).transpose()?;
        let conn = self.db.connect()?;
        let mut stmt=conn.prepare("SELECT id,json,created_at,updated_at,last_analyzed_at FROM targets WHERE ?1 IS NULL OR project_id=?1 ORDER BY updated_at DESC,id").map_err(db_error)?;
        let rows = stmt
            .query_map([project], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                ))
            })
            .map_err(db_error)?;
        rows.map(|r| {
            let (id, json, created_at, updated_at, last_analyzed_at) = r.map_err(db_error)?;
            let input: TargetInput = serde_json::from_str(&json).map_err(json_error)?;
            Ok(Target {
                id,
                scope_status: scope_for(&input, &input.value).0.into(),
                input,
                created_at,
                updated_at,
                last_analyzed_at,
            })
        })
        .collect()
    }
    pub fn target(&self, id: &str) -> Result<Target> {
        self.list_targets(None)?
            .into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| AppError::new("NOT_FOUND", "Target does not exist"))
    }
    pub fn update_target(&self, id: &str, input: TargetInput) -> Result<Target> {
        let previous = self.target(id)?;
        let input = self.validate_target_input(input)?;
        if input.project_id != previous.input.project_id {
            return Err(invalid(
                "Moving targets between projects is not supported; export and import instead",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute(
            "UPDATE targets SET name=?1,kind=?2,value=?3,json=?4,updated_at=?5 WHERE id=?6",
            params![
                input.name,
                serde_json::to_string(&input.kind).map_err(json_error)?,
                input.value,
                serde_json::to_string(&input).map_err(json_error)?,
                now(),
                id
            ],
        )
        .map_err(duplicate_error)?;
        Self::audit(&tx, "target.updated", id, "completed")?;
        tx.commit().map_err(db_error)?;
        self.target(id)
    }
    pub fn remove_target(&self, id: &str) -> Result<()> {
        self.target(id)?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("DELETE FROM targets WHERE id=?1", [id])
            .map_err(db_error)?;
        Self::audit(&tx, "target.removed", id, "completed")?;
        tx.commit().map_err(db_error)
    }
    pub fn export_targets(&self, project: Option<&str>) -> Result<TargetExport> {
        Ok(TargetExport {
            format_version: 1,
            targets: self
                .list_targets(project)?
                .into_iter()
                .map(|t| t.input)
                .collect(),
        })
    }
    pub fn validate_scope(&self, id: &str, candidate: Option<&str>) -> Result<ScopeDecision> {
        let target = self.target(id)?;
        let input = &target.input;
        let candidate = match candidate {
            Some(c) => normalize_rule(c)?,
            None => input.value.clone(),
        };
        let (status, reason, in_scope) = scope_for(input, &candidate);
        Ok(ScopeDecision {
            target_id: id.into(),
            candidate,
            authorized: input.authorized,
            in_scope,
            status: status.into(),
            reason: reason.into(),
        })
    }
}
