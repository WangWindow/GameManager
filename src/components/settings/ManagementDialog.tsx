import { useEffect, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { getEngines, getIntegrationStatus, setIntegrationSettings } from "@/lib/api";
import { useI18n } from "@/i18n";
import type { EngineDto } from "@/types";

interface ManagementDialogProps {
  open: boolean;
  onOpenChange?: (open: boolean) => void;
  onDownloadNwjs?: () => void;
  onCleanupOldNwjs?: () => void;
  onCleanupContainers?: () => void;
  onUpdateEngine?: (engine: EngineDto) => void;
  onRemoveEngine?: (engine: EngineDto) => void;
}

export default function ManagementDialog({
  open,
  onOpenChange,
  onDownloadNwjs,
  onCleanupOldNwjs,
  onCleanupContainers,
  onUpdateEngine,
  onRemoveEngine,
}: ManagementDialogProps) {
  const { t } = useI18n();
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
      const list = await getEngines();
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
      const status = await getIntegrationStatus("bottles");
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
      await setIntegrationSettings({ key: "bottles", enabled: value });
    } catch (e) {
      console.error("设置 Bottles 启用状态失败:", e);
    }
  }

  async function updateDefaultBottle(value: string) {
    setDefaultBottle(value);
    try {
      await setIntegrationSettings({ key: "bottles", options: { defaultBottle: value || "" } });
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
          <DialogTitle>{t("maintenance.title")}</DialogTitle>
          <DialogDescription>{t("maintenance.description")}</DialogDescription>
        </DialogHeader>
        <ScrollArea className="max-h-[60vh] pr-3">
          <div className="space-y-4">
            <div className="space-y-2">
              <Button variant="secondary" className="w-full justify-start" onClick={onDownloadNwjs}>
                {t("maintenance.installOrUpdateNwjs")}
              </Button>
              <Button variant="secondary" className="w-full justify-start" onClick={onCleanupOldNwjs}>
                {t("maintenance.cleanupOldNwjs")}
              </Button>
              <Button variant="secondary" className="w-full justify-start" onClick={onCleanupContainers}>
                {t("maintenance.cleanupUnusedContainers")}
              </Button>
            </div>

            <Separator />

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="text-sm font-medium">{t("maintenance.installedRuntimes")}</div>
                <Button variant="ghost" size="sm" disabled={loading} onClick={fetchEngines}>
                  {t("common.refresh")}
                </Button>
              </div>
              {engines.length === 0 ? (
                <div className="text-xs text-muted-foreground">
                  {t("maintenance.noRuntimes")}
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
                        {t("maintenance.runtimeMeta", {
                          type: engine.engineType,
                          version: engine.version,
                        })}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Button variant="ghost" size="sm" onClick={() => onUpdateEngine?.(engine)}>
                        {t("common.update")}
                      </Button>
                      <Button variant="destructive" size="sm" onClick={() => onRemoveEngine?.(engine)}>
                        {t("common.uninstall")}
                      </Button>
                    </div>
                  </div>
                ))
              )}
              <div className="text-xs text-muted-foreground">
                {t("maintenance.nwjsHint")}
              </div>
            </div>

            <Separator />

            {bottlesAvailable && (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="text-sm font-medium">{t("maintenance.bottles")}</div>
                  <Button variant="ghost" size="sm" disabled={bottlesLoading} onClick={fetchBottlesStatus}>
                    {t("common.refresh")}
                  </Button>
                </div>

                <div className="flex items-center justify-between rounded-md border px-3 py-2">
                  <div>
                    <div className="text-sm font-medium">{t("maintenance.enableBottles")}</div>
                    <div className="text-xs text-muted-foreground">{t("maintenance.enableBottlesDesc")}</div>
                  </div>
                  <Switch
                    checked={bottlesEnabled}
                    disabled={bottlesLoading || !bottlesInstalled}
                    onCheckedChange={(v: boolean) => updateBottlesEnabled(Boolean(v))}
                  />
                </div>

                {!bottlesInstalled && (
                  <div className="rounded-md border px-3 py-2 text-xs text-muted-foreground">
                    <div className="mb-1">{t("maintenance.unavailable")}</div>
                    <div>{t("maintenance.installHint")}</div>
                    <div className="mt-2 rounded bg-muted px-2 py-1 font-mono">
                      {bottlesInstallCommand}
                    </div>
                  </div>
                )}

                {bottlesInstalled && (
                  <div className="space-y-2">
                    <div className="text-xs text-muted-foreground">
                      {t("maintenance.defaultBottle")}
                    </div>
                    <Select
                      value={defaultBottle}
                      onValueChange={updateDefaultBottle}
                      disabled={bottlesLoading || bottlesList.length === 0 || !bottlesEnabled}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t("maintenance.selectBottle")} />
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
            {t("common.close")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
