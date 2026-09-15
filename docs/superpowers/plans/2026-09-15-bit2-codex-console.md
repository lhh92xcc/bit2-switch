# bit2-switch Codex Console Implementation Plan

Revised after examining the real bit2-api session and token system. The
[desktop auth contract](../../bit2-desktop-auth-contract.md) replaces the earlier
password-login and management-token assumptions.

- [x] Add explicit website consent, S256 PKCE, single-use code exchange, session/token checks and audit logging on the authorized server branch.
- [x] Test backend with real SQLite, MySQL and PostgreSQL; test website consent and compile server/web.
- [x] Replace fake desktop login callbacks with verified Rust exchange and native macOS Keychain storage, including cancellation and logout races.
- [x] Redesign the main window as a Codex console; retain secondary provider and settings pages, add manual API wizard and install/recheck actions.
- [x] Select reference-backed providers without writing references to ordinary Codex auth files; launch the chosen API configuration in a new terminal.
- [x] Finish launcher review and verify current macOS app and DMG with Xcode tools.
- [x] Launch the packaged application, inspect its UI and record actual artifact paths.
- [x] Commit/push the client and update validation record.

Production deployment of the server branch, signed/notarized distribution and
real-account browser-to-desktop/inference testing remain separate release work.
Local builds and unit tests are not evidence those deployment steps happened.
