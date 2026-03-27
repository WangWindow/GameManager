import { useEffect, useState } from "react";
import { Icon } from "@iconify/react";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
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

  /** 简化布局行 */
  const FormRow = ({ label, children }: { label: string; children: React.ReactNode }) => (
    <div className="flex items-center justify-between gap-4">
      <span className="text-sm text-muted-foreground shrink-0">{label}</span>
      <div className="flex-1 flex justify-end">{children}</div>
    </div>
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("maintenance.title")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-1">
          {/* 快速操作 */}
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" size="sm" onClick={onDownloadNwjs}>
              <Icon icon="ri:download-line" className="h-3.5 w-3.5 mr-1" />
              {t("maintenance.installNwjs")}
            </Button>
            <Button variant="outline" size="sm" onClick={onCleanupOldNwjs}>
              <Icon icon="ri:delete-bin-line" className="h-3.5 w-3.5 mr-1" />
              {t("maintenance.cleanupOld")}
            </Button>
            <Button variant="outline" size="sm" onClick={onCleanupContainers}>
              <Icon icon="ri:folder-reduce-line" className="h-3.5 w-3.5 mr-1" />
              {t("maintenance.cleanupContainers")}
            </Button>
          </div>

          {/* 已安装运行时 */}
          <div className="pt-2 border-t space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">{t("maintenance.runtimes")}</span>
              <Button variant="ghost" size="xs" disabled={loading} onClick={fetchEngines}>
                <Icon icon="ri:refresh-line" className="h-3.5 w-3.5" />
              </Button>
            </div>
            {engines.length === 0 ? (
              <div className="text-xs text-muted-foreground py-2">
                {t("maintenance.noRuntimes")}
              </div>
            ) : (
              <div className="space-y-1">
                {engines.map((engine) => (
                  <div
                    key={engine.id}
                    className="flex items-center justify-between rounded border px-2 py-1.5 text-sm"
                  >
                    <div className="min-w-0">
                      <span className="truncate">{engine.name}</span>
                      <span className="text-xs text-muted-foreground ml-2">v{engine.version}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Button variant="ghost" size="xs" onClick={() => onUpdateEngine?.(engine)}>
                        <Icon icon="ri:refresh-line" className="h-3.5 w-3.5" />
                      </Button>
                      <Button variant="ghost" size="xs" className="text-destructive" onClick={() => onRemoveEngine?.(engine)}>
                        <Icon icon="ri:delete-bin-line" className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Bottles 集成 */}
          {bottlesAvailable && (
            <div className="pt-2 border-t space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Bottles</span>
                <Button variant="ghost" size="xs" disabled={bottlesLoading} onClick={fetchBottlesStatus}>
                  <Icon icon="ri:refresh-line" className="h-3.5 w-3.5" />
                </Button>
              </div>

              {!bottlesInstalled ? (
                <div className="rounded border px-2 py-2 text-xs text-muted-foreground">
                  <div>{t("maintenance.bottlesNotInstalled")}</div>
                  <code className="block mt-1 bg-muted px-2 py-1 rounded text-[10px]">
                    {bottlesInstallCommand}
                  </code>
                </div>
              ) : (
                <>
                  <FormRow label={t("maintenance.enableBottles")}>
                    <Switch
                      checked={bottlesEnabled}
                      disabled={bottlesLoading}
                      onCheckedChange={(v: boolean) => updateBottlesEnabled(Boolean(v))}
                    />
                  </FormRow>

                  <FormRow label={t("maintenance.defaultBottle")}>
                    <Select
                      value={defaultBottle}
                      onValueChange={updateDefaultBottle}
                      disabled={bottlesLoading || bottlesList.length === 0 || !bottlesEnabled}
                    >
                      <SelectTrigger size="sm" className="w-[140px]">
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
                  </FormRow>
                </>
              )}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="ghost" size="sm" onClick={() => onOpenChange?.(false)}>
            {t("common.close")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
