import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import { Sparkles } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useSettingsQuery } from "@/lib/query";
import { settingsApi } from "@/lib/api";

/** 首次运行欢迎提示：仅当后端启动阶段保留 firstRunNoticeConfirmed 为空时弹出。 */
export function FirstRunNoticeDialog({
  onStartSetup,
}: {
  onStartSetup?: () => void;
}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { data: settings } = useSettingsQuery();

  // 后端启动时已经决定好要不要弹：条件不满足的话字段会立即被写成 true，
  // 所以前端这里只需要判空即可——与其他既有确认标记的模式一致。
  const isOpen = settings != null && settings.firstRunNoticeConfirmed !== true;

  const handleAcknowledge = async () => {
    if (!settings) return;
    try {
      const { webdavSync: _, ...rest } = settings;
      await settingsApi.save({ ...rest, firstRunNoticeConfirmed: true });
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
    } catch (error) {
      console.error("Failed to save firstRunNoticeConfirmed:", error);
    }
  };

  const handleStart = async () => {
    await handleAcknowledge();
    onStartSetup?.();
  };

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) void handleAcknowledge();
      }}
    >
      <DialogContent className="max-w-md" zIndex="top">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-blue-500" />
            {t("firstRunNotice.title")}
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-3 px-6 py-5">
          <DialogDescription className="whitespace-pre-line leading-relaxed">
            {t("firstRunNotice.bodyDefault")}
          </DialogDescription>
          <div className="rounded-lg border border-orange-200 bg-orange-50 p-3 text-sm leading-relaxed text-orange-950 dark:border-orange-900/50 dark:bg-orange-950/20 dark:text-orange-100">
            <strong>bit2-switch 快速开始</strong>
            <br />
            选择 Claude Code 或 Codex → 选择服务商 → 填写 API Key → 保存并打开终端。
            API 请求地址会自动填充。
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={handleAcknowledge}>
            {t("firstRunNotice.confirm")}
          </Button>
          <Button onClick={handleStart}>开始配置</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
