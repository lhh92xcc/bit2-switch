import {
  ArrowUpRight,
  Check,
  ChevronRight,
  CircleAlert,
  Cloud,
  Code2,
  KeyRound,
  LogOut,
  Play,
  Settings2,
  SlidersHorizontal,
} from "lucide-react";
import type { Provider } from "@/types";
import {
  getCodexBaseUrl,
  extractCodexModelName,
} from "@/utils/providerConfigUtils";
import { Button } from "@/components/ui/button";

export type CodexCliStatus =
  | { state: "checking" }
  | { state: "ready"; version: string }
  | { state: "missing" }
  | { state: "error"; message?: string };

interface CodexConsoleProps {
  authStatus: { connected: boolean; baseUrl?: string | null } | null;
  launching?: boolean;
  onRefreshCli?: () => void;
  provider?: Provider;
  cliStatus: CodexCliStatus;
  onConnect: () => void;
  onLogout: () => void;
  onLaunch: () => void;
  onAddProvider: () => void;
  onOpenProviders: () => void;
  onOpenSettings: () => void;
  onOpenAbout: () => void;
}

function StatusDot({ good }: { good: boolean }) {
  return (
    <span className={good ? "codex-status-dot is-good" : "codex-status-dot"} />
  );
}

export function CodexConsole({
  authStatus,
  launching = false,
  onRefreshCli,
  provider,
  cliStatus,
  onConnect,
  onLogout,
  onLaunch,
  onAddProvider,
  onOpenProviders,
  onOpenSettings,
  onOpenAbout,
}: CodexConsoleProps) {
  const config =
    typeof provider?.settingsConfig?.config === "string"
      ? provider.settingsConfig.config
      : "";
  const endpoint = getCodexBaseUrl(provider) ?? "尚未配置";
  const model = extractCodexModelName(config) || "尚未配置";
  const cliReady = cliStatus.state === "ready";
  const canLaunch = Boolean(
    provider && cliStatus.state !== "checking" && !launching,
  );

  return (
    <div className="codex-console">
      <div className="codex-console-orb codex-console-orb-one" />
      <div className="codex-console-orb codex-console-orb-two" />
      <section className="codex-hero">
        <div className="codex-eyebrow">
          <Code2 /> CODEX WORKSPACE
        </div>
        <h1>
          从这里启动你的
          <br />
          <span>Codex 工作流。</span>
        </h1>
        <p>
          连接 bit2.ai，或继续使用你已有的 API
          配置。每次启动都会在新终端中使用当前配置。
        </p>
        <div className="codex-actions">
          <Button
            onClick={onLaunch}
            disabled={!canLaunch}
            className="codex-launch"
          >
            <Play className="h-4 w-4 fill-current" />
            {launching
              ? "正在准备 Codex…"
              : cliReady
                ? "启动 Codex"
                : "安装并启动 Codex"}
          </Button>
          {!provider && (
            <Button
              variant="outline"
              onClick={onAddProvider}
              className="codex-secondary"
            >
              <KeyRound className="h-4 w-4" />
              手动配置 API Key
            </Button>
          )}
        </div>
        {!canLaunch && (
          <p className="codex-launch-hint">
            <CircleAlert className="h-4 w-4" />
            {!provider
              ? "先连接账号或配置一个 Codex 供应商。"
              : "检测到 Codex CLI 后即可启动。"}
          </p>
        )}
      </section>

      <section className="codex-grid" aria-label="Codex 状态">
        <article className="codex-panel codex-account-panel">
          <div className="codex-panel-heading">
            <Cloud />
            <span>BIT2.AI CONNECTION</span>
          </div>
          <div className="codex-connection-row">
            <div>
              <div className="codex-status-label">
                <StatusDot good={Boolean(authStatus?.connected)} />
                {authStatus?.connected ? "已连接 bit2.ai" : "尚未连接"}
              </div>
              <p>
                {authStatus?.connected
                  ? authStatus.baseUrl
                  : "登录后自动同步你的 Codex 凭据"}
              </p>
            </div>
            {authStatus?.connected ? (
              <Button variant="ghost" size="sm" onClick={onLogout}>
                <LogOut className="h-4 w-4" />
                退出
              </Button>
            ) : (
              <Button onClick={onConnect} size="sm">
                连接 bit2.ai
                <ArrowUpRight className="h-4 w-4" />
              </Button>
            )}
          </div>
          <div className="codex-security-note">
            <Check className="h-4 w-4" />
            账号凭据通过验证后保存在系统钥匙串
          </div>
        </article>

        <article className="codex-panel">
          <div className="codex-panel-heading">
            <SlidersHorizontal />
            <span>ACTIVE CONFIGURATION</span>
          </div>
          <dl className="codex-config-list">
            <div>
              <dt>供应商</dt>
              <dd>{provider?.name ?? "尚未选择"}</dd>
            </div>
            <div>
              <dt>模型</dt>
              <dd>{model}</dd>
            </div>
            <div>
              <dt>API 端点</dt>
              <dd title={endpoint}>{endpoint}</dd>
            </div>
          </dl>
          <button className="codex-text-link" onClick={onOpenProviders}>
            管理供应商 <ChevronRight />
          </button>
        </article>

        <article className="codex-panel codex-cli-panel">
          <div className="codex-panel-heading">
            <Code2 />
            <span>LOCAL RUNTIME</span>
          </div>
          <div className="codex-runtime-state">
            <StatusDot good={cliReady} />
            <div>
              <strong>
                {cliStatus.state === "checking" && "正在检测 Codex CLI"}
                {cliStatus.state === "ready" &&
                  `Codex CLI ${cliStatus.version}`}
                {cliStatus.state === "missing" && "未检测到 Codex CLI"}
                {cliStatus.state === "error" && "无法检测 Codex CLI"}
              </strong>
              <p>
                {cliStatus.state === "error"
                  ? cliStatus.message
                  : cliReady
                    ? "本机运行环境已就绪"
                    : "可在设置中安装或修复"}
              </p>
            </div>
          </div>
          <button className="codex-text-link" onClick={onOpenSettings}>
            打开 CLI 设置 <ChevronRight />
          </button>
          {onRefreshCli && (
            <button
              className="codex-text-link"
              disabled={cliStatus.state === "checking"}
              onClick={onRefreshCli}
            >
              重新检测 CLI
            </button>
          )}
        </article>
      </section>

      <nav className="codex-footer-nav" aria-label="次要功能">
        <button onClick={onOpenProviders}>
          <KeyRound />
          供应商与 API
        </button>
        <button onClick={onOpenSettings}>
          <Settings2 />
          设置与导入导出
        </button>
        <button onClick={onOpenAbout}>
          <CircleAlert />
          关于与开源许可
        </button>
      </nav>
    </div>
  );
}
