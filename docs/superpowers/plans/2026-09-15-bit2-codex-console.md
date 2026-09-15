# bit2-switch Codex Console Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a commercial macOS Codex-first bit2-switch that logs into bit2.ai, imports the user's API credential and URL, preserves CC Switch functionality, and ships as a tested `.app` and `.dmg`.

**Architecture:** Keep CC Switch's provider and switching services as the backend source of truth. Add a Rust bit2.ai client and Keychain session store, expose narrow Tauri commands to the React UI, and make the primary shell Codex-only while leaving existing secondary capabilities intact.

**Tech Stack:** Rust, Tauri 2, React, TypeScript, Tailwind, macOS `security` CLI, Vite, Cargo, Xcode command-line build tools.

**Spec:** `docs/superpowers/specs/2026-09-15-bit2-codex-console-design.md`

## Global Constraints

- Preserve CC Switch `LICENSE`, About attribution, and `NOTICE.md`.
- Never place sessions or API keys in URLs, logs, command arguments, ordinary settings, or telemetry.
- First release exposes Codex only in primary navigation; other tools remain hidden but their backend data stays intact.
- API URL is the configured bit2.ai origin plus `/v1`.
- Every task ends with focused tests and a git commit.

### Task 1: Bit2 API client and secure session storage

**Files:**
- Create: `src-tauri/src/bit2_api.rs`
- Modify: `src-tauri/src/commands/auth.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/bit2_api.rs` unit tests

**Interfaces:**
- `Bit2ApiClient::login(base_url, username, password) -> Result<Bit2Session, Bit2ApiError>`
- `Bit2ApiClient::get_or_create_codex_token(session) -> Result<CodexCredential, Bit2ApiError>`
- Tauri commands `bit2_login`, `bit2_sync_codex`, `bit2_logout`.

- [ ] Write tests for URL normalization, login response parsing, token response parsing, and redacted error strings.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml bit2_api` and verify the new tests fail before implementation.
- [ ] Implement bounded HTTPS requests with timeouts and JSON response limits; never log response bodies.
- [ ] Implement macOS Keychain storage under service `bit2-switch`, with separate accounts for session and Codex credential; return an explicit unsupported-platform error elsewhere.
- [ ] Register the three commands in `src-tauri/src/lib.rs` and run the focused tests again.
- [ ] Commit `feat: add bit2 api client and keychain session storage`.

### Task 2: Codex synchronization workflow

**Files:**
- Modify: `src-tauri/src/commands/provider.rs`
- Modify: `src-tauri/src/commands/misc.rs`
- Modify: `src/lib/api/providers.ts`
- Modify: `src/lib/api/settings.ts`
- Test: existing Rust provider tests plus new sync tests

**Interfaces:**
- `sync_bit2_codex_provider() -> Result<Provider, String>` creates or updates one managed Codex provider whose secret is a Keychain reference.
- `providersApi.syncBit2Codex() -> Promise<Provider>` invokes the command.

- [ ] Add tests proving a sync updates the managed provider without changing unrelated providers and emits no secret in serialized JSON.
- [ ] Implement idempotent provider upsert, active-provider switch, and API URL derivation from the configured origin.
- [ ] Update terminal launch to resolve Keychain references at runtime before process creation.
- [ ] Add explicit errors for expired session, missing permission, and unavailable token endpoint.
- [ ] Run focused Rust tests and TypeScript checks.
- [ ] Commit `feat: sync bit2 ai codex provider`.

### Task 3: Embedded login and Codex-only UI

**Files:**
- Create: `src/components/Bit2LoginDialog.tsx`
- Create: `src/components/CodexConsole.tsx`
- Modify: `src/App.tsx`
- Modify: `src/lib/api/index.ts`
- Modify: `src/index.css`
- Test: `src/components/__tests__/CodexConsole.test.tsx`

**Interfaces:**
- `Bit2LoginDialog` calls `bit2_login` and reports authenticated session state.
- `CodexConsole` displays connection, CLI readiness, current endpoint/model, sync, and launch actions.

- [ ] Add component tests for logged-out, syncing, ready, expired-session, and CLI-not-installed states.
- [ ] Run the focused Vitest test and verify failure before implementation.
- [ ] Implement the dialog using the existing Tauri webview/window bridge; do not build a second password-auth protocol in React.
- [ ] Replace primary app switcher with Codex console and hide other app buttons from the main shell.
- [ ] Keep provider library, import/export, settings, About, and MIT attribution accessible as secondary views.
- [ ] Run Vitest, `tsc --noEmit`, and `vite build`.
- [ ] Commit `feat: add codex first console and bit2 login flow`.

### Task 4: macOS packaging and verification

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `README_ZH.md`
- Create: `scripts/verify-macos-package.sh`

- [ ] Add a release verification script that checks app bundle identifier, product name, icon presence, LICENSE/NOTICE inclusion, and absence of legacy primary branding.
- [ ] Run `cargo check`, all Rust tests, TypeScript checks, and Vite production build.
- [ ] Build the ARM64 `.app` and `.dmg` with Tauri using the installed Xcode command-line tools.
- [ ] Launch the generated `.app` locally, verify the Codex console is visible, and record any signing/notarization limitation.
- [ ] Run the package verification script against the generated bundle.
- [ ] Update README with installation, login, Keychain, and build instructions.
- [ ] Commit `chore: verify macos codex release package` and push `main` to `https://github.com/lhh92xcc/bit2-switch`.
