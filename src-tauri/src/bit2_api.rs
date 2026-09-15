use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use futures::StreamExt;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::app_config::AppType;
use crate::provider::Provider;
use crate::store::AppState;

const CLIENT_ID: &str = "bit2-switch";
const REDIRECT_URI: &str = "bit2switch://auth";
const ORIGIN: &str = "https://bit2.ai";
const BASE_URL: &str = "https://bit2.ai/v1";
const KEYCHAIN_SERVICE: &str = "bit2-switch";
const KEYCHAIN_ACCOUNT: &str = "codex-api-key";
const KEYCHAIN_REFERENCE: &str = "keychain://bit2-switch/codex-api-key";
const PROVIDER_ID: &str = "bit2-managed-codex";
const PENDING_TTL: Duration = Duration::from_secs(10 * 60);
const MAX_RESPONSE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Bit2Status {
    pub connected: bool,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthCallback {
    code: String,
    state: String,
}

struct PendingAuth {
    state: String,
    verifier: String,
    created_at: Instant,
}

#[derive(Default)]
struct AuthState {
    pending: Option<PendingAuth>,
    active: Option<String>,
}

impl AuthState {
    fn cancel(&mut self) {
        self.pending = None;
        self.active = None;
    }

    fn take_verifier(&mut self, state: &str) -> Result<String, String> {
        if !self
            .pending
            .as_ref()
            .is_some_and(|flow| flow.created_at.elapsed() <= PENDING_TTL && flow.state == state)
        {
            return Err("Authentication state is invalid or expired".into());
        }
        let flow = self
            .pending
            .take()
            .expect("validated pending authentication");
        self.active = Some(flow.state);
        Ok(flow.verifier)
    }

    fn ensure_active(&self, state: &str) -> Result<(), String> {
        if self.active.as_deref() == Some(state) {
            Ok(())
        } else {
            Err("Authentication was cancelled".into())
        }
    }
}

static PENDING_AUTH: OnceLock<Mutex<AuthState>> = OnceLock::new();

fn pending_auth() -> &'static Mutex<AuthState> {
    PENDING_AUTH.get_or_init(|| Mutex::new(AuthState::default()))
}

fn parse_auth_callback(raw: &str) -> Result<AuthCallback, String> {
    let url = url::Url::parse(raw).map_err(|_| "Invalid authentication callback".to_string())?;
    if url.scheme() != "bit2switch"
        || url.host_str() != Some("auth")
        || url.path() != ""
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return Err("Invalid authentication callback".into());
    }

    let mut code = None;
    let mut state = None;
    for (key, value) in url.query_pairs() {
        let slot = match key.as_ref() {
            "code" => &mut code,
            "state" => &mut state,
            _ => return Err("Invalid authentication callback".into()),
        };
        if slot.is_some() || value.is_empty() {
            return Err("Invalid authentication callback".into());
        }
        *slot = Some(value.into_owned());
    }
    Ok(AuthCallback {
        code: code.ok_or_else(|| "Invalid authentication callback".to_string())?,
        state: state.ok_or_else(|| "Invalid authentication callback".to_string())?,
    })
}

fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn random_base64url(bytes: usize) -> String {
    let mut value = vec![0_u8; bytes];
    rand::rngs::OsRng.fill_bytes(&mut value);
    URL_SAFE_NO_PAD.encode(value)
}

fn authorize_url(state: &str, challenge: &str) -> Result<url::Url, String> {
    let mut url = url::Url::parse("https://bit2.ai/desktop/authorize")
        .map_err(|_| "Unable to start authentication".to_string())?;
    url.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("state", state)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256");
    Ok(url)
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    client_id: &'a str,
    redirect_uri: &'a str,
    code: &'a str,
    code_verifier: &'a str,
}

#[derive(Deserialize)]
struct TokenEnvelope {
    success: bool,
    data: Option<TokenData>,
}

#[derive(Deserialize)]
struct TokenData {
    api_key: String,
    base_url: String,
    #[allow(dead_code)]
    token_id: i64,
    token_name: String,
}

async fn exchange_code(code: &str, verifier: &str) -> Result<TokenData, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|_| "Authentication service is unavailable".to_string())?;
    let response = client
        .post("https://bit2.ai/api/desktop/token")
        .json(&TokenRequest {
            client_id: CLIENT_ID,
            redirect_uri: REDIRECT_URI,
            code,
            code_verifier: verifier,
        })
        .send()
        .await
        .map_err(|_| "Authentication request failed".to_string())?;
    if !response.status().is_success() {
        return Err("Authentication was rejected".into());
    }
    if response
        .content_length()
        .is_some_and(|size| size > MAX_RESPONSE_BYTES as u64)
    {
        return Err("Authentication response was invalid".into());
    }
    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "Authentication response was invalid".to_string())?;
        if body.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err("Authentication response was invalid".into());
        }
        body.extend_from_slice(&chunk);
    }
    let envelope: TokenEnvelope = serde_json::from_slice(&body)
        .map_err(|_| "Authentication response was invalid".to_string())?;
    let data = envelope
        .data
        .ok_or_else(|| "Authentication was rejected".to_string())?;
    if !envelope.success || !data.api_key.starts_with("sk-") || data.base_url != BASE_URL {
        return Err("Authentication response was invalid".into());
    }
    Ok(data)
}

#[cfg(target_os = "macos")]
fn write_keychain_secret(account: &str, secret: &[u8]) -> Result<(), String> {
    use security_framework::passwords::set_generic_password;
    // The native API updates existing entries atomically on errSecDuplicateItem.
    set_generic_password(KEYCHAIN_SERVICE, account, secret)
        .map_err(|_| "Unable to save credential in Keychain".to_string())
}

#[cfg(target_os = "macos")]
fn read_keychain_secret(account: &str) -> Result<Option<Vec<u8>>, String> {
    use security_framework::passwords::get_generic_password;
    match get_generic_password(KEYCHAIN_SERVICE, account) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.code() == -25300 => Ok(None),
        Err(_) => Err("Unable to read credential from Keychain".into()),
    }
}

#[cfg(target_os = "macos")]
fn delete_keychain_secret(account: &str) -> Result<(), String> {
    use security_framework::passwords::delete_generic_password;
    match delete_generic_password(KEYCHAIN_SERVICE, account) {
        Ok(()) => Ok(()),
        Err(error) if error.code() == -25300 => Ok(()),
        Err(_) => Err("Unable to remove credential from Keychain".into()),
    }
}

#[cfg(not(target_os = "macos"))]
fn read_keychain_secret(_account: &str) -> Result<Option<Vec<u8>>, String> {
    Ok(None)
}

#[cfg(not(target_os = "macos"))]
fn delete_keychain_secret(_account: &str) -> Result<(), String> {
    Ok(())
}

pub(crate) fn load_codex_api_key() -> Result<String, String> {
    let bytes = read_keychain_secret(KEYCHAIN_ACCOUNT)?
        .ok_or_else(|| "bit2.ai is not connected".to_string())?;
    String::from_utf8(bytes).map_err(|_| "Stored bit2.ai credential is invalid".to_string())
}

fn validate_launch_account(account: &str) -> Result<(), String> {
    if account
        .strip_prefix("bit2-launch-")
        .is_some_and(|suffix| uuid::Uuid::parse_str(suffix).is_ok())
    {
        Ok(())
    } else {
        Err("Invalid temporary credential account".into())
    }
}

pub(crate) fn store_launch_api_key(account: &str, key: &str) -> Result<(), String> {
    validate_launch_account(account)?;
    if key.is_empty() {
        return Err("Credential is empty".into());
    }
    #[cfg(target_os = "macos")]
    return write_keychain_secret(account, key.as_bytes());
    #[cfg(not(target_os = "macos"))]
    Err("Secure credential storage is unavailable".into())
}

pub(crate) fn delete_launch_api_key(account: &str) -> Result<(), String> {
    validate_launch_account(account)?;
    delete_keychain_secret(account)
}

pub(crate) fn provider_keychain_account(value: &str) -> Result<Option<String>, String> {
    if value == KEYCHAIN_REFERENCE {
        return Ok(Some(KEYCHAIN_ACCOUNT.into()));
    }
    if let Some(account) = value.strip_prefix("bit2-keychain://") {
        validate_setup_account(account)?;
        return Ok(Some(account.into()));
    }
    if value.contains("://") {
        return Err("Invalid credential reference".into());
    }
    Ok(None)
}

fn validate_setup_account(account: &str) -> Result<(), String> {
    if account.strip_prefix("codex-").is_some_and(|id| uuid::Uuid::parse_str(id).is_ok()) {
        Ok(())
    } else {
        Err("Invalid Codex credential account".into())
    }
}

pub(crate) fn store_setup_api_key(account: &str, key: &str) -> Result<(), String> {
    validate_setup_account(account)?;
    if key.trim().is_empty() || key.chars().any(char::is_control) {
        return Err("API key is empty or invalid".into());
    }
    #[cfg(target_os = "macos")]
    return write_keychain_secret(account, key.as_bytes());
    #[cfg(not(target_os = "macos"))]
    Err("Secure credential storage is unavailable".into())
}

pub(crate) fn load_provider_api_key(value: &str) -> Result<String, String> {
    let key = match provider_keychain_account(value)? {
        Some(account) if account == KEYCHAIN_ACCOUNT => load_codex_api_key()?,
        Some(account) => String::from_utf8(read_keychain_secret(&account)?
            .ok_or_else(|| "Codex credential is missing from Keychain".to_string())?)
            .map_err(|_| "Stored Codex credential is invalid".to_string())?,
        None => value.to_string(),
    };
    if key.trim().is_empty() || key.chars().any(char::is_control) {
        return Err("API key is empty or invalid".into());
    }
    Ok(key)
}

fn managed_provider(token_name: &str) -> Provider {
    let config = format!(
        "model = \"gpt-5.2-codex\"\nmodel_provider = \"bit2\"\n[model_providers.bit2]\nname = \"bit2.ai\"\nbase_url = \"{BASE_URL}\"\nwire_api = \"responses\"\n"
    );
    let mut provider = Provider::with_id(
        PROVIDER_ID.into(),
        format!("bit2.ai · {token_name}"),
        serde_json::json!({
            "auth": { "OPENAI_API_KEY": KEYCHAIN_REFERENCE },
            "config": config,
        }),
        Some(ORIGIN.into()),
    );
    provider.category = Some("partner".into());
    provider.icon = Some("openai".into());
    provider
}

#[cfg(target_os = "macos")]
fn replace_keychain_and_commit(account: &str, secret: &[u8], commit: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    let previous = read_keychain_secret(account)?;
    write_keychain_secret(account, secret)?;
    if let Err(error) = commit() {
        let rollback = match previous {
            Some(value) => write_keychain_secret(account, &value),
            None => delete_keychain_secret(account),
        };
        if rollback.is_err() {
            return Err("Authentication failed and the previous credential could not be restored".into());
        }
        return Err(error);
    }
    Ok(())
}

fn persist_authenticated_provider(state: &AppState, data: &TokenData) -> Result<(), String> {
    #[cfg(not(target_os = "macos"))]
    return Err("Secure credential storage is unavailable".into());
    #[cfg(target_os = "macos")]
    {
        let previous = state.db.get_provider_by_id(PROVIDER_ID, AppType::Codex.as_str())
            .map_err(|_| "Unable to read the existing bit2.ai provider".to_string())?;
        replace_keychain_and_commit(KEYCHAIN_ACCOUNT, data.api_key.as_bytes(), || {
            state.db.save_provider(AppType::Codex.as_str(), &managed_provider(&data.token_name))
                .map_err(|_| "Unable to save the bit2.ai provider".to_string())?;
            if crate::services::provider::select_keychain_codex_provider(state, PROVIDER_ID).is_err() {
                let rollback = match previous {
                    Some(provider) => state.db.save_provider(AppType::Codex.as_str(), &provider),
                    None => state.db.delete_provider(AppType::Codex.as_str(), PROVIDER_ID),
                };
                if rollback.is_err() {
                    return Err("Unable to select or restore the bit2.ai provider".into());
                }
                return Err("Unable to select the bit2.ai provider".into());
            }
            Ok(())
        })
    }
}

#[tauri::command]
pub fn bit2_open_login_window(app: tauri::AppHandle) -> Result<(), String> {
    let state = random_base64url(32);
    let verifier = random_base64url(32);
    let challenge = pkce_challenge(&verifier);
    let url = authorize_url(&state, &challenge)?;
    let mut auth = pending_auth()
        .lock()
        .map_err(|_| "Unable to start authentication".to_string())?;
    auth.cancel();
    auth.pending = Some(PendingAuth {
        state,
        verifier,
        created_at: Instant::now(),
    });
    if app.opener().open_url(url.as_str(), None::<String>).is_err() {
        auth.cancel();
        return Err("Unable to open the sign-in page".into());
    }
    Ok(())
}

pub async fn handle_auth_callback(app: tauri::AppHandle, raw: String) -> Result<(), String> {
    let callback = parse_auth_callback(&raw)?;
    let verifier = {
        let mut pending = pending_auth()
            .lock()
            .map_err(|_| "Authentication state is unavailable".to_string())?;
        pending.take_verifier(&callback.state)?
    };
    let data = exchange_code(&callback.code, &verifier).await?;
    // Serialize the final commit with cancellation, logout, and a new sign-in.
    let mut auth = pending_auth()
        .lock()
        .map_err(|_| "Authentication state is unavailable".to_string())?;
    auth.ensure_active(&callback.state)?;
    let state = app.state::<AppState>();
    persist_authenticated_provider(state.inner(), &data)?;
    auth.cancel();
    app.emit(
        "bit2-auth-success",
        Bit2Status {
            connected: true,
            base_url: Some(data.base_url),
        },
    )
    .map_err(|_| "Authentication completed but the UI could not be notified".to_string())?;
    Ok(())
}

#[tauri::command]
pub fn bit2_cancel_login() -> Result<(), String> {
    pending_auth()
        .lock()
        .map_err(|_| "Authentication state is unavailable".to_string())?
        .cancel();
    Ok(())
}

#[tauri::command]
pub fn bit2_status() -> Result<Bit2Status, String> {
    let connected = read_keychain_secret(KEYCHAIN_ACCOUNT)?.is_some();
    Ok(Bit2Status {
        connected,
        base_url: connected.then(|| BASE_URL.to_string()),
    })
}

#[tauri::command]
pub fn bit2_logout(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut auth = pending_auth()
        .lock()
        .map_err(|_| "Authentication state is unavailable".to_string())?;
    auth.cancel();
    delete_keychain_secret(KEYCHAIN_ACCOUNT)?;
    state
        .db
        .delete_provider(AppType::Codex.as_str(), PROVIDER_ID)
        .map_err(|_| "Unable to remove the bit2.ai provider".to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_prevents_inflight_exchange_commit_and_callback_replay() {
        let mut auth = AuthState::default();
        auth.pending = Some(PendingAuth {
            state: "first".into(),
            verifier: "secret".into(),
            created_at: Instant::now(),
        });
        assert!(auth.take_verifier("wrong").is_err());
        assert_eq!(auth.take_verifier("first").unwrap(), "secret");
        assert!(auth.take_verifier("first").is_err());
        assert!(auth.ensure_active("first").is_ok());
        auth.cancel();
        assert!(auth.ensure_active("first").is_err());
        auth.pending = Some(PendingAuth {
            state: "second".into(),
            verifier: "new-secret".into(),
            created_at: Instant::now(),
        });
        assert_eq!(auth.take_verifier("second").unwrap(), "new-secret");
        assert!(auth.ensure_active("first").is_err());
        assert!(auth.ensure_active("second").is_ok());
    }

    #[test]
    fn expired_authorization_cannot_exchange() {
        let mut auth = AuthState::default();
        auth.pending = Some(PendingAuth {
            state: "expired".into(),
            verifier: "secret".into(),
            created_at: Instant::now() - PENDING_TTL - Duration::from_secs(1),
        });
        assert!(auth.take_verifier("expired").is_err());
        assert!(auth.ensure_active("expired").is_err());
    }

    #[test]
    fn parses_exact_auth_callback() {
        assert_eq!(
            parse_auth_callback("bit2switch://auth?code=one-use-code&state=expected").unwrap(),
            AuthCallback {
                code: "one-use-code".into(),
                state: "expected".into()
            }
        );
    }

    #[test]
    fn rejects_non_exact_callback_target() {
        for raw in [
            "https://auth?code=x&state=y",
            "bit2switch://other?code=x&state=y",
            "bit2switch://auth/path?code=x&state=y",
            "bit2switch://auth/?code=x&state=y",
            "bit2switch://user@auth?code=x&state=y",
            "bit2switch://auth:42?code=x&state=y",
            "bit2switch://auth?code=x&state=y#fragment",
        ] {
            assert!(parse_auth_callback(raw).is_err(), "accepted {raw}");
        }
    }

    #[test]
    fn rejects_missing_empty_duplicate_or_extra_callback_params() {
        for raw in [
            "bit2switch://auth?state=y",
            "bit2switch://auth?code=x",
            "bit2switch://auth?code=&state=y",
            "bit2switch://auth?code=x&state=",
            "bit2switch://auth?code=x&code=z&state=y",
            "bit2switch://auth?code=x&state=y&state=z",
            "bit2switch://auth?code=x&state=y&other=z",
        ] {
            assert!(parse_auth_callback(raw).is_err(), "accepted {raw}");
        }
    }

    #[test]
    fn computes_rfc7636_s256_challenge() {
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn temporary_keychain_accounts_require_uuid_suffix() {
        assert!(
            validate_launch_account("bit2-launch-550e8400-e29b-41d4-a716-446655440000").is_ok()
        );
        assert!(validate_launch_account("codex-api-key").is_err());
        assert!(validate_launch_account("bit2-launch-not-a-uuid").is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "uses a dedicated Keychain test item to verify failed reconnect rollback"]
    fn macos_failed_reconnect_restores_previous_credential() {
        let account = format!("codex-api-key-test-{}", uuid::Uuid::new_v4());
        write_keychain_secret(&account, b"previous-test-key").unwrap();
        let result = replace_keychain_and_commit(&account, b"replacement-test-key", || Err("database unavailable".into()));
        let retained = read_keychain_secret(&account).unwrap();
        delete_keychain_secret(&account).unwrap();
        assert!(result.is_err());
        assert_eq!(retained, Some(b"previous-test-key".to_vec()));
        assert!(replace_keychain_and_commit(&account, b"first-test-key", || Err("database unavailable".into())).is_err());
        assert!(read_keychain_secret(&account).unwrap().is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "touches a dedicated macOS Keychain test item"]
    fn macos_keychain_roundtrip_uses_dedicated_test_item() {
        let account = format!("codex-api-key-test-{}", uuid::Uuid::new_v4());
        let secret = format!("sk-test-{}", uuid::Uuid::new_v4());
        write_keychain_secret(&account, secret.as_bytes()).unwrap();
        assert_eq!(
            read_keychain_secret(&account).unwrap(),
            Some(secret.into_bytes())
        );
        delete_keychain_secret(&account).unwrap();
        assert_eq!(read_keychain_secret(&account).unwrap(), None);
    }
}
