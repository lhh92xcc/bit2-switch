use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bit2Session {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexCredential {
    pub api_key: String,
    pub api_url: String,
}

pub fn normalize_origin(input: &str) -> Result<String, String> {
    let trimmed = input.trim().trim_end_matches('/');
    let parsed = url::Url::parse(trimmed).map_err(|e| format!("invalid bit2.ai URL: {e}"))?;
    if parsed.scheme() != "https" && parsed.host_str() != Some("localhost") {
        return Err("bit2.ai origin must use HTTPS".into());
    }
    Ok(trimmed.to_string())
}

pub fn codex_api_url(origin: &str) -> Result<String, String> {
    Ok(format!("{}/v1", normalize_origin(origin)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn normalizes_origin() { assert_eq!(normalize_origin("https://bit2.ai/").unwrap(), "https://bit2.ai"); }
    #[test] fn derives_codex_url() { assert_eq!(codex_api_url("https://bit2.ai").unwrap(), "https://bit2.ai/v1"); }
    #[test] fn rejects_http() { assert!(normalize_origin("http://bit2.ai").is_err()); }
}
