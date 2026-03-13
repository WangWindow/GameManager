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
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { ScrollArea } from "@/components/ui/scroll-area";
import { getAppSettings, setContainerRoot, setNwjsKeepLatestOnly } from "@/lib/api";
import { useI18n } from "@/i18n";
import { useEffect, useState } from "react";

interface SettingsDialogProps {
  open: boolean;
  themeMode: "system" | "light" | "dark";
  showStatusBar: boolean;
  onOpenChange?: (open: boolean) => void;
  onThemeModeChange?: (mode: "system" | "light" | "dark") => void;
  onShowStatusBarChange?: (v: boolean) => void;
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
  const [nwjsKeepLatestOnly, setNwjsKeepLatestOnlyState] = useState(true);
  const [savingRoot, setSavingRoot] = useState(false);

  useEffect(() => {
    if (!open) return;
    getAppSettings()
      .then((settings) => {
        setContainerRootInput(settings.containerRoot);
        setNwjsKeepLatestOnlyState(settings.nwjsKeepLatestOnly);
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

  async function updateNwjsKeepLatestOnly(value: boolean) {
    setNwjsKeepLatestOnlyState(value);
    try {
      await setNwjsKeepLatestOnly(value);
    } catch (e) {
      console.error("保存 NW.js 保留策略失败:", e);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("settings.title")}</DialogTitle>
          <DialogDescription>{t("settings.description")}</DialogDescription>
        </DialogHeader>

        <ScrollArea className="max-h-[60vh] pr-2">
          <div className="space-y-4">
            <div className="flex items-center justify-between rounded-md border px-3 py-2">
              <div>
                <div className="text-sm font-medium">{t("settings.language")}</div>
                <div className="text-xs text-muted-foreground">{t("settings.languageDescription")}</div>
              </div>
              <Select value={locale} onValueChange={(v) => setLocale(v as "zh-CN" | "en-US")}>
                <SelectTrigger>
                  <SelectValue placeholder={t("settings.language")} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="zh-CN">{t("settings.language.zh-CN")}</SelectItem>
                  <SelectItem value="en-US">{t("settings.language.en-US")}</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between rounded-md border px-3 py-2">
              <div>
                <div className="text-sm font-medium">{t("settings.theme")}</div>
                <div className="text-xs text-muted-foreground">{t("settings.themeDescription")}</div>
              </div>
              <Select
                value={themeMode}
                onValueChange={(v) => onThemeModeChange?.(v as "system" | "light" | "dark")}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t("settings.theme")} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="system">{t("settings.theme.system")}</SelectItem>
                  <SelectItem value="light">{t("settings.theme.light")}</SelectItem>
                  <SelectItem value="dark">{t("settings.theme.dark")}</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between rounded-md border px-3 py-2">
              <div>
                <div className="text-sm font-medium">{t("settings.statusBar")}</div>
                <div className="text-xs text-muted-foreground">{t("settings.statusBarDescription")}</div>
              </div>
              <Switch
                checked={showStatusBar}
                onCheckedChange={(v) => onShowStatusBarChange?.(Boolean(v))}
              />
            </div>

            <div className="space-y-2 rounded-md border px-3 py-2">
              <div>
                <div className="text-sm font-medium">{t("settings.containerRoot")}</div>
                <div className="text-xs text-muted-foreground">
                  {t("settings.containerRootDescription")}
                </div>
              </div>
              <div className="flex gap-2">
                <Input
                  value={containerRoot}
                  onChange={(e) => setContainerRootInput(e.target.value)}
                  placeholder={t("settings.containerRootPlaceholder")}
                />
                <Button variant="secondary" onClick={pickContainerRoot}>
                  {t("common.browse")}
                </Button>
              </div>
              <Button
                variant="secondary"
                className="w-full"
                disabled={savingRoot || !containerRoot.trim()}
                onClick={saveContainerRootSetting}
              >
                {t("settings.saveContainerRoot")}
              </Button>
            </div>

            <div className="flex items-center justify-between rounded-md border px-3 py-2">
              <div>
                <div className="text-sm font-medium">{t("settings.nwjsKeepLatest")}</div>
                <div className="text-xs text-muted-foreground">{t("settings.nwjsKeepLatestDescription")}</div>
              </div>
              <Switch
                checked={nwjsKeepLatestOnly}
                onCheckedChange={(v) => updateNwjsKeepLatestOnly(Boolean(v))}
              />
            </div>

            <Separator />
            <div className="text-xs text-muted-foreground">{t("settings.maintenanceHint")}</div>
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
