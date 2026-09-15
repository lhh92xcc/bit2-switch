# bit2-switch desktop credential authorization

The macOS client opens the system browser at `https://bit2.ai/desktop/authorize`.
The website handles its existing password, MFA, passkey and session lifecycle.
Users explicitly select an existing inference API token and approve its export.
No password or management PAT is requested by the desktop app.

## Request and exchange

Authorize query: `client_id=bit2-switch`, `redirect_uri=bit2switch://auth`, a random
43-character base64url `state`, `code_challenge` (SHA-256 of a random verifier,
base64url without padding), and `code_challenge_method=S256`.

The website POSTs that data plus `token_id` to `/api/desktop/authorize` with its
existing session Authorization header and Origin `https://bit2.ai`. Only live
browser sessions are accepted; PATs and foreign origins are rejected. Consent
is required before this POST. The response contains a callback with only a
short-lived code and state, never an API key.

The client validates the exact `bit2switch://auth` URI and pending state, then
POSTs `{client_id, redirect_uri, code, code_verifier}` as JSON to
`https://bit2.ai/api/desktop/token`. The response is:

```json
{"success":true,"data":{"api_key":"sk-…","base_url":"https://bit2.ai/v1","token_id":123,"token_name":"Codex"}}
```

The code expires after two minutes and can be consumed once. The server stores
only its HMAC and a token fingerprint; it rechecks the original session version,
user state, token ownership, expiry, quota, deletion and key rotation during an
atomic exchange. Key exports are audited without code, verifier or key values.
The exchange has no redirect behavior; the client uses fixed HTTPS endpoints,
timeouts and a 64 KiB response cap. Cancel, logout or a new login invalidates any
pending exchange before it can persist credentials.

## Local storage and launch

The API key is stored with the native macOS Keychain API under service
`bit2-switch`, account `codex-api-key`. The managed provider stores only
`keychain://bit2-switch/codex-api-key`. Manual wizard providers use separate
Keychain accounts. Reference-backed provider selection does not write a
reference into the user's ordinary Codex auth file.

Each launch uses a private persistent configuration/session directory under
`~/.bit2-switch/codex-runs/<uuid>` and a configuration snapshot and reads the selected key
into the child process environment. No key appears in the launcher arguments
or configuration snapshot. Existing sessions keep their original settings;
new sessions use the current selection. Desktop disconnect removes its stored
credential, but does not revoke the website API token or terminate existing
Codex sessions. Revoke a token from the website to invalidate API use.

## Validation and deployment boundary

Backend: `new-sailfish/bit2-api`, branch `codex/bit2-5177`. Tests exercise actual
SQLite, MySQL 26.7.0 and PostgreSQL 17.11, including HTTP authentication, PKCE,
concurrent replay, expiry, session revocation, token rotation/deletion and secret
exclusion from audits. No schema changes are introduced. Existing schema
creation and repeated AuthFlow migrations are covered on the three engines.

The server branch must be deployed before the website authorization page and
exchange endpoint can work in the installed desktop client. Production browser
login and a paid inference request are not validated by these local tests.
The endpoint `/v1/models` on bit2.ai was confirmed to return the expected
unauthenticated API response (HTTP 401), not a frontend page.
