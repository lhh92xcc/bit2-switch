import { LogIn, ExternalLink } from "lucide-react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { bit2Api } from "@/lib/api";

export function Bit2LoginDialog({ open, onOpenChange, onSuccess }: { open: boolean; onOpenChange: (open: boolean) => void; onSuccess?: () => void }) {
  return <Dialog open={open} onOpenChange={onOpenChange}><DialogContent className="max-w-sm"><DialogHeader><DialogTitle className="flex items-center gap-2"><LogIn className="h-4 w-4" />连接 bit2.ai</DialogTitle></DialogHeader><div className="space-y-4"><p className="text-sm text-muted-foreground">将在安全登录窗口中完成账号、二次验证或 Passkey 验证。登录信息不会经过 bit2-switch。</p><Button className="w-full" onClick={() => void bit2Api.openLogin()}><ExternalLink className="mr-2 h-4 w-4" />打开 bit2.ai 登录</Button><Button variant="outline" className="w-full" onClick={() => { onSuccess?.(); onOpenChange(false); }}>我已完成登录</Button></div></DialogContent></Dialog>;
}
