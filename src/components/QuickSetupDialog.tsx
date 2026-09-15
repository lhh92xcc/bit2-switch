import { useMemo, useState } from "react";
import { Check, ChevronLeft, ChevronRight, Loader2, ShieldCheck } from "lucide-react";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import { settingsApi } from "@/lib/api";
import { generateThirdPartyConfig } from "@/config/codexProviderPresets";

type QuickApp = Extract<AppId, "claude" | "codex">;

const DEFAULT_BASE_URL = "https://api.bit2.ai/v1";

interface QuickSetupDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: (app: QuickApp, provider: Provider) => Promise<void>;
}

export function QuickSetupDialog({ open, onOpenChange, onComplete }: QuickSetupDialogProps) {
  const [app, setApp] = useState<QuickApp>("claude");
  const [baseUrl, setBaseUrl] = useState(DEFAULT_BASE_URL);
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [step, setStep] = useState(0);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const appLabel = app === "claude" ? "Claude Code" : "Codex";
  const canContinue = useMemo(() => {
    if (step === 0) return true;
    return baseUrl.trim().startsWith("http") && apiKey.trim().length > 0;
  }, [apiKey, baseUrl, step]);

  const reset = () => {
    setStep(0);
    setApiKey("");
    setModel("");
    setBaseUrl(DEFAULT_BASE_URL);
    setError(null);
    setBusy(false);
  };

  const finish = async () => {
    setBusy(true);
    setError(null);
    const trimmedUrl = baseUrl.trim().replace(/\/+$/, "");
    const trimmedKey = apiKey.trim();
    const provider: Provider = app === "claude"
      ? {
          id: crypto.randomUUID(),
          name: "bit2.ai",
          websiteUrl: "https://bit2.ai",
          category: "custom",
          icon: "bit2",
          iconColor: "#E78242",
          settingsConfig: {
            env: {
              ANTHROPIC_BASE_URL: trimmedUrl,
              ANTHROPIC_AUTH_TOKEN: trimmedKey,
              ...(model.trim() ? { ANTHROPIC_MODEL: model.trim() } : {}),
            },
          },
        }
      : {
          id: crypto.randomUUID(),
          name: "bit2.ai",
          websiteUrl: "https://bit2.ai",
          category: "custom",
          icon: "bit2",
          iconColor: "#E78242",
          settingsConfig: {
            auth: { OPENAI_API_KEY: trimmedKey },
            config: generateThirdPartyConfig("bit2.ai", trimmedUrl, model.trim() || "gpt-5.6-sol"),
          },
        };

    try {
      // Make the selected CLI ready before applying its configuration.
      const [status] = await settingsApi.getToolVersions([app]);
      if (!status?.version) {
        await settingsApi.runToolLifecycleAction([app], "install");
      } else if (status.latest_version && status.latest_version !== status.version) {
        await settingsApi.runToolLifecycleAction([app], "update");
      }
      await settingsApi.storeBit2Secret("bit2-switch", `${app}-${provider.id}`, trimmedKey);
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
    <Dialog open={open} onOpenChange={(next) => { if (!next) reset(); onOpenChange(next); }}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2"><ShieldCheck className="h-5 w-5 text-orange-600" />bit2-switch 快速配置</DialogTitle>
          <DialogDescription>按步骤填写一次，bit2-switch 会保存配置并打开对应工具。</DialogDescription>
        </DialogHeader>

        <div className="flex items-center gap-2 px-6 pt-2 text-xs text-muted-foreground">
          {["选择工具", "填写 API", "完成"].map((label, index) => (
            <div key={label} className="flex flex-1 items-center gap-2">
              <span className={`flex h-6 w-6 items-center justify-center rounded-full ${index <= step ? "bg-orange-600 text-white" : "bg-muted"}`}>{index < step ? <Check className="h-3.5 w-3.5" /> : index + 1}</span>
              <span>{label}</span>
              {index < 2 && <span className="h-px flex-1 bg-border" />}
            </div>
          ))}
        </div>

        <div className="space-y-5 px-6 py-5">
          {step === 0 && (
            <div className="grid grid-cols-2 gap-3">
              {(["claude", "codex"] as QuickApp[]).map((value) => (
                <button key={value} type="button" onClick={() => setApp(value)} className={`rounded-lg border p-4 text-left transition ${app === value ? "border-orange-500 bg-orange-50 dark:bg-orange-950/30" : "border-border hover:border-orange-300"}`}>
                  <div className="font-medium">{value === "claude" ? "Claude Code" : "Codex"}</div>
                  <div className="mt-1 text-xs text-muted-foreground">{value === "claude" ? "Anthropic 兼容接口" : "OpenAI Responses 兼容接口"}</div>
                </button>
              ))}
            </div>
          )}
          {step === 1 && (
            <div className="space-y-4">
              <div className="rounded-lg bg-muted/50 p-3 text-sm">正在配置：<strong>{appLabel}</strong></div>
              <div className="space-y-2"><Label htmlFor="quick-base-url">API 请求地址</Label><Input id="quick-base-url" value={baseUrl} onChange={(event) => setBaseUrl(event.target.value)} placeholder={DEFAULT_BASE_URL} /></div>
              <div className="space-y-2"><Label htmlFor="quick-api-key">API Key</Label><Input id="quick-api-key" value={apiKey} onChange={(event) => setApiKey(event.target.value)} type="password" placeholder="粘贴你的 API Key" autoComplete="off" /></div>
              <div className="space-y-2"><Label htmlFor="quick-model">模型（可选）</Label><Input id="quick-model" value={model} onChange={(event) => setModel(event.target.value)} placeholder={app === "claude" ? "留空使用服务商默认模型" : "gpt-5.6-sol"} /></div>
              <p className="text-xs text-muted-foreground">API Key 仅用于生成本机工具配置，不会显示在界面日志中。</p>
            </div>
          )}
          {step === 2 && (
            <div className="space-y-3 rounded-lg border border-green-200 bg-green-50 p-4 text-sm text-green-900 dark:border-green-900/50 dark:bg-green-950/20 dark:text-green-100">
              <div className="font-medium">准备完成</div>
              <div>保存 {appLabel} 的 bit2.ai 配置，并打开终端。</div>
              <div className="text-xs opacity-80">如果工具尚未安装，bit2-switch 会使用现有的一键安装入口提示你安装。</div>
            </div>
          )}
          {error && <p className="text-sm text-red-600">{error}</p>}
        </div>

        <DialogFooter className="flex-row justify-between">
          <Button variant="ghost" onClick={() => { if (step === 0) onOpenChange(false); else setStep((value) => value - 1); }} disabled={busy}><ChevronLeft className="mr-1 h-4 w-4" />返回</Button>
          {step < 2 ? <Button onClick={() => setStep((value) => value + 1)} disabled={!canContinue}>下一步<ChevronRight className="ml-1 h-4 w-4" /></Button> : <Button onClick={() => void finish()} disabled={busy}>{busy && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}保存并打开</Button>}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
