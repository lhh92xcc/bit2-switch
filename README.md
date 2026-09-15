# bit2-switch

A beginner-friendly macOS Codex configuration workspace from [bit2.ai](https://bit2.ai).

This branch is an **Apple Silicon macOS preview**, with a Codex-only primary interface. It is not a notarized commercial release. See [中文说明](README_ZH.md) for the full workflow and build instructions.

Connect through the website's existing login and explicit API-token consent, or use the manual API-key wizard with `https://bit2.ai/v1` prefilled. Missing Codex CLI installations are installed and checked before launch. Each launch starts a new terminal using the selected configuration; running sessions are left intact.

Website authorization requires deploying [the server integration branch](https://github.com/new-sailfish/bit2-api/tree/codex/bit2-5177). Pushing source does not deploy it. Manual setup remains available while deployment is pending.

Credentials imported from the website or manual wizard are stored in macOS Keychain through native APIs. Provider records contain references. The launcher uses environment credentials and a private, persistent configuration/session directory under `~/.bit2-switch/codex-runs/`. Disconnecting the desktop app does not revoke the website token or stop running sessions.

The provider library, settings and import/export remain accessible. This preview launcher supports third-party API-key configurations, not ChatGPT OAuth cards. It does not copy global MCP/plugin credentials into launch snapshots.

Build with `pnpm tauri build --bundles app,dmg`; bundles are under `src-tauri/target/release/bundle/`. Local builds use ad-hoc signing. Developer ID signing, notarization and bit2-owned signed updates remain release work. Automatic updates are disabled in this preview.

See [authorization contract](docs/bit2-desktop-auth-contract.md) and [validation record](docs/bit2-macos-validation.md) for security boundaries and test limitations.

## Attribution

Based on [CC Switch](https://github.com/farion1231/cc-switch), distributed under MIT. Original copyright and license text are preserved in [LICENSE](LICENSE), [NOTICE.md](NOTICE.md), and About. The product interface uses the bit2-switch brand.
