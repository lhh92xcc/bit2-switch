# bit2-switch Codex Console Design

## Goal

Create a commercial macOS product based on CC Switch's MIT-licensed functionality, while presenting a Codex-first experience. Preserve the upstream LICENSE, About attribution, and NOTICE file.

## Product scope

The first product release exposes Codex only. Existing provider switching, import/export, model configuration, CLI detection, installation/update, terminal launch, and legacy migration remain available behind the Codex console. Claude, Gemini, Grok, OpenCode, OpenClaw, Hermes, and Pi entry points are hidden from the primary navigation and remain disabled in this release.

## Bit2.ai authentication and sync

The desktop app opens an embedded bit2.ai login webview. The webview supports the server's existing username/password, 2FA, and Passkey flows without reimplementing authentication. A callback bridge returns only a short-lived session result to Rust. Rust stores the session securely in macOS Keychain.

After authentication, Rust calls the bit2-api user token and token-management endpoints over HTTPS. It obtains the user's Codex API credential and derives the API URL as the configured bit2.ai origin plus `/v1`. The credential is stored only in Keychain. The local provider record contains a Keychain reference and non-secret metadata. Sync refreshes the provider, switches it active, and reports token expiry or permission errors with a re-login action.

## UI architecture

Replace the multi-tool switcher as the primary shell with a Codex workspace: top brand bar, connection status, current model/API endpoint, CLI readiness, and one primary “Launch Codex” action. Secondary pages expose provider library, model settings, import/export, and About. The layout uses the existing component system and preserves accessibility and keyboard navigation.

## Data flow

1. User selects “Connect bit2.ai”.
2. Embedded webview completes login and returns a session.
3. Rust validates the session, stores it in Keychain, and requests/refreshes the Codex token.
4. Rust writes a non-secret Codex provider reference, switches it active, and resolves the CLI installation.
5. Launch reads the Keychain secret at runtime, injects it into the Codex process, and never logs it.

## Error handling and security

All network calls use HTTPS, bounded response bodies, timeouts, and redacted logs. Session and API credentials are never placed in URLs, command arguments, ordinary JSON settings, or telemetry. Logout revokes/forgets the local session and removes the Keychain item. If the API token endpoint is unavailable, the existing manually configured Codex provider remains usable.

## Validation and release

Add unit tests for endpoint derivation, token response parsing, Keychain reference resolution, and provider synchronization. Run TypeScript checks, Vite build, Rust check/tests, and a Tauri macOS ARM64 release build. Verify the generated `.app` and `.dmg` launch on macOS and document the exact artifact path and known signing/notarization status.
