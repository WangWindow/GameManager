import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
import { Switch } from "@/components/ui/switch";
import { cleanupUnusedContainers, getAppSettings, removeAllGames, setContainerRoot } from "@/lib/api";
import { useI18n } from "@/i18n";
import { useEffect, useState } from "react";
import { Icon } from "@iconify/react";
import { toast } from "sonner";

interface SettingsDialogProps {
  open: boolean;
  themeMode: "system" | "light" | "dark";
  showStatusBar: boolean;
  onOpenChange?: (open: boolean) => void;
  onThemeModeChange?: (mode: "system" | "light" | "dark") => void;
  onShowStatusBarChange?: (v: boolean) => void;
}

/** 简洁设置项 */
function SettingRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between py-2">
      <span className="text-sm">{label}</span>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

export default function SettingsDialog({
  open,
  themeMode,
  showStatusBar,
  onOpenChange,
  onThemeModeChange,
  onShowStatusBarChange,
}: SettingsDialogProps) {
  const { locale, setLocale, t } = useI18n();
  const [containerRoot, setContainerRootInput] = useState("");
  const [savingRoot, setSavingRoot] = useState(false);
  const [cleaningContainers, setCleaningContainers] = useState(false);
  const [removeAllConfirmOpen, setRemoveAllConfirmOpen] = useState(false);
  const [removingAllGames, setRemovingAllGames] = useState(false);

  useEffect(() => {
    if (!open) return;
    getAppSettings()
      .then((settings) => {
        setContainerRootInput(settings.containerRoot);
      })
      .catch((e) => {
        console.error("读取应用设置失败:", e);
      });
  }, [open]);

  async function pickContainerRoot() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const path = await open({
        directory: true,
        multiple: false,
        title: t("settings.containerRootDialogTitle"),
      });
      if (!path) return;
      const selected = Array.isArray(path) ? path[0] ?? "" : path;
      if (!selected) return;
      setContainerRootInput(selected);
    } catch (e) {
      console.error("选择容器目录失败:", e);
    }
  }

  async function saveContainerRootSetting() {
    if (!containerRoot.trim()) return;
    setSavingRoot(true);
    try {
      await setContainerRoot({ containerRoot: containerRoot.trim() });
    } catch (e) {
      console.error("保存容器根目录失败:", e);
    } finally {
      setSavingRoot(false);
    }
  }

  async function cleanupContainers() {
    if (cleaningContainers) return;
    setCleaningContainers(true);
    try {
      const result = await cleanupUnusedContainers();
      toast.success(t("maintenance.toastCleanupDone").replace("{{count}}", String(result.deleted)));
      window.dispatchEvent(new CustomEvent("gm:refresh-games"));
    } catch (e) {
      const msg = e instanceof Error ? e.message : t("maintenance.toastCleanupFailed");
      toast.error(msg);
    } finally {
      setCleaningContainers(false);
    }
  }

  async function confirmRemoveAllGames() {
    if (removingAllGames) return;
    setRemovingAllGames(true);
    try {
      const count = await removeAllGames();
      toast.success(t("maintenance.toastRemoveAllGamesDone").replace("{{count}}", String(count)));
      window.dispatchEvent(new CustomEvent("gm:refresh-games"));
      setRemoveAllConfirmOpen(false);
    } catch (e) {
      const msg = e instanceof Error ? e.message : t("maintenance.toastRemoveAllGamesFailed");
      toast.error(msg);
    } finally {
      setRemovingAllGames(false);
    }
  }

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t("settings.title")}</DialogTitle>
          </DialogHeader>

          <div className="divide-y">
            {/* 外观 */}
            <SettingRow label={t("settings.theme")}>
              <Select
                value={themeMode}
                onValueChange={(v) => onThemeModeChange?.(v as "system" | "light" | "dark")}
              >
                <SelectTrigger className="w-24" size="sm">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="system">{t("settings.theme.system")}</SelectItem>
                  <SelectItem value="light">{t("settings.theme.light")}</SelectItem>
                  <SelectItem value="dark">{t("settings.theme.dark")}</SelectItem>
                </SelectContent>
              </Select>
            </SettingRow>

            <SettingRow label={t("settings.language")}>
              <Select value={locale} onValueChange={(v) => setLocale(v as "zh-CN" | "en-US")}>
                <SelectTrigger className="w-24" size="sm">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="zh-CN">中文</SelectItem>
                  <SelectItem value="en-US">English</SelectItem>
                </SelectContent>
              </Select>
            </SettingRow>

            <SettingRow label={t("settings.statusBar")}>
              <Switch
                checked={showStatusBar}
                onCheckedChange={(v) => onShowStatusBarChange?.(Boolean(v))}
              />
            </SettingRow>

            {/* 存储路径 */}

            {/* 存储路径 */}
            <div className="space-y-2 py-3">
              <span className="text-sm">{t("settings.containerRoot")}</span>
              <div className="flex gap-2">
                <Input
                  value={containerRoot}
                  onChange={(e) => setContainerRootInput(e.target.value)}
                  placeholder={t("settings.containerRootPlaceholder")}
                  className="flex-1 h-8 text-sm"
                />
                <Button variant="outline" size="sm" className="h-8 px-2" onClick={pickContainerRoot}>
                  <Icon icon="ri:folder-open-line" className="h-4 w-4" />
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  className="h-8"
                  disabled={savingRoot || !containerRoot.trim()}
                  onClick={saveContainerRootSetting}
                >
                  {savingRoot ? (
                    <Icon icon="ri:loader-4-line" className="h-4 w-4 animate-spin" />
                  ) : (
                    t("common.save")
                  )}
                </Button>
              </div>
            </div>

            {/* 危险操作 */}
            <div className="py-3 border-t">
              <div className="flex items-center justify-between py-2 gap-3">
                <span className="text-sm text-destructive">{t("maintenance.removeAllGames")}</span>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={cleaningContainers}
                    onClick={cleanupContainers}
                  >
                    {cleaningContainers ? (
                      <Icon icon="ri:loader-4-line" className="h-3.5 w-3.5 mr-1 animate-spin" />
                    ) : (
                      <Icon icon="ri:broom-line" className="h-3.5 w-3.5 mr-1" />
                    )}
                    {t("maintenance.cleanupContainers")}
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => setRemoveAllConfirmOpen(true)}
                  >
                    <Icon icon="ri:delete-bin-6-line" className="h-3.5 w-3.5 mr-1" />
                    {t("maintenance.removeAllGamesConfirmAction")}
                  </Button>
                </div>
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button variant="ghost" size="sm" onClick={() => onOpenChange?.(false)}>
              {t("common.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog open={removeAllConfirmOpen} onOpenChange={setRemoveAllConfirmOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t("maintenance.removeAllGamesConfirmTitle")}</DialogTitle>
            <DialogDescription>
              {t("maintenance.removeAllGamesConfirmDescription")}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="ghost" disabled={removingAllGames} onClick={() => setRemoveAllConfirmOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" disabled={removingAllGames} onClick={confirmRemoveAllGames}>
              {removingAllGames && <Icon icon="ri:loader-4-line" className="h-4 w-4 animate-spin mr-1" />}
              {t("maintenance.removeAllGamesConfirmAction")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
