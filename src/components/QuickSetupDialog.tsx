import { useMemo, useState } from "react";
import {
  Check,
  ChevronLeft,
  ChevronRight,
  Loader2,
  ShieldCheck,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import { settingsApi } from "@/lib/api";
import { generateThirdPartyConfig } from "@/config/codexProviderPresets";

type QuickApp = Extract<AppId, "claude" | "codex">;

const DEFAULT_BASE_URL = "https://bit2.ai/v1";

interface QuickSetupDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: (app: QuickApp, provider: Provider) => Promise<void>;
}

export function QuickSetupDialog({
  open,
  onOpenChange,
  onComplete,
}: QuickSetupDialogProps) {
  const app: QuickApp = "codex";
  const [progress, setProgress] = useState("");
  const [baseUrl, setBaseUrl] = useState(DEFAULT_BASE_URL);
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [step, setStep] = useState(0);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const appLabel = "Codex";
  const canContinue = useMemo(() => {
    if (step === 0) return true;
    try {
      return (
        ["https:", "http:"].includes(new URL(baseUrl.trim()).protocol) &&
        apiKey.trim().length > 0
      );
    } catch {
      return false;
    }
  }, [apiKey, baseUrl, step]);

  const reset = () => {
    setStep(0);
    setApiKey("");
    setModel("");
    setBaseUrl(DEFAULT_BASE_URL);
    setError(null);
    setBusy(false);
    setProgress("");
  };

  const finish = async () => {
    setBusy(true);
    setError(null);
    const trimmedUrl = baseUrl.trim().replace(/\/+$/, "");
    const trimmedKey = apiKey.trim();
    const providerId = crypto.randomUUID();
    const provider: Provider = {
      id: providerId,
      name:
        new URL(trimmedUrl).hostname === "bit2.ai"
          ? "bit2.ai"
          : new URL(trimmedUrl).hostname,
      category: "custom",
      icon: "bit2",
      iconColor: "#22d3ee",
      settingsConfig: {
        auth: { OPENAI_API_KEY: `bit2-keychain://codex-${providerId}` },
        config: generateThirdPartyConfig(
          "custom",
          trimmedUrl,
          model.trim() || "gpt-5.6-sol",
        ),
      },
    };

    try {
      setProgress("正在检测 Codex CLI…");
      let [status] = await settingsApi.getToolVersions([app]);
      if (!status?.version || status.installed_but_broken || status.error) {
        await settingsApi.runToolLifecycleAction([app], "install");
        setProgress("已运行安装程序，正在等待检测结果…");
        const deadline = Date.now() + 90_000;
        while (Date.now() < deadline) {
          await new Promise((resolve) => setTimeout(resolve, 3000));
          [status] = await settingsApi.getToolVersions([app]);
          if (status?.version && !status.installed_but_broken && !status.error)
            break;
        }
        if (!status?.version || status.installed_but_broken || status.error) {
          throw new Error(
            "尚未检测到可用的 Codex CLI。请检查安装终端，完成后点击保存并打开以重试。",
          );
        }
      }
      setProgress("CLI 已就绪，正在保存配置并打开新终端…");
      await settingsApi.storeBit2Secret(
        "bit2-switch",
        `${app}-${provider.id}`,
        trimmedKey,
      );
      await onComplete(app, provider);
      onOpenChange(false);
      reset();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "配置失败，请重试");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (busy) return;
        if (!next) reset();
        onOpenChange(next);
      }}
    >
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-cyan-400" />
            Codex 快速配置
          </DialogTitle>
          <DialogDescription>
            保存 API 凭据，并在新的终端中启动 Codex。
          </DialogDescription>
        </DialogHeader>

        <div className="flex items-center gap-2 px-6 pt-2 text-xs text-muted-foreground">
          {["开始", "填写 API", "确认"].map((label, index) => (
            <div key={label} className="flex flex-1 items-center gap-2">
              <span
                className={`flex h-6 w-6 items-center justify-center rounded-full ${index <= step ? "bg-cyan-400 text-slate-950" : "bg-muted"}`}
              >
                {index < step ? <Check className="h-3.5 w-3.5" /> : index + 1}
              </span>
              <span>{label}</span>
              {index < 2 && <span className="h-px flex-1 bg-border" />}
            </div>
          ))}
        </div>

        <div className="space-y-5 px-6 py-5">
          {step === 0 && (
            <div className="rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5">
              <div className="font-medium text-cyan-300">
                Codex · OpenAI Responses
              </div>
              <p className="mt-2 text-sm text-muted-foreground">
                填写 API Key 即可开始。默认使用
                bit2.ai，也可以填写其他兼容服务的 API 地址。
              </p>
            </div>
          )}
          {step === 1 && (
            <div className="space-y-4">
              <div className="rounded-lg bg-muted/50 p-3 text-sm">
                正在配置：<strong>{appLabel}</strong>
              </div>
              <div className="space-y-2">
                <Label htmlFor="quick-base-url">API 请求地址</Label>
                <Input
                  id="quick-base-url"
                  value={baseUrl}
                  onChange={(event) => setBaseUrl(event.target.value)}
                  placeholder={DEFAULT_BASE_URL}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="quick-api-key">API Key</Label>
                <Input
                  id="quick-api-key"
                  value={apiKey}
                  onChange={(event) => setApiKey(event.target.value)}
                  type="password"
                  placeholder="粘贴你的 API Key"
                  autoComplete="off"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="quick-model">模型（可选）</Label>
                <Input
                  id="quick-model"
                  value={model}
                  onChange={(event) => setModel(event.target.value)}
                  placeholder="gpt-5.6-sol"
                />
              </div>
              <p className="text-xs text-muted-foreground">
                API Key 保存在系统钥匙串，启动时读取。
              </p>
            </div>
          )}
          {step === 2 && (
            <div className="space-y-3 rounded-lg border border-green-200 bg-green-50 p-4 text-sm text-green-900 dark:border-green-900/50 dark:bg-green-950/20 dark:text-green-100">
              <div className="font-medium">准备完成</div>
              <div>保存 {appLabel} 配置，并打开新终端。</div>
              <div className="text-xs opacity-80">
                如果工具尚未安装，bit2-switch
                会使用现有的一键安装入口提示你安装。
              </div>
            </div>
          )}
          {progress && (
            <p role="status" className="text-sm text-cyan-400">
              {progress}
            </p>
          )}
          {error && (
            <p role="alert" className="text-sm text-red-400">
              {error}
            </p>
          )}
        </div>

        <DialogFooter className="flex-row justify-between">
          <Button
            variant="ghost"
            onClick={() => {
              if (step === 0) onOpenChange(false);
              else setStep((value) => value - 1);
            }}
            disabled={busy}
          >
            <ChevronLeft className="mr-1 h-4 w-4" />
            返回
          </Button>
          {step < 2 ? (
            <Button
              onClick={() => setStep((value) => value + 1)}
              disabled={!canContinue}
            >
              下一步
              <ChevronRight className="ml-1 h-4 w-4" />
            </Button>
          ) : (
            <Button onClick={() => void finish()} disabled={busy}>
              {busy && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              保存并打开
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
