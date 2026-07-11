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
  onImportMkxpz?: () => void;
  onUpdateEngine?: (engine: EngineDto) => void;
  onRemoveEngine?: (engine: EngineDto) => void;
}

export default function ManagementDialog({
  open,
  onOpenChange,
  onDownloadNwjs,
  onImportMkxpz,
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

  const nwjsEngines = engines.filter(e => e.engineType === "nwjs");
  const mkxpzEngines = engines.filter(e => e.engineType === "mkxpz");

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

  /** 运行器专用行 */
  function RuntimeRow({
    icon,
    name,
    engines,
    onInstall,
    onUpdate,
    onRemove,
  }: {
    icon: string;
    name: string;
    engines: EngineDto[];
    onInstall?: () => void;
    onUpdate?: (engine: EngineDto) => void;
    onRemove?: (engine: EngineDto) => void;
  }) {
    const installed = engines.length > 0;
    // 取最新安装的
    const latest = engines.sort((a, b) => b.installedAt - a.installedAt)[0];

    return (
      <div className="rounded border px-3 py-2 space-y-1">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 min-w-0">
            <Icon icon={icon} className="h-4 w-4 shrink-0" />
            <span className="text-sm font-medium truncate">{name}</span>
            {installed && (
              <span className="text-xs text-muted-foreground">v{latest.version}</span>
            )}
          </div>
          <div className="flex items-center gap-1 shrink-0">
            {installed ? (
              <>
                <Button variant="ghost" size="xs" onClick={() => onUpdate?.(latest)}>
                  <Icon icon="ri:refresh-line" className="h-3.5 w-3.5" />
                </Button>
                <Button variant="ghost" size="xs" className="text-destructive" onClick={() => onRemove?.(latest)}>
                  <Icon icon="ri:delete-bin-line" className="h-3.5 w-3.5" />
                </Button>
              </>
            ) : (
              <Button variant="outline" size="xs" onClick={onInstall}>
                <Icon icon="ri:download-line" className="h-3.5 w-3.5 mr-1" />
                {t("common.install")}
              </Button>
            )}
          </div>
        </div>
      </div>
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("maintenance.title")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-1">
          {/* 运行时区域 — 常驻显示 */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">{t("maintenance.runtimes")}</span>
              <Button variant="ghost" size="xs" disabled={loading} onClick={fetchEngines}>
                <Icon icon="ri:refresh-line" className="h-3.5 w-3.5" />
              </Button>
            </div>

            {/* NW.js 行 */}
            <RuntimeRow
              icon="ri:window-line"
              name="NW.js"
              engines={nwjsEngines}
              onInstall={onDownloadNwjs}
              onUpdate={onUpdateEngine}
              onRemove={onRemoveEngine}
            />

            {/* mkxp-z 行 */}
            <RuntimeRow
              icon="ri:gamepad-line"
              name="mkxp-z"
              engines={mkxpzEngines}
              onInstall={onImportMkxpz}
              onUpdate={onImportMkxpz}
              onRemove={onRemoveEngine}
            />
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
                      <SelectTrigger size="sm" className="w-35">
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
