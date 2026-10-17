use reconx_security::{AppError, Result, validate_gateway};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct GatewayOptions {
    /// None selects the native protocol's authentication header.
    pub auth_header: Option<String>,
    pub auth_prefix: String,
    pub proxy_url: Option<String>,
    /// Relative model-list path only; never changes the credential destination.
    pub discovery_path: Option<String>,
    pub streaming: bool,
}
impl Default for GatewayOptions {
    fn default() -> Self {
        Self {
            auth_header: None,
            auth_prefix: "Bearer ".into(),
            proxy_url: None,
            discovery_path: None,
            streaming: true,
        }
    }
}
pub fn validate_header(name: &str, value: &str) -> Result<()> {
    if name.len() > 100
        || value.len() > 2048
        || value.chars().any(char::is_control)
        || matches!(
            name.to_ascii_lowercase().as_str(),
            "host"
                | "content-length"
                | "connection"
                | "transfer-encoding"
                | "cookie"
                | "proxy-authorization"
        )
        || reqwest::header::HeaderName::from_bytes(name.as_bytes()).is_err()
        || reqwest::header::HeaderValue::from_str(value).is_err()
    {
        return Err(AppError::new(
            "INVALID_HEADER",
            "Invalid or prohibited custom HTTP header",
        ));
    }
    Ok(())
}
impl GatewayOptions {
    pub fn validate(&self) -> Result<()> {
        if let Some(name) = &self.auth_header {
            validate_header(name, "synthetic")?;
        }
        if !["", "Bearer ", "Token "].contains(&self.auth_prefix.as_str()) {
            return Err(AppError::new(
                "INVALID_AUTH",
                "Choose no prefix, Bearer, or Token",
            ));
        }
        if let Some(url) = &self.proxy_url {
            validate_gateway(url)?;
        }
        if let Some(path) = &self.discovery_path
            && (path.is_empty()
                || path.len() > 200
                || path.contains("..")
                || path.starts_with('/')
                || !path
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_/".contains(&b)))
        {
            return Err(AppError::new(
                "INVALID_DISCOVERY_PATH",
                "Use a relative API path without query parameters or traversal",
            ));
        }
        Ok(())
    }
}
