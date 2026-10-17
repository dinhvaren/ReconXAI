//! Shared provider profiles. Normal SQLite records never contain header values or API keys.
use super::*;
pub use reconx_ai::{GatewayOptions, ModelCapabilities, ModelPreset, ProviderPreset};
use std::collections::{BTreeMap, HashSet};
pub const AI_AGENT_IDS: [&str; 8] = [
    "orchestrator",
    "recon",
    "web",
    "api",
    "reverse",
    "code_audit",
    "evidence",
    "report",
];
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConnection {
    pub state: String,
    pub checked_at: String,
    pub last_success_at: Option<String>,
    pub latency_ms: u64,
    pub error_code: Option<String>,
    pub message: String,
    pub fingerprint: String,
}
#[cfg(test)]
mod privacy_tests {
    use super::*;
    #[tokio::test]
    async fn evidence_cannot_silently_follow_a_configured_chat_proxy() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::open(Some(dir.path().join("private.db"))).unwrap();
        let config = reconx_ai::provider_preset("lmstudio")
            .unwrap()
            .configuration("Private model", "fixture");
        core.save_ai_setup(
            AiProfileInput {
                provider: config.clone(),
                vendor_id: "lmstudio".into(),
                make_default: false,
                options: GatewayOptions {
                    proxy_url: Some("http://127.0.0.1:1".into()),
                    ..Default::default()
                },
            },
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            core.chat_artifacts_to_gateway(config, "Private artifact fixture")
                .await
                .unwrap_err()
                .code,
            "ARTIFACT_PROXY_APPROVAL_REQUIRED"
        );
    }
}
#[derive(Serialize)]
pub struct AiProfile {
    pub provider: ProviderConfig,
    pub vendor_id: String,
    pub options: GatewayOptions,
    pub header_names: Vec<String>,
    pub credential_state: String,
    pub key_configured: bool,
    pub connection: Option<AiConnection>,
    pub models: Vec<String>,
    pub models_refreshed_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRoute {
    pub agent_id: String,
    pub provider_id: String,
}
#[derive(Serialize)]
pub struct AiManagerState {
    pub catalog: Vec<ProviderPreset>,
    pub profiles: Vec<AiProfile>,
    pub routing: Vec<AgentRoute>,
    pub agents: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiProfileInput {
    pub provider: ProviderConfig,
    pub vendor_id: String,
    pub make_default: bool,
    #[serde(default)]
    pub options: GatewayOptions,
}
type Metadata = (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    String,
);
fn signature(config: &ProviderConfig, options: &GatewayOptions) -> Result<String> {
    Ok(
        Sha256::digest(serde_json::to_vec(&(config, options)).map_err(json_error)?)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}
fn headers_reference(config: &ProviderConfig) -> String {
    format!("{}:headers", credential_reference(config))
}
impl Core {
    pub fn ai_catalog(&self) -> Vec<ProviderPreset> {
        reconx_ai::provider_catalog()
    }
    pub fn ai_options(&self, id: &str) -> Result<GatewayOptions> {
        let raw: Option<String> = self
            .db
            .connect()?
            .query_row(
                "SELECT options_json FROM ai_profile_metadata WHERE provider_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        raw.map(|s| serde_json::from_str(&s).map_err(json_error))
            .transpose()
            .map(|o| o.unwrap_or_default())
    }
    pub(crate) fn ai_gateway(&self, config: ProviderConfig) -> Result<reconx_ai::Gateway> {
        let options = self.ai_options(&config.id)?;
        let names: Option<String> = self
            .db
            .connect()?
            .query_row(
                "SELECT header_names_json FROM ai_profile_metadata WHERE provider_id=?1",
                [&config.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        let headers = if names.as_deref().is_some_and(|s| s != "[]") {
            let raw = self
                .secrets
                .get(&headers_reference(&config))?
                .ok_or_else(|| {
                    AppError::new(
                        "MISSING_HEADER_CREDENTIAL",
                        "Re-enter the custom gateway headers in Settings",
                    )
                })?;
            serde_json::from_str(&raw).map_err(json_error)?
        } else {
            BTreeMap::new()
        };
        reconx_ai::Gateway::with_options(config, options, headers)
    }
    pub fn ai_manager(&self) -> Result<AiManagerState> {
        let conn = self.db.connect()?;
        let mut profiles = Vec::new();
        for provider in self.providers()? {
            let row:Option<Metadata>=conn.query_row("SELECT vendor_id,models_json,models_fingerprint,models_refreshed_at,connection_json,options_json,header_names_json FROM ai_profile_metadata WHERE provider_id=?1",[&provider.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).optional().map_err(db_error)?;
            let (vendor_id, models, models_fp, models_refreshed_at, health, options, header_names) =
                row.unwrap_or_else(|| {
                    let vendor = self
                        .ai_catalog()
                        .into_iter()
                        .find(|p| p.kind == provider.kind && p.base_url == provider.base_url)
                        .map(|p| p.id)
                        .unwrap_or_else(|| "custom".into());
                    (
                        vendor,
                        "[]".into(),
                        None,
                        None,
                        None,
                        "{}".into(),
                        "[]".into(),
                    )
                });
            let options: GatewayOptions = serde_json::from_str(&options).map_err(json_error)?;
            let fingerprint = signature(&provider, &options)?;
            let connection = health
                .map(|s| serde_json::from_str::<AiConnection>(&s).map_err(json_error))
                .transpose()?
                .filter(|h| h.fingerprint == fingerprint);
            let mut credential_state = if !provider.requires_key {
                "not_required"
            } else {
                match self.credential_status(&provider.id) {
                    Ok(true) => "stored",
                    Ok(false) => "missing",
                    Err(_) => "store_unavailable",
                }
            }
            .to_owned();
            let key_configured = credential_state == "stored";
            let header_names: Vec<String> =
                serde_json::from_str(&header_names).map_err(json_error)?;
            if !header_names.is_empty() {
                match self.secrets.get(&headers_reference(&provider)) {
                    Ok(Some(_)) => {}
                    Ok(None) => credential_state = "missing_headers".into(),
                    Err(_) => credential_state = "store_unavailable".into(),
                }
            }
            profiles.push(AiProfile {
                provider,
                vendor_id,
                options,
                header_names,
                credential_state,
                key_configured,
                connection,
                models: if models_fp.as_deref() == Some(&fingerprint) {
                    serde_json::from_str(&models).map_err(json_error)?
                } else {
                    Vec::new()
                },
                models_refreshed_at,
            });
        }
        Ok(AiManagerState {
            catalog: self.ai_catalog(),
            profiles,
            routing: self.ai_routing()?,
            agents: AI_AGENT_IDS.iter().map(|s| s.to_string()).collect(),
        })
    }
    pub fn save_ai_profile(
        &self,
        config: &ProviderConfig,
        vendor_id: &str,
        key: Option<&str>,
        make_default: bool,
    ) -> Result<()> {
        self.save_ai_setup(
            AiProfileInput {
                provider: config.clone(),
                vendor_id: vendor_id.into(),
                make_default,
                options: self.ai_options(&config.id)?,
            },
            key,
            None,
        )
    }
    /// Config/key/header updates roll back on secure-store failure. Compensation
    /// restores old credential entries if SQLite cannot commit.
    pub fn save_ai_setup(
        &self,
        input: AiProfileInput,
        key: Option<&str>,
        headers: Option<&BTreeMap<String, String>>,
    ) -> Result<()> {
        let AiProfileInput {
            provider: config,
            vendor_id,
            make_default,
            options,
        } = input;
        config.validate()?;
        options.validate()?;
        reconx_ai::provider_preset(&vendor_id)?;
        if make_default && !config.enabled {
            return Err(AppError::new(
                "PROVIDER_DISABLED",
                "Enable a profile before making it default",
            ));
        }
        if let Ok(old) = self.provider(&config.id)
            && (old.base_url != config.base_url || old.kind != config.kind)
            && (self.credential_status(&old.id)?
                || self.secrets.get(&headers_reference(&old))?.is_some())
        {
            return Err(AppError::new(
                "GATEWAY_CREDENTIAL_BOUND",
                "Remove the stored API key and custom headers before changing endpoint or protocol",
            ));
        }
        if key.is_some_and(|k| {
            k.trim().is_empty() || k.len() > 8192 || k.chars().any(char::is_control)
        }) {
            return Err(AppError::new(
                "INVALID_SECRET",
                "Enter a non-empty API key without control characters",
            ));
        }
        let reference = credential_reference(&config);
        let href = headers_reference(&config);
        let previous_key = if config.requires_key || key.is_some() {
            self.secrets.get(&reference)?
        } else {
            None
        };
        if config.requires_key && key.is_none() && previous_key.is_none() {
            return Err(AppError::new(
                "MISSING_CREDENTIAL",
                "Paste an API key before saving this profile",
            ));
        }
        let header_json = if let Some(headers) = headers {
            if headers.len() > 8 {
                return Err(AppError::new(
                    "HEADER_LIMIT",
                    "At most eight custom headers",
                ));
            }
            for (name, value) in headers {
                reconx_ai::validate_header(name, value)?;
            }
            let raw = Zeroizing::new(serde_json::to_string(headers).map_err(json_error)?);
            if raw.len() > 8192 {
                return Err(AppError::new(
                    "HEADER_LIMIT",
                    "Custom headers exceed secure storage limits",
                ));
            }
            Some(raw)
        } else {
            None
        };
        let previous_headers = if headers.is_some() {
            self.secrets.get(&href)?
        } else {
            None
        };
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO providers(id,name,json) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=excluded.name,json=excluded.json",params![config.id,config.name,serde_json::to_string(&config).map_err(json_error)?]).map_err(duplicate_error)?;
        tx.execute("INSERT INTO ai_profile_metadata(provider_id,vendor_id,options_json) VALUES(?1,?2,?3) ON CONFLICT(provider_id) DO UPDATE SET vendor_id=excluded.vendor_id,options_json=excluded.options_json,connection_json=NULL,revision=revision+1",params![config.id,vendor_id,serde_json::to_string(&options).map_err(json_error)?]).map_err(db_error)?;
        if let Some(headers) = headers {
            tx.execute(
                "UPDATE ai_profile_metadata SET header_names_json=?2 WHERE provider_id=?1",
                params![
                    config.id,
                    serde_json::to_string(&headers.keys().collect::<Vec<_>>())
                        .map_err(json_error)?
                ],
            )
            .map_err(db_error)?;
        }
        if make_default {
            let mut settings = self.settings()?;
            settings.default_provider_id = Some(config.id.clone());
            if settings.fallback_provider_id.as_deref() == Some(&config.id) {
                settings.fallback_provider_id = None;
            }
            tx.execute("INSERT INTO settings(id,json) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",[serde_json::to_string(&settings).map_err(json_error)?]).map_err(db_error)?;
        }
        Self::audit(&tx, "provider.profile_saved", &config.id, "completed")?;
        if let Some(key) = key {
            self.secrets.set(&reference, key)?;
        }
        if let Some(raw) = &header_json {
            let result = if headers.is_some_and(|h| h.is_empty()) {
                self.secrets.delete(&href)
            } else {
                self.secrets.set(&href, raw)
            };
            if let Err(error) = result {
                if key.is_some() {
                    self.restore_ai_secret(
                        &reference,
                        previous_key.as_deref().map(|s| s.as_str()),
                    )?;
                }
                return Err(error);
            }
        }
        if let Err(error) = tx.commit() {
            if key.is_some() {
                self.restore_ai_secret(&reference, previous_key.as_deref().map(|s| s.as_str()))?;
            }
            if headers.is_some() {
                self.restore_ai_secret(&href, previous_headers.as_deref().map(|s| s.as_str()))?;
            }
            return Err(db_error(error));
        }
        Ok(())
    }
    fn restore_ai_secret(&self, id: &str, value: Option<&str>) -> Result<()> {
        match value {
            Some(v) => self.secrets.set(id, v),
            None => self.secrets.delete(id),
        }
    }
    pub fn invalidate_ai_connection(&self, id: &str) -> Result<()> {
        self.db.connect()?.execute("INSERT INTO ai_profile_metadata(provider_id,revision) VALUES(?1,1) ON CONFLICT(provider_id) DO UPDATE SET connection_json=NULL,models_json='[]',models_fingerprint=NULL,models_refreshed_at=NULL,revision=revision+1",[id]).map_err(db_error)?;
        Ok(())
    }
    fn ai_revision(&self, id: &str) -> Result<i64> {
        Ok(self
            .db
            .connect()?
            .query_row(
                "SELECT revision FROM ai_profile_metadata WHERE provider_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?
            .unwrap_or(0))
    }
    pub async fn test_ai_connection(&self, id: &str) -> Result<AiConnection> {
        let mut config = self.provider(id)?;
        let options = self.ai_options(id)?;
        let fingerprint = signature(&config, &options)?;
        let revision = self.ai_revision(id)?;
        let previous = self
            .ai_manager()?
            .profiles
            .into_iter()
            .find(|p| p.provider.id == id)
            .and_then(|p| p.connection)
            .and_then(|h| h.last_success_at);
        config.max_output_tokens = config.max_output_tokens.clamp(1024, 4096);
        let start = std::time::Instant::now();
        let key = if config.requires_key {
            self.secrets.get(&credential_reference(&config))?
        } else {
            None
        };
        let result = self
            .ai_gateway(config)?
            .chat(
                "Connection check only. Reply with OK. Do not use tools.",
                key.as_ref().map(|k| k.as_str()),
            )
            .await;
        let time = now();
        let (state, error_code, message, last_success_at) = match result {
            Ok(_) => (
                "connected",
                None,
                "Selected model returned a valid inference response".into(),
                Some(time.clone()),
            ),
            Err(e) => ("failed", Some(e.code), e.message, previous),
        };
        let health = AiConnection {
            state: state.into(),
            checked_at: time,
            last_success_at,
            latency_ms: start.elapsed().as_millis() as u64,
            error_code,
            message,
            fingerprint,
        };
        if signature(&self.provider(id)?, &self.ai_options(id)?)? != health.fingerprint {
            return Err(AppError::new(
                "STALE_CONFIGURATION",
                "Provider changed during test",
            ));
        }
        let conn = self.db.connect()?;
        let changed=conn.execute("INSERT INTO ai_profile_metadata(provider_id,connection_json) VALUES(?1,?2) ON CONFLICT(provider_id) DO UPDATE SET connection_json=excluded.connection_json WHERE revision=?3",params![id,serde_json::to_string(&health).map_err(json_error)?,revision]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new(
                "STALE_CONFIGURATION",
                "Credential changed during test; test again",
            ));
        }
        Self::audit(&conn, "ai.connection_test", id, &health.state)?;
        Ok(health)
    }
    pub async fn refresh_ai_models(&self, id: &str) -> Result<Vec<String>> {
        let config = self.provider(id)?;
        let fingerprint = signature(&config, &self.ai_options(id)?)?;
        let revision = self.ai_revision(id)?;
        let models = self.models(id).await?;
        if signature(&self.provider(id)?, &self.ai_options(id)?)? != fingerprint {
            return Err(AppError::new(
                "STALE_CONFIGURATION",
                "Provider changed during discovery",
            ));
        }
        let changed=self.db.connect()?.execute("INSERT INTO ai_profile_metadata(provider_id,models_json,models_fingerprint,models_refreshed_at) VALUES(?1,?2,?3,?4) ON CONFLICT(provider_id) DO UPDATE SET models_json=excluded.models_json,models_fingerprint=excluded.models_fingerprint,models_refreshed_at=excluded.models_refreshed_at WHERE revision=?5",params![id,serde_json::to_string(&models).map_err(json_error)?,fingerprint,now(),revision]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new(
                "STALE_CONFIGURATION",
                "Credential changed during discovery",
            ));
        }
        Ok(models)
    }
    pub async fn discover_ai_models(
        &self,
        config: ProviderConfig,
        key: Option<&str>,
        options: GatewayOptions,
    ) -> Result<Vec<String>> {
        let saved_key = if key.is_none() && config.requires_key {
            let old = self.provider(&config.id)?;
            if old.base_url != config.base_url || old.kind != config.kind {
                return Err(AppError::new(
                    "GATEWAY_CREDENTIAL_BOUND",
                    "Enter a key explicitly for this new destination",
                ));
            }
            self.secrets.get(&credential_reference(&old))?
        } else {
            None
        };
        reconx_ai::Gateway::with_options(config, options, BTreeMap::new())?
            .models(key.or(saved_key.as_ref().map(|k| k.as_str())))
            .await
    }
    pub fn ai_routing(&self) -> Result<Vec<AgentRoute>> {
        let conn = self.db.connect()?;
        let mut q = conn
            .prepare("SELECT agent_id,provider_id FROM ai_agent_routing ORDER BY agent_id")
            .map_err(db_error)?;
        q.query_map([], |r| {
            Ok(AgentRoute {
                agent_id: r.get(0)?,
                provider_id: r.get(1)?,
            })
        })
        .map_err(db_error)?
        .collect::<std::result::Result<_, _>>()
        .map_err(db_error)
    }
    pub fn save_ai_routing(&self, routes: &[AgentRoute]) -> Result<()> {
        if routes.len() > AI_AGENT_IDS.len() {
            return Err(AppError::new("INVALID_ROUTING", "Too many agent overrides"));
        }
        let mut seen = HashSet::new();
        for route in routes {
            if !AI_AGENT_IDS.contains(&route.agent_id.as_str()) || !seen.insert(&route.agent_id) {
                return Err(AppError::new(
                    "INVALID_ROUTING",
                    "Unknown or duplicate agent",
                ));
            }
            let c = self.provider(&route.provider_id)?;
            if !c.enabled || (c.requires_key && !self.credential_status(&c.id)?) {
                return Err(AppError::new(
                    "MODEL_UNAVAILABLE",
                    "Enable the model and configure its credential first",
                ));
            }
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("DELETE FROM ai_agent_routing", [])
            .map_err(db_error)?;
        for r in routes {
            tx.execute(
                "INSERT INTO ai_agent_routing(agent_id,provider_id) VALUES(?1,?2)",
                params![r.agent_id, r.provider_id],
            )
            .map_err(db_error)?;
        }
        Self::audit(&tx, "ai.routing_saved", "workspace", "completed")?;
        tx.commit().map_err(db_error)
    }
    pub fn ai_agent_provider(&self, agent: &str) -> Result<ProviderConfig> {
        if !AI_AGENT_IDS.contains(&agent) {
            return Err(AppError::new("INVALID_ROUTING", "Unknown agent"));
        }
        let id = self
            .ai_routing()?
            .into_iter()
            .find(|r| r.agent_id == agent)
            .map(|r| r.provider_id)
            .or(self.settings()?.default_provider_id)
            .ok_or_else(|| AppError::new("MODEL_UNAVAILABLE", "Configure a default AI model"))?;
        let c = self.provider(&id)?;
        if !c.enabled {
            return Err(AppError::new(
                "MODEL_UNAVAILABLE",
                "The selected model is disabled",
            ));
        }
        Ok(c)
    }
}
