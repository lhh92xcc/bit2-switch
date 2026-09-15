import { useState } from "react";
import { Loader2, LogIn } from "lucide-react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { bit2Api } from "@/lib/api";

export function Bit2LoginDialog({ open, onOpenChange, onSuccess }: { open: boolean; onOpenChange: (open: boolean) => void; onSuccess?: () => void }) {
  const [username, setUsername] = useState(""); const [password, setPassword] = useState(""); const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null);
  const submit = async () => { setBusy(true); setError(null); try { await bit2Api.login("https://bit2.ai", username.trim(), password); onSuccess?.(); onOpenChange(false); } catch (e) { setError(e instanceof Error ? e.message : "登录失败"); } finally { setBusy(false); } };
  return <Dialog open={open} onOpenChange={onOpenChange}><DialogContent className="max-w-sm"><DialogHeader><DialogTitle className="flex items-center gap-2"><LogIn className="h-4 w-4" />连接 bit2.ai</DialogTitle></DialogHeader><div className="space-y-4"><div><Label>账号</Label><Input value={username} onChange={e => setUsername(e.target.value)} autoComplete="username" /></div><div><Label>密码</Label><Input type="password" value={password} onChange={e => setPassword(e.target.value)} autoComplete="current-password" /></div>{error && <p className="text-sm text-red-500">{error}</p>}<Button className="w-full" onClick={() => void submit()} disabled={busy || !username || !password}>{busy && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}登录并同步 Codex</Button></div></DialogContent></Dialog>;
}
