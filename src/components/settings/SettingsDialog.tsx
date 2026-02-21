import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";

interface SettingsDialogProps {
  open: boolean;
  themeMode: "system" | "light" | "dark";
  onOpenChange?: (open: boolean) => void;
  onThemeModeChange?: (mode: "system" | "light" | "dark") => void;
}

export default function SettingsDialog({
  open,
  themeMode,
  onOpenChange,
  onThemeModeChange,
}: SettingsDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>设置</DialogTitle>
          <DialogDescription>基础偏好设置</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="flex items-center justify-between rounded-md border px-3 py-2">
            <div>
              <div className="text-sm font-medium">主题</div>
              <div className="text-xs text-muted-foreground">跟随系统或手动选择</div>
            </div>
            <Select
              value={themeMode}
              onValueChange={(v) => onThemeModeChange?.(v as "system" | "light" | "dark")}
            >
              <SelectTrigger>
                <SelectValue placeholder="主题" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">系统</SelectItem>
                <SelectItem value="light">浅色</SelectItem>
                <SelectItem value="dark">深色</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Separator />
          <div className="text-xs text-muted-foreground">更多设置将逐步补充</div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange?.(false)}>
            关闭
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
