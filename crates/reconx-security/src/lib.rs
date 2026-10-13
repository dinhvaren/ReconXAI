use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, thiserror::Error, Serialize)]
#[error("{code}: {message}")]
pub struct AppError {
    pub code: String,
    pub message: String,
}
impl AppError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityMode {
    #[default]
    PassiveAuto,
    AuthorizedAuto,
    LabResearch,
}

pub fn validate_text(value: &str, max: usize, field: &str) -> Result<String> {
    let text = value.trim();
    if text.is_empty()
        || text.len() > max
        || text
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(AppError::new(
            "INVALID_INPUT",
            &format!("{field} must contain 1–{max} bytes of valid text"),
        ));
    }
    Ok(text.to_owned())
}

/// Gateway URLs are user-configured destinations, never derived from model output.
pub fn validate_gateway(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).map_err(|_| {
        AppError::new(
            "INVALID_GATEWAY",
            "A valid absolute gateway URL is required",
        )
    })?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !(url.scheme() == "https" || (url.scheme() == "http" && local))
    {
        return Err(AppError::new(
            "INVALID_GATEWAY",
            "Use HTTPS, or loopback HTTP; credentials, query parameters and fragments are prohibited",
        ));
    }
    Ok(url)
}

/// No execution capability is exposed by this foundation, regardless of mode.
pub fn require_local_analysis(action: &str) -> Result<()> {
    if matches!(action, "project" | "settings" | "local_analysis") {
        Ok(())
    } else {
        Err(AppError::new(
            "BLOCKED_BY_POLICY",
            "This build only supports local analysis; external tool execution is unavailable",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gateway_rejects_credentials_and_insecure_remote() {
        for url in [
            "http://example.com/v1",
            "https://user:secret@example.com",
            "https://example.com?key=abc",
            "file:///tmp",
            "https://example.com/#key",
        ] {
            assert!(validate_gateway(url).is_err(), "{url}");
        }
        for url in [
            "https://example.com/v1",
            "http://127.0.0.1:11434",
            "http://[::1]:8080/v1",
        ] {
            assert!(validate_gateway(url).is_ok());
        }
    }
    #[test]
    fn no_mode_unlocks_shell_execution() {
        assert!(require_local_analysis("shell").is_err());
    }
}
