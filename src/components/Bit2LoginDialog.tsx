import { useState } from "react";
import { Loader2, LogIn, ShieldCheck } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { bit2Api } from "@/lib/api";

export function Bit2LoginDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [busy, setBusy] = useState(false);
  const [started, setStarted] = useState(false);
  const [error, setError] = useState("");

  const beginLogin = async () => {
    setBusy(true);
    setStarted(true);
    setError("");
    try {
      await bit2Api.openLoginWindow();
    } catch (cause) {
      setStarted(false);
      setError("无法打开登录窗口，请重试");
    } finally {
      setBusy(false);
    }
  };

  const changeOpen = (next: boolean) => {
    if (!next && started)
      void bit2Api.cancelLogin().catch(() => setError("无法取消登录，请重试"));
    if (!next) {
      setStarted(false);
      setError("");
    }
    onOpenChange(next);
  };

  return (
    <Dialog open={open} onOpenChange={changeOpen}>
      <DialogContent className="bit2-login-dialog max-w-md p-6 gap-5 overflow-y-auto">
        <DialogHeader className="p-0 border-0 bg-transparent">
          <div className="bit2-login-icon">
            <LogIn />
          </div>
          <DialogTitle>连接 bit2.ai</DialogTitle>
          <DialogDescription>
            在系统浏览器中完成账号、二次验证或 Passkey 登录。
          </DialogDescription>
        </DialogHeader>
        <div className="bit2-login-assurance">
          <ShieldCheck />
          <span>
            <strong>安全验证</strong>
            只有服务端验证并完成凭据交换后，桌面端才会显示已连接。
          </span>
        </div>
        {started && (
          <p role="status" className="text-sm text-cyan-300">
            等待浏览器完成验证。关闭此对话框将取消本次登录。
          </p>
        )}
        {error && (
          <p role="alert" className="bit2-login-error">
            {error}
          </p>
        )}
        <div className="flex gap-3 pt-1">
          <Button variant="outline" onClick={() => changeOpen(false)}>
            取消
          </Button>
          <Button
            className="flex-1"
            disabled={busy}
            onClick={() => void beginLogin()}
          >
            {busy ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <LogIn className="h-4 w-4" />
            )}
            {busy ? "正在打开…" : "继续登录"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
