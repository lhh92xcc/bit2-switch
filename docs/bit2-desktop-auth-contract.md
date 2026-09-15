# bit2.ai Desktop Authorization Contract

The desktop client must not read or replay the web application's HttpOnly login cookie. The server should expose a separate one-time authorization-code flow.

## Proposed flow

1. `GET /api/desktop/authorize?redirect_uri=bit2switch://auth` opens the normal bit2.ai login page.
2. After login and any 2FA/Passkey checks, the server redirects to `bit2switch://auth?code=<single-use-code>&state=<opaque-state>`.
3. The desktop client sends `POST /api/desktop/token` with `{ code, code_verifier, client_id }`.
4. The server validates the one-time code, expiry, redirect URI, PKCE verifier, and client ID, then returns `{ access_token, expires_in }`.
5. The desktop client calls `GET /api/desktop/codex-credential` with the short-lived token and receives `{ api_key, api_url }`.

The authorization code must expire quickly, be single use, be bound to the redirect URI and PKCE challenge, and never contain an API key. Responses and audit events must not log passwords, codes, sessions, or API keys.
