import { useEffect, useState } from "react";
import { Icon } from "@iconify/react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  ENGINE_PICKER_OPTIONS,
  resolveSelectedEngineType,
} from "@/constants/engines";
import { useI18n } from "@/i18n";
import { EngineType } from "@/types/engine";

interface ImportDialogProps {
  open: boolean;
  loading?: boolean;
  initialExecutablePath?: string;
  onOpenChange?: (open: boolean) => void;
  onSubmit?: (payload: { executablePath: string; engineType: string }) => void;
}

export default function ImportDialog({
  open,
  loading = false,
  initialExecutablePath,
  onOpenChange,
  onSubmit,
}: ImportDialogProps) {
  const { t } = useI18n();
  const [executablePath, setExecutablePath] = useState("");
  const [engineType, setEngineType] = useState<string>(EngineType.Other);

  useEffect(() => {
    if (open && initialExecutablePath) {
      setExecutablePath(initialExecutablePath);
    }
    if (!open) {
      setExecutablePath("");
      setEngineType(EngineType.Other);
    }
  }, [open, initialExecutablePath]);

  async function pickExecutable() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const res = await open({ multiple: false, title: t("import.pickExecutableTitle") });
      if (!res) return;
      const selected = Array.isArray(res) ? res[0] ?? "" : res;
      if (!selected) return;
      setExecutablePath(selected);
    } catch (e) {
      console.error("选择可执行文件失败:", e);
    }
  }

  function handleSubmit() {
    if (!executablePath) return;
    onSubmit?.({
      executablePath,
      engineType: resolveSelectedEngineType(engineType),
    });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("import.title")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm">{t("import.executable")}</label>
            <div className="flex gap-2">
              <Input
                value={executablePath}
                placeholder={t("import.executablePlaceholder")}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setExecutablePath(e.target.value)}
                className="h-8 text-sm"
              />
              <Button variant="outline" size="sm" className="h-8 px-2" onClick={pickExecutable}>
                <Icon icon="ri:folder-open-line" className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-sm">{t("import.engineType")}</label>
            <Select value={engineType} onValueChange={(v) => setEngineType(v)}>
              <SelectTrigger size="sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {ENGINE_PICKER_OPTIONS.map((engine) => (
                  <SelectItem key={engine.value} value={engine.value}>
                    {engine.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" size="sm" onClick={() => onOpenChange?.(false)}>
            {t("common.cancel")}
          </Button>
          <Button size="sm" disabled={!executablePath || loading} className="gap-2" onClick={handleSubmit}>
            {loading && <Icon icon="ri:loader-4-line" className="h-4 w-4 animate-spin" />}
            {t("common.import")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
