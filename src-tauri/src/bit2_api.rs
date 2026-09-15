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

pub async fn login(origin: &str, username: &str, password: &str) -> Result<Bit2Session, String> {
    let url = format!("{}/api/user/login", normalize_origin(origin)?);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).build().map_err(|e| e.to_string())?;
    let response = client.post(url).json(&serde_json::json!({"username": username, "password": password})).send().await.map_err(|_| "bit2.ai 登录请求失败".to_string())?;
    if !response.status().is_success() { return Err(format!("bit2.ai 登录失败（HTTP {}）", response.status())); }
    let token = response.headers().get("Authorization").and_then(|v| v.to_str().ok()).unwrap_or("").trim_start_matches("Bearer ").to_string();
    if token.is_empty() { return Err("登录成功但服务端未返回访问令牌，请使用网页登录".into()); }
    Ok(Bit2Session { access_token: token, refresh_token: None, expires_at: None })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn normalizes_origin() { assert_eq!(normalize_origin("https://bit2.ai/").unwrap(), "https://bit2.ai"); }
    #[test] fn derives_codex_url() { assert_eq!(codex_api_url("https://bit2.ai").unwrap(), "https://bit2.ai/v1"); }
    #[test] fn rejects_http() { assert!(normalize_origin("http://bit2.ai").is_err()); }
}
