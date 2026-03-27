import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
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
import { getAppSettings, setContainerRoot, setNwjsKeepLatestOnly } from "@/lib/api";
import { useI18n } from "@/i18n";
import { useEffect, useState } from "react";
import { Icon } from "@iconify/react";

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

          <SettingRow label={t("settings.nwjsKeepLatest")}>
            <Switch
              checked={nwjsKeepLatestOnly}
              onCheckedChange={(v) => updateNwjsKeepLatestOnly(Boolean(v))}
            />
          </SettingRow>

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
