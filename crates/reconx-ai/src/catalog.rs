//! Official API defaults and reviewed presets; account availability is never implied.
use crate::{ProviderConfig, ProviderKind};
use reconx_security::{AppError, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    pub tool_calling: Option<bool>,
    pub vision: Option<bool>,
    pub reasoning: Option<bool>,
    pub streaming: Option<bool>,
    pub structured_output: Option<bool>,
    pub context_window: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: ModelCapabilities,
    pub deprecated: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreset {
    pub id: String,
    pub name: String,
    pub category: String,
    pub kind: ProviderKind,
    pub base_url: String,
    #[serde(default)]
    pub endpoint_template: Option<String>,
    pub requires_key: bool,
    pub discovery: bool,
    pub setup_note: String,
    pub docs_url: String,
    pub verified_on: String,
    pub models: Vec<ModelPreset>,
}
pub fn provider_catalog() -> Vec<ProviderPreset> {
    serde_json::from_str(include_str!("provider-catalog.json"))
        .expect("compiled catalog validated by tests")
}
pub fn provider_preset(id: &str) -> Result<ProviderPreset> {
    provider_catalog()
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::new("UNKNOWN_PROVIDER", "Choose a provider from the catalog"))
}
impl ProviderPreset {
    pub fn configuration(&self, name: &str, model: &str) -> ProviderConfig {
        ProviderConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            kind: self.kind,
            base_url: self.base_url.clone(),
            model: model.into(),
            enabled: true,
            timeout_secs: 120,
            max_output_tokens: 4096,
            requires_key: self.requires_key,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_complete_unique_and_uses_native_protocols() {
        let catalog = provider_catalog();
        assert_eq!(catalog.len(), 24);
        let mut ids = std::collections::HashSet::new();
        for p in catalog {
            assert!(ids.insert(p.id.clone()));
            assert!(p.docs_url.starts_with("https://"));
            for m in &p.models {
                let mut c = p.configuration("Catalog test", &m.id);
                if let Some(t) = &p.endpoint_template {
                    c.base_url = t.replace("{workspace}", "fixture");
                }
                c.validate().unwrap();
            }
        }
        assert_eq!(
            provider_preset("openai").unwrap().kind,
            ProviderKind::Openai
        );
        assert_eq!(
            provider_preset("anthropic").unwrap().kind,
            ProviderKind::Anthropic
        );
        assert_eq!(
            provider_preset("google").unwrap().kind,
            ProviderKind::Gemini
        );
        assert!(!provider_preset("ollama").unwrap().requires_key);
    }
}
