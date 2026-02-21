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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { SUPPORTED_ENGINES, getEngineDisplayName } from "@/constants/engines";

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
  const [executablePath, setExecutablePath] = useState("");
  const [engineType, setEngineType] = useState<string>(SUPPORTED_ENGINES[0]);

  useEffect(() => {
    if (open && initialExecutablePath) {
      setExecutablePath(initialExecutablePath);
    }
    if (!open) {
      setExecutablePath("");
      setEngineType(SUPPORTED_ENGINES[0]);
    }
  }, [open, initialExecutablePath]);

  async function pickExecutable() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const res = await open({ multiple: false, title: "选择游戏可执行文件" });
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
    onSubmit?.({ executablePath, engineType });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>导入游戏</DialogTitle>
          <DialogDescription>选择游戏目录并指定引擎类型</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">可执行文件</label>
            <div className="flex gap-2">
              <Input
                value={executablePath}
                placeholder="选择游戏可执行文件"
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setExecutablePath(e.target.value)}
              />
              <Button variant="secondary" className="px-3" onClick={pickExecutable}>
                <Icon icon="ri:file-3-line" className="h-4 w-4" />
              </Button>
            </div>
            <div className="text-xs text-muted-foreground">
              游戏目录将自动识别为可执行文件所在目录
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">引擎类型</label>
            <Select value={engineType} onValueChange={(v) => setEngineType(v)}>
              <SelectTrigger>
                <SelectValue placeholder="选择引擎类型" />
              </SelectTrigger>
              <SelectContent>
                {SUPPORTED_ENGINES.map((engine) => (
                  <SelectItem key={engine} value={engine}>
                    {getEngineDisplayName(engine as any)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange?.(false)}>
            取消
          </Button>
          <Button disabled={!executablePath || loading} className="gap-2" onClick={handleSubmit}>
            {loading && <Icon icon="ri:loader-4-line" className="h-4 w-4 animate-spin" />}
            导入
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
