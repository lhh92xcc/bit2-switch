# macOS preview validation — 2026-09-15

## Server

`new-sailfish/bit2-api`, `codex/bit2-5177`, commit
`c06e6ca0b2d8d2d125ef08ee1aa2e3963a1561e4`. Remote tree and commit were verified
through the GitHub API. No production deployment was performed.

- `go build ./...` passed with Go 1.27.1.
- Focused controller/middleware/service authentication regression selection passed.
- `go test ./service -run '^TestDesktopAuthorization$' -count=2 -race -v` passed
  with real SQLite 3.50.4 (glebarez driver), MySQL 26.7.0 and PostgreSQL 17.11.
- Scratch MySQL ran on 127.0.0.1:55339 and PostgreSQL on 127.0.0.1:55439; both
  used a dedicated `bit2_desktop_test` database. No schema changes were made.
  Tests initialize existing schema and repeat AuthFlow migration over populated
  data. The audit table is also exercised on all three engines.
- Website: 13 focused tests, TypeScript, targeted oxlint/format checks and Rsbuild
  production build passed.
- Security review covered PKCE, exact redirect/client/origin binding, one-time
  concurrent exchange, session/version and token ownership/rotation/deletion,
  body limits, no-store and safe audit fields. The audit gap found in review was
  fixed and covered by a failing-then-passing regression test.

OWASP references: OAuth2, Session Management and CSRF Cheat Sheets. These checks
are not a claim of full ASVS certification. Production TLS/proxy/CORS behavior
and real-account login/export/inference require deployment and separate testing.

## Client

- Focused native auth/launcher/provider tests exercise real Keychain write/read/
  deletion and failure rollback, callback cancellation, config pinning, private
  file permissions, history retention, and selected-provider live-file isolation.
- All six native installer tests passed, including the full official GitHub asset
  download, SHA256 verification, extraction, atomic install into a temporary
  directory and real `codex-cli 0.154.0` probe on Apple silicon.
- Installed Codex accepted generated configuration without an API request.
- A controlled CLI received its test key via environment after actual Keychain
  retrieval. No real inference credential was used by those tests.
- TypeScript and focused Codex console, login, wizard and App integration tests
  passed. Installation tests require a usable version probe before saving or
  launching, rather than treating installer invocation as completion.
- The broad frontend run had 1097 passes and 22 failures before updating obsolete
  App navigation tests. App integration now passes. A clean checkout of baseline
  `c486ea7f` reproduced 10 failures in Pi presets/form, routing brand, sessions,
  WebDAV and directory/settings tests. Five additional load-related failures in
  unchanged Claude/Pi form tests disappeared on focused rerun, leaving the same
  baseline Pi failure. The full inherited suite is therefore not reported green.

## Package

Build machine: Apple Silicon macOS, Xcode 27.0 (27A266a), Apple clang toolchain.
Packaging uses Tauri's real native `.app`/`.dmg` pipeline. Local ad-hoc signing
seals the bundle; it is not Developer ID signing or Apple notarization.

Final `pnpm tauri build --bundles app,dmg` completed successfully. The package
verification script passed for both the generated `.app` and the copy mounted
read-only from its DMG: ARM64 executable, bundle identity, icon, MIT/NOTICE
resources and deep/strict signature integrity. The two executables matched.
The final application was quit/reopened and visually inspected: Codex console,
fixed toolbar, scrollable content and padded login dialog with working Cancel.
No real website login or paid inference was attempted.

Artifacts (relative to the repository):

- `src-tauri/target/release/bundle/macos/bit2-switch.app`
- `src-tauri/target/release/bundle/dmg/bit2-switch_3.20.3_aarch64.dmg`

SHA256:

- DMG: `cce6087fdf6a680d1078d58cbb0c0bc4aa7e0cbeaeb82d4969ab93b2dc185399`
- App executable: `819a019464dfb8c620bcfa7e1ea52de29834abcf8ca88563df7b04e2875470c4`

## Preview scope

The primary flow supports macOS Codex API-key providers. ChatGPT OAuth launch,
Windows packaging, global MCP/plugin credential import into snapshots, server
production deployment, developer signing/notarization and a real paid inference
request have not been delivered or validated in this preview. API-key status is
local credential availability, not proof the website has not since revoked it.
