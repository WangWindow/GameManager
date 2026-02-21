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
      const res = await open({ directory: true, multiple: false, title: "选择扫描根目录" });
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
          <DialogTitle>扫描游戏</DialogTitle>
          <DialogDescription>选择根目录并设置最大扫描深度</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">扫描根目录</label>
            <div className="flex gap-2">
              <Input
                value={root}
                placeholder="选择扫描根目录"
                onChange={(e) => setRoot(e.target.value)}
              />
              <Button variant="secondary" className="px-3" onClick={pickDirectory}>
                <Icon icon="ri:folder-line" className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">最大扫描深度</label>
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
            取消
          </Button>
          <Button disabled={!root || loading} className="gap-2" onClick={handleSubmit}>
            {loading && <Icon icon="ri:loader-4-line" className="h-4 w-4 animate-spin" />}
            开始扫描
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
