//! Project/target-bound identity metadata. Credential values have no read/export API.
use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HttpAuthKind {
    Anonymous,
    Cookie,
    Bearer,
    ApiKey,
    Basic,
    CustomHeader,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpAuthInput {
    pub project_id: String,
    pub target_id: String,
    pub name: String,
    pub role: String,
    pub kind: HttpAuthKind,
    #[serde(default)]
    pub header_name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAuthProfile {
    pub id: String,
    pub revision: i64,
    #[serde(flatten)]
    pub input: HttpAuthInput,
    pub origin: String,
    pub expired: bool,
    pub credential_state: String,
    pub last_check: Option<Value>,
    pub cleanup_pending: bool,
}
pub(super) struct AuthRecord {
    pub profile: HttpAuthProfile,
    pub secret_ref: Option<String>,
}
type AuthRow = (
    String,
    i64,
    Option<String>,
    bool,
    Option<String>,
    Option<String>,
);
fn label(value: &str, max: usize) -> Result<String> {
    let text = validate_text(value, max, "Identity label")?;
    if super::super::reporting::redact_content(&text).0 != text {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Keep credentials out of identity labels",
        ));
    }
    Ok(text)
}
pub(super) fn auth_header(kind: HttpAuthKind, configured: &str) -> Result<String> {
    let name = match kind {
        HttpAuthKind::Anonymous => return Ok(String::new()),
        HttpAuthKind::Bearer | HttpAuthKind::Basic => "authorization",
        HttpAuthKind::Cookie => "cookie",
        HttpAuthKind::ApiKey | HttpAuthKind::CustomHeader => configured,
    };
    let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
        .map_err(|_| AppError::new("INVALID_HEADER", "Authentication header name is invalid"))?;
    if matches!(
        name.as_str(),
        "host"
            | "connection"
            | "content-length"
            | "transfer-encoding"
            | "expect"
            | "upgrade"
            | "trailer"
            | "te"
            | "proxy-authorization"
            | "proxy-connection"
            | "forwarded"
            | "x-forwarded-host"
            | "x-forwarded-for"
            | "content-type"
            | "accept-encoding"
    ) {
        return Err(AppError::new(
            "HEADER_POLICY",
            "Authentication cannot modify transport, routing or content headers",
        ));
    }
    Ok(name.as_str().into())
}
pub(super) fn credential_header(kind: HttpAuthKind, secret: &str) -> Result<Zeroizing<String>> {
    if secret.len() > 2048 || secret.chars().any(char::is_control) || secret.contains("[REDACTED]")
    {
        return Err(AppError::new(
            "INVALID_SECRET",
            "Credential exceeds limits, contains controls or is a redacted placeholder",
        ));
    }
    let value = match kind {
        HttpAuthKind::Anonymous => {
            return Err(AppError::new(
                "INVALID_SECRET",
                "Anonymous identities do not store credentials",
            ));
        }
        HttpAuthKind::Basic => {
            let (user, password) = secret.split_once(':').ok_or_else(|| {
                AppError::new("INVALID_SECRET", "Basic credentials use username:password")
            })?;
            if user.is_empty() || user.len() > 100 || password.len() < 8 {
                return Err(AppError::new(
                    "INVALID_SECRET",
                    "Basic username is required; use a password of at least eight characters",
                ));
            }
            format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode(secret.as_bytes())
            )
        }
        HttpAuthKind::Bearer => {
            if secret.len() < 8 || secret.chars().any(char::is_whitespace) {
                return Err(AppError::new(
                    "INVALID_SECRET",
                    "Bearer token needs at least eight characters and no whitespace",
                ));
            }
            format!("Bearer {secret}")
        }
        HttpAuthKind::Cookie | HttpAuthKind::ApiKey | HttpAuthKind::CustomHeader => {
            if secret.len() < 8 {
                return Err(AppError::new(
                    "INVALID_SECRET",
                    "Use a credential of at least eight characters",
                ));
            }
            if kind == HttpAuthKind::Cookie {
                let pairs: Vec<_> = secret.split(';').collect();
                if pairs.len() > 16
                    || pairs.iter().any(|p| {
                        p.split_once('=')
                            .is_none_or(|(n, v)| n.trim().is_empty() || v.trim().len() < 8)
                    })
                {
                    return Err(AppError::new(
                        "INVALID_SECRET",
                        "Cookie credentials use at most sixteen name=value pairs with values of at least eight characters",
                    ));
                }
            }
            secret.to_owned()
        }
    };
    reqwest::header::HeaderValue::from_str(&value).map_err(|_| {
        AppError::new(
            "INVALID_SECRET",
            "Credential cannot be used as an HTTP header",
        )
    })?;
    Ok(Zeroizing::new(value))
}
impl Core {
    fn auth_namespace(&self) -> String {
        let path =
            std::fs::canonicalize(self.db.path()).unwrap_or_else(|_| self.db.path().to_owned());
        let path = path.to_string_lossy();
        #[cfg(windows)]
        let path = path.to_lowercase();
        format!(
            "http-auth:{}:",
            super::super::evidence::digest(path.as_bytes())
        )
    }
    pub(super) fn auth_record(&self, id: &str) -> Result<AuthRecord> {
        let row:Option<AuthRow>=self.db.connect()?.query_row("SELECT data_json,revision,secret_ref,expired,last_check_json,target_id FROM http_auth_profiles WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(db_error)?;
        let (data, revision, secret_ref, expired, last, target) =
            row.ok_or_else(|| AppError::new("NOT_FOUND", "HTTP identity does not exist"))?;
        if secret_ref
            .as_ref()
            .is_some_and(|r| !r.starts_with(&self.auth_namespace()))
        {
            return Err(AppError::new(
                "CREDENTIAL_SCOPE",
                "HTTP credential reference belongs to another workspace",
            ));
        }
        let (input, origin): (HttpAuthInput, String) =
            serde_json::from_str(&data).map_err(json_error)?;
        let state = if target.is_none() {
            "target_removed"
        } else if expired {
            "expired"
        } else if input.kind == HttpAuthKind::Anonymous {
            "not_required"
        } else if secret_ref.is_some() {
            "configured"
        } else {
            "missing"
        };
        Ok(AuthRecord {
            profile: HttpAuthProfile {
                id: id.into(),
                revision,
                input,
                origin,
                expired: expired || target.is_none(),
                credential_state: state.into(),
                last_check: last
                    .map(|s| serde_json::from_str(&s).map_err(json_error))
                    .transpose()?,
                cleanup_pending: false,
            },
            secret_ref,
        })
    }
    pub fn http_auth_profiles(
        &self,
        project: &str,
        target: Option<&str>,
    ) -> Result<Vec<HttpAuthProfile>> {
        let project = self.project(project)?.id;
        if let Some(id) = target
            && self.target(id)?.input.project_id != project
        {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Identity target belongs to another project",
            ));
        }
        let conn = self.db.connect()?;
        let mut q=conn.prepare("SELECT id FROM http_auth_profiles WHERE project_id=?1 AND (?2 IS NULL OR target_id=?2) ORDER BY id LIMIT 100").map_err(db_error)?;
        let ids = q
            .query_map(params![project, target], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)?;
        ids.iter()
            .map(|id| self.auth_record(id).map(|r| r.profile))
            .collect()
    }
    pub fn http_auth_save(
        &self,
        id: Option<&str>,
        mut input: HttpAuthInput,
        revision: Option<i64>,
        secret: Option<Zeroizing<String>>,
    ) -> Result<HttpAuthProfile> {
        input.project_id = self.project(&input.project_id)?.id;
        if self.project(&input.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project before editing identities",
            ));
        }
        let target = self.target(&input.target_id)?;
        if target.input.project_id != input.project_id {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Identity target belongs to another project",
            ));
        }
        let origin = super::replay::lab_url(&target.input.value)?
            .origin()
            .ascii_serialization();
        input.name = label(&input.name, 100)?;
        input.role = label(&input.role, 100)?;
        input.header_name = auth_header(input.kind, &input.header_name)?;
        if let Some(s) = secret.as_ref() {
            credential_header(input.kind, s)?;
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let old = id.map(|i| self.auth_record(i)).transpose()?;
        if let Some(old) = &old {
            if old.profile.input.project_id != input.project_id
                || old.profile.input.target_id != input.target_id
                || revision != Some(old.profile.revision)
            {
                return Err(AppError::new(
                    "STALE_REVISION",
                    "Identity changed or belongs to another target; reload it",
                ));
            }
            if old.profile.input.kind != input.kind
                && secret.is_none()
                && input.kind != HttpAuthKind::Anonymous
            {
                return Err(AppError::new(
                    "CREDENTIAL_REQUIRED",
                    "Changing authentication type requires a new credential",
                ));
            }
        }
        let id = id
            .map(str::to_owned)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let new_reference = secret
            .as_ref()
            .map(|_| format!("{}{}:{}", self.auth_namespace(), id, Uuid::new_v4()));
        let reference = if input.kind == HttpAuthKind::Anonymous {
            None
        } else {
            new_reference
                .clone()
                .or_else(|| old.as_ref().and_then(|o| o.secret_ref.clone()))
        };
        if input.kind != HttpAuthKind::Anonymous && reference.is_none() {
            return Err(AppError::new(
                "CREDENTIAL_REQUIRED",
                "Enter a credential before saving this identity",
            ));
        }
        if let (Some(reference), Some(s)) = (&new_reference, &secret) {
            self.secrets.set(reference, s)?;
        }
        let outcome = (|| {
            tx.execute("INSERT INTO http_auth_profiles(id,project_id,target_id,revision,data_json,secret_ref) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,data_json=excluded.data_json,secret_ref=excluded.secret_ref,expired=0,last_check_json=NULL",params![id,input.project_id,input.target_id,old.as_ref().map_or(1,|o|o.profile.revision+1),serde_json::to_string(&(&input,&origin)).map_err(json_error)?,reference]).map_err(db_error)?;
            if let Some(retired) = old.and_then(|o| o.secret_ref)
                && Some(&retired) != reference.as_ref()
            {
                tx.execute(
                    "INSERT OR IGNORE INTO http_auth_retired_secrets VALUES(?1)",
                    [retired],
                )
                .map_err(db_error)?;
            }
            Self::audit(&tx, "http.identity_saved", &id, "not_authenticated")?;
            tx.commit().map_err(db_error)
        })();
        if let Err(e) = outcome {
            if let Some(r) = new_reference {
                let _ = self.secrets.delete(&r);
            }
            return Err(e);
        }
        let pending = self.http_auth_cleanup().is_err();
        let mut profile = self.auth_record(&id)?.profile;
        profile.cleanup_pending = pending;
        Ok(profile)
    }
    pub fn http_auth_expire(&self, id: &str, revision: i64) -> Result<()> {
        let record = self.auth_record(id)?;
        if self.project(&record.profile.input.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project before changing identities",
            ));
        }
        let changed=self.db.connect()?.execute("UPDATE http_auth_profiles SET expired=1,revision=revision+1,last_check_json=NULL WHERE id=?1 AND revision=?2",params![id,revision]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new(
                "STALE_REVISION",
                "Identity changed; reload it",
            ));
        }
        Ok(())
    }
    pub fn http_auth_remove(&self, id: &str, revision: i64) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let record = self.auth_record(id)?;
        if record.profile.revision != revision {
            return Err(AppError::new(
                "STALE_REVISION",
                "Identity changed; reload it",
            ));
        }
        if self.project(&record.profile.input.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project before changing identities",
            ));
        }
        if let Some(reference) = record.secret_ref {
            tx.execute(
                "INSERT OR IGNORE INTO http_auth_retired_secrets VALUES(?1)",
                [reference],
            )
            .map_err(db_error)?;
        }
        tx.execute("DELETE FROM http_auth_profiles WHERE id=?1", [id])
            .map_err(db_error)?;
        Self::audit(&tx, "http.identity_removed", id, "completed")?;
        tx.commit().map_err(db_error)?;
        self.http_auth_cleanup()
    }
    /// Retry only retired HTTP credentials in this workspace's own namespace.
    pub fn http_auth_cleanup(&self) -> Result<()> {
        let conn = self.db.connect()?;
        let mut q = conn
            .prepare("SELECT secret_ref FROM http_auth_retired_secrets LIMIT 100")
            .map_err(db_error)?;
        let refs = q
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)?;
        for reference in refs {
            if !reference.starts_with(&self.auth_namespace()) {
                return Err(AppError::new(
                    "CREDENTIAL_SCOPE",
                    "Retired HTTP credential belongs to a different workspace",
                ));
            }
            self.secrets.delete(&reference)?;
            conn.execute(
                "DELETE FROM http_auth_retired_secrets WHERE secret_ref=?1",
                [reference],
            )
            .map_err(db_error)?;
        }
        Ok(())
    }
}
