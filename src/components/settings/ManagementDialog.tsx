import { useEffect, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Switch } from "@/components/ui/switch";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { EngineDto } from "@/types";

interface ManagementDialogProps {
  open: boolean;
  showStatusBar: boolean;
  onOpenChange?: (open: boolean) => void;
  onShowStatusBarChange?: (v: boolean) => void;
  onDownloadNwjs?: () => void;
  onCleanupContainers?: () => void;
  onUpdateEngine?: (engine: EngineDto) => void;
  onRemoveEngine?: (engine: EngineDto) => void;
}

export default function ManagementDialog({
  open,
  showStatusBar,
  onOpenChange,
  onShowStatusBarChange,
  onDownloadNwjs,
  onCleanupContainers,
  onUpdateEngine,
  onRemoveEngine,
}: ManagementDialogProps) {
  const [engines, setEngines] = useState<EngineDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [bottlesLoading, setBottlesLoading] = useState(false);
  const [bottlesAvailable, setBottlesAvailable] = useState(false);
  const [bottlesInstalled, setBottlesInstalled] = useState(false);
  const [bottlesEnabled, setBottlesEnabled] = useState(false);
  const [bottlesList, setBottlesList] = useState<string[]>([]);
  const [defaultBottle, setDefaultBottle] = useState("");
  const bottlesInstallCommand = 'flatpak install flathub com.usebottles.bottles';

  async function fetchEngines() {
    setLoading(true);
    try {
      const list = await import("@/lib/api").then((m) => m.getEngines());
      setEngines(list);
    } catch (e) {
      console.error("获取运行器失败:", e);
    } finally {
      setLoading(false);
    }
  }

  async function fetchBottlesStatus() {
    setBottlesLoading(true);
    try {
      const status = await import("@/lib/api").then((m) => m.getIntegrationStatus("bottles"));
      const options = status.options ?? {};
      setBottlesAvailable(status.available);
      setBottlesInstalled(options.installed ?? status.available);
      setBottlesEnabled(status.enabled);
      setBottlesList(options.bottles ?? []);
      setDefaultBottle(options.defaultBottle ?? "");
    } catch (e) {
      setBottlesAvailable(false);
      setBottlesInstalled(false);
      setBottlesEnabled(false);
      setBottlesList([]);
      setDefaultBottle("");
      console.error("获取 Bottles 状态失败:", e);
    } finally {
      setBottlesLoading(false);
    }
  }

  async function updateBottlesEnabled(value: boolean) {
    setBottlesEnabled(value);
    try {
      await import("@/lib/api").then((m) =>
        m.setIntegrationSettings({ key: "bottles", enabled: value }),
      );
    } catch (e) {
      console.error("设置 Bottles 启用状态失败:", e);
    }
  }

  async function updateDefaultBottle(value: string) {
    setDefaultBottle(value);
    try {
      await import("@/lib/api").then((m) =>
        m.setIntegrationSettings({ key: "bottles", options: { defaultBottle: value || "" } }),
      );
    } catch (e) {
      console.error("设置默认 Bottle 失败:", e);
    }
  }

  useEffect(() => {
    if (open) {
      fetchEngines();
      fetchBottlesStatus();
      const handler = () => fetchEngines();
      window.addEventListener("gm:refresh-engines", handler);
      return () => window.removeEventListener("gm:refresh-engines", handler);
    }
  }, [open]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>管理</DialogTitle>
          <DialogDescription>应用运行与维护相关操作</DialogDescription>
        </DialogHeader>
        <ScrollArea className="max-h-[70vh] pr-3">
          <div className="space-y-4">
            <div className="flex items-center justify-between rounded-md border px-3 py-2">
              <div>
                <div className="text-sm font-medium">显示状态栏</div>
                <div className="text-xs text-muted-foreground">后台任务与进度信息展示</div>
              </div>
              <Switch
                checked={showStatusBar}
                onCheckedChange={(v) => onShowStatusBarChange?.(Boolean(v))}
              />
            </div>

            <Separator />

            <div className="space-y-2">
              <Button variant="secondary" className="w-full justify-start" onClick={onDownloadNwjs}>
                下载 NW.js
              </Button>
              <Button variant="secondary" className="w-full justify-start" onClick={onCleanupContainers}>
                清理无用容器
              </Button>
            </div>

            <Separator />

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="text-sm font-medium">已安装运行器</div>
                <Button variant="ghost" size="sm" disabled={loading} onClick={fetchEngines}>
                  刷新
                </Button>
              </div>
              {engines.length === 0 ? (
                <div className="text-xs text-muted-foreground">
                  暂无已安装运行器
                </div>
              ) : (
                engines.map((engine) => (
                  <div
                    key={engine.id}
                    className="flex items-center justify-between rounded-md border px-3 py-2 text-sm"
                  >
                    <div className="min-w-0">
                      <div className="truncate font-medium">{engine.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {engine.engineType} · {engine.version}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Button variant="ghost" size="sm" onClick={() => onUpdateEngine?.(engine)}>
                        更新
                      </Button>
                      <Button variant="destructive" size="sm" onClick={() => onRemoveEngine?.(engine)}>
                        卸载
                      </Button>
                    </div>
                  </div>
                ))
              )}
              <div className="text-xs text-muted-foreground">
                RPG Maker MV/MZ 使用 NW.js 运行时
              </div>
            </div>

            <Separator />

            {bottlesAvailable && (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="text-sm font-medium">Bottles</div>
                  <Button variant="ghost" size="sm" disabled={bottlesLoading} onClick={fetchBottlesStatus}>
                    刷新
                  </Button>
                </div>

                <div className="flex items-center justify-between rounded-md border px-3 py-2">
                  <div>
                    <div className="text-sm font-medium">启用 Bottles</div>
                    <div className="text-xs text-muted-foreground">用于 Windows EXE（Linux）</div>
                  </div>
                  <Switch
                    checked={bottlesEnabled}
                    disabled={bottlesLoading || !bottlesInstalled}
                    onCheckedChange={(v) => updateBottlesEnabled(Boolean(v))}
                  />
                </div>

                {!bottlesInstalled && (
                  <div className="rounded-md border px-3 py-2 text-xs text-muted-foreground">
                    <div className="mb-1">该功能在你的系统上不可用。</div>
                    <div>To add it, please run:</div>
                    <div className="mt-2 rounded bg-muted px-2 py-1 font-mono">
                      {bottlesInstallCommand}
                    </div>
                  </div>
                )}

                {bottlesInstalled && (
                  <div className="space-y-2">
                    <div className="text-xs text-muted-foreground">
                      选择默认 Bottle，用于 Windows EXE
                    </div>
                    <Select
                      value={defaultBottle}
                      onValueChange={updateDefaultBottle}
                      disabled={bottlesLoading || bottlesList.length === 0 || !bottlesEnabled}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="选择 Bottle" />
                      </SelectTrigger>
                      <SelectContent>
                        {bottlesList.map((name) => (
                          <SelectItem key={name} value={name}>
                            {name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                )}
              </div>
            )}
          </div>
        </ScrollArea>
        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange?.(false)}>
            关闭
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
