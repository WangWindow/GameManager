import { useEffect, useState } from "react";
import { Icon } from "@iconify/react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { useI18n } from "@/i18n";

interface ScanDialogProps {
  open: boolean;
  loading?: boolean;
  onOpenChange?: (open: boolean) => void;
  onSubmit?: (payload: { root: string; maxDepth: number }) => void;
}

export default function ScanDialog({
  open,
  loading = false,
  onOpenChange,
  onSubmit,
}: ScanDialogProps) {
  const { t } = useI18n();
  const [root, setRoot] = useState("");
  const [maxDepth, setMaxDepth] = useState(3);

  useEffect(() => {
    if (!open) {
      setRoot("");
      setMaxDepth(3);
    }
  }, [open]);

  async function pickDirectory() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const res = await open({ directory: true, multiple: false, title: t("scan.pickRootTitle") });
      if (!res) return;
      const selected = Array.isArray(res) ? res[0] ?? "" : res;
      if (!selected) return;
      setRoot(selected);
    } catch (e) {
      console.error("选择目录失败:", e);
    }
  }

  function handleSubmit() {
    if (!root) return;
    onSubmit?.({ root, maxDepth: Math.max(1, Number(maxDepth) || 1) });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("scan.title")}</DialogTitle>
          <DialogDescription>{t("scan.description")}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">{t("scan.root")}</label>
            <div className="flex gap-2">
              <Input
                value={root}
                placeholder={t("scan.rootPlaceholder")}
                onChange={(e) => setRoot(e.target.value)}
              />
              <Button variant="secondary" className="px-3" onClick={pickDirectory}>
                <Icon icon="ri:folder-line" className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">{t("scan.maxDepth")}</label>
            <Input
              value={maxDepth}
              type="number"
              min={1}
              max={10}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setMaxDepth(Number(e.target.value))}
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange?.(false)}>
            {t("common.cancel")}
          </Button>
          <Button disabled={!root || loading} className="gap-2" onClick={handleSubmit}>
            {loading && <Icon icon="ri:loader-4-line" className="h-4 w-4 animate-spin" />}
            {t("scan.start")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
