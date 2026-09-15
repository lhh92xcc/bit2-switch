# bit2-switch Codex Console Design

## Goal

Create a commercial macOS product based on CC Switch's MIT-licensed functionality, while presenting a Codex-first experience. Preserve the upstream LICENSE, About attribution, and NOTICE file.

## Product scope

The first product release exposes Codex only. Existing provider switching, import/export, model configuration, CLI detection, installation/update, terminal launch, and legacy migration remain available behind the Codex console. Claude, Gemini, Grok, OpenCode, OpenClaw, Hermes, and Pi entry points are hidden from the primary navigation and remain disabled in this release.

## Bit2.ai authentication and sync

The desktop app opens the bit2.ai authorization page in the system browser. The website supports the server's existing username/password, 2FA, and Passkey flows without reimplementing authentication. A one-time authorization code returns to Rust via the exact registered callback URI. Rust validates pending state and exchanges the code using S256 PKCE. The native macOS Keychain stores the exported inference API key. Desktop access never uses a management PAT or guessed password-login response headers.

The website requires explicit selection and consent for an existing API token. The desktop provider record contains only a Keychain reference and non-secret metadata. The API URL is fixed to the verified bit2.ai `/v1` gateway. See [desktop auth contract](../../bit2-desktop-auth-contract.md) for request fields and revocation semantics.

## UI architecture

Replace the multi-tool switcher as the primary shell with a Codex workspace: top brand bar, connection status, current model/API endpoint, CLI readiness, and one primary “Launch Codex” action. Secondary pages expose provider library, model settings, import/export, and About. The layout uses the existing component system and preserves accessibility and keyboard navigation.

## Data flow

1. User selects “Connect bit2.ai”.
2. System browser completes website login and explicit token consent.
3. Rust verifies callback state and PKCE exchange, then saves the key in Keychain.
4. Rust saves/selects the managed provider without replacing global Codex auth files.
5. Launch checks/installs the CLI, creates a private selected-configuration snapshot and supplies the Keychain key to the process environment.

## Error handling and security

All network calls use HTTPS, bounded response bodies, timeouts, and redacted logs. Session and API credentials are never placed in URLs, command arguments, ordinary JSON settings, or telemetry. Desktop disconnect removes the local Keychain item; website API-token revocation remains a separate explicit website action. If the API token endpoint is unavailable, the existing manually configured Codex provider remains usable.

## Validation and release

Add unit tests for endpoint derivation, token response parsing, Keychain reference resolution, and provider synchronization. Run TypeScript checks, Vite build, Rust check/tests, and a Tauri macOS ARM64 release build. Verify the generated `.app` and `.dmg` launch on macOS and document the exact artifact path and known signing/notarization status.
