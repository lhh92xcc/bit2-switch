# bit2-switch

面向新手的 macOS Codex 配置工作台。官网：[bit2.ai](https://bit2.ai)。

当前分支是 **macOS Apple Silicon 预览版**，主界面只展示 Codex。不是已签名、公证的商业发行版。

## 使用

1. 打开 bit2-switch，选择“连接 bit2.ai”，在系统浏览器完成登录并明确选择一个 API Key；也可以使用“手动配置 API Key”。
2. 手动向导默认填入 `https://bit2.ai/v1`，模型可以按账户支持情况修改。
3. 选择配置后点击“启动 Codex”；缺少 CLI 时使用“安装并启动 Codex”。安装成功后会再次检测，再打开终端。
4. 每次启动都会使用当前所选模型、API 地址和 Key；已有终端不会被强制关闭，也不会被重新配置。

官网登录接入需要部署服务端分支 [`new-sailfish/bit2-api:codex/bit2-5177`](https://github.com/new-sailfish/bit2-api/tree/codex/bit2-5177)。代码推送不等于线上已部署。没有部署时请使用手动向导。

## 凭据与记录

- 官网导入及向导填写的 Key 通过原生 macOS Keychain API 保存。新记录只包含钥匙串引用，不把它当成真实 Key 写进全局 Codex 登录文件。
- Codex 启动时从钥匙串读取 Key，通过子进程环境传递；启动参数和配置快照不包含 Key。
- 每次启动使用独立且持久的 `~/.bit2-switch/codex-runs/<uuid>` 目录（自定义应用配置目录时随之移动）。会话记录不会在退出后删除。
- 若要从同一会话目录继续，可在终端执行 `CODEX_HOME="相应目录" codex resume`；环境凭据仍需通过 bit2-switch 的启动路径取得。
- “退出 bit2.ai”只断开桌面连接；不会撤销官网 API Key，也不会结束已有 Codex 进程。需要停用 API Key 时在官网撤销。
- 安全流程和接口字段见 [桌面授权协议](docs/bit2-desktop-auth-contract.md)。

旧有供应商库、设置、导入导出与开源说明仍可访问。此版本的隔离启动器面向第三方 API Key 配置，不支持 ChatGPT OAuth 卡片。启动快照仅带入所选模型和受支持的运行参数，不复制全局 MCP、插件或项目配置中的凭据。

## 构建与验证

依赖 Node.js/pnpm、Rust 和完整 Xcode。以下命令在项目目录执行：

```sh
pnpm install
pnpm typecheck
pnpm test:unit src/components/CodexConsole.test.tsx src/components/Bit2LoginDialog.test.tsx src/components/QuickSetupDialog.test.tsx tests/integration/App.test.tsx
cargo test --manifest-path src-tauri/Cargo.toml --lib bit2_api
pnpm tauri build --bundles app,dmg
scripts/verify-macos-package.sh src-tauri/target/release/bundle/macos/bit2-switch.app
```

产物位于 `src-tauri/target/release/bundle/macos/` 和 `src-tauri/target/release/bundle/dmg/`。本地构建采用 ad-hoc 签名；正式分发需配置 Apple Developer ID、公证及 bit2 自有更新签名。预览版关闭自动更新，手动检查打开本项目 Releases 页面。

完整测试结果与已知限制见 [验证记录](docs/bit2-macos-validation.md)。本地测试不代表已完成真实账户登录、付费模型请求或干净系统安装验证。

## 开源归属

bit2-switch 基于 [CC Switch](https://github.com/farion1231/cc-switch) 修改。MIT 允许商业使用及修改，但须保留原版权和许可证文本。本项目在 [LICENSE](LICENSE)、[NOTICE.md](NOTICE.md) 和应用 About 中保留归属；主界面使用 bit2-switch 品牌。
