import { useEffect, useMemo, useState } from "react";
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
import { Switch } from "@/components/ui/switch";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  ENGINE_OPTION_RPGMAKER_NWJS,
  ENGINE_PICKER_OPTIONS,
  normalizeEngineTypeForSelect,
  resolveSelectedEngineType,
} from "@/constants/engines";
import {
  getGameProfileDir,
  getGameSettings,
  getIntegrationStatus,
  openPath,
} from "@/lib/api";
import { useI18n } from "@/i18n";
import type { GameConfig, GameDto } from "@/types";

interface GameSettingsDialogProps {
  open: boolean;
  game: GameDto | null;
  loading?: boolean;
  onOpenChange?: (open: boolean) => void;
  onSave?: (payload: {
    id: string;
    title: string;
    engineType: string;
    path: string;
    runtimeVersion?: string;
    settings: GameConfig;
  }) => void;
  onRefreshCover?: (id: string) => void;
}

export default function GameSettingsDialog({
  open,
  game,
  loading = false,
  onOpenChange,
  onSave,
  onRefreshCover,
}: GameSettingsDialogProps) {
  const { t } = useI18n();
  const [title, setTitle] = useState("");
  const [engineType, setEngineType] = useState<string>("");
  const [path, setPath] = useState("");
  const [runtimeVersion, setRuntimeVersion] = useState("");
  const [entryPath, setEntryPath] = useState("");
  const [argsText, setArgsText] = useState("");
  const [sandboxHome, setSandboxHome] = useState(true);
  const [coverFile, setCoverFile] = useState("");

  const [settingsLoading, setSettingsLoading] = useState(false);
  const [settingsLoaded, setSettingsLoaded] = useState(false);

  const [bottlesLoading, setBottlesLoading] = useState(false);
  const [bottlesAvailable, setBottlesAvailable] = useState(false);
  const [bottlesInstalled, setBottlesInstalled] = useState(false);
  const [bottlesEnabled, setBottlesEnabled] = useState(false);
  const [bottlesList, setBottlesList] = useState<string[]>([]);
  const [defaultBottle, setDefaultBottle] = useState("");
  const [bottleName, setBottleName] = useState("");

  const isMvMz = useMemo(() => engineType === ENGINE_OPTION_RPGMAKER_NWJS, [engineType]);
  const requiresEntryPath = useMemo(() => engineType === "other", [engineType]);
  const canSave = useMemo(() => {
    const basicValid = !!game && title.trim().length > 0 && path.trim().length > 0;
    const entryValid = !requiresEntryPath || entryPath.trim().length > 0;
    return basicValid && entryValid && !settingsLoading;
  }, [game, title, path, requiresEntryPath, entryPath, settingsLoading]);

  // reset when dialog closes or game changes
  useEffect(() => {
    if (!open) {
      // clear transient fields
      setEntryPath("");
      setArgsText("");
      setSandboxHome(true);
      setCoverFile("");
      setBottleName("");
      setSettingsLoaded(false);
    }
  }, [open]);

  useEffect(() => {
    if (game) {
      setTitle(game.title);
      setEngineType(normalizeEngineTypeForSelect(game.engineType));
      setPath(game.path);
      setRuntimeVersion(game.runtimeVersion ?? "");
      // other values will be filled once settings load
    }
  }, [game]);

  // load persistent settings when dialog first opens for a game
  useEffect(() => {
    if (!open || !game || settingsLoaded) return;

    setSettingsLoading(true);
    getGameSettings(game.id)
      .then((config) => {
        setEngineType(normalizeEngineTypeForSelect(config.engineType || game.engineType));
        setEntryPath(
          config.entryPath ? toAbsoluteEntryPath(config.entryPath) : ""
        );
        setRuntimeVersion(config.runtimeVersion ?? game.runtimeVersion ?? "");
        setArgsText((config.args ?? []).join(" "));
        setSandboxHome(config.sandboxHome ?? true);
        setCoverFile(config.coverFile ?? "");
        setBottleName(config.bottleName ?? "");
        return refreshBottlesStatus();
      })
      .then(() => {
        if (!bottleName && defaultBottle) {
          setBottleName(defaultBottle);
        }
        setSettingsLoaded(true);
      })
      .catch((e) => {
        console.error("加载游戏设置失败:", e);
      })
      .finally(() => {
        setSettingsLoading(false);
      });
  }, [open, game, settingsLoaded]);

  // when engine type toggles to "other" while dialog is open, refresh bottles
  useEffect(() => {
    if (engineType === "other" && open) {
      refreshBottlesStatus();
    }
  }, [engineType, open]);

  async function handleSave() {
    if (!game) return;
    const resolvedEngineType = resolveSelectedEngineType(engineType, game.engineType);
    const args = argsText
      .split(/\s+/)
      .map((s) => s.trim())
      .filter(Boolean);

    const resolvedEntryPath = toAbsoluteEntryPath(
      entryPath.trim() || path.trim()
    );

    const usingBottles =
      resolvedEngineType === "other" &&
      bottlesAvailable &&
      bottlesEnabled &&
      bottlesInstalled;

    const settings: GameConfig = {
      engineType: resolvedEngineType,
      entryPath: resolvedEntryPath,
      runtimeVersion: runtimeVersion.trim() || undefined,
      args,
      sandboxHome,
      useBottles: usingBottles,
      bottleName: usingBottles ? (bottleName.trim() || undefined) : undefined,
      coverFile: coverFile.trim() || undefined,
    };

    onSave?.({
      id: game.id,
      title: title.trim(),
      engineType: resolvedEngineType,
      path: path.trim(),
      runtimeVersion: runtimeVersion.trim() || undefined,
      settings,
    });
  }

  async function refreshBottlesStatus() {
    if (!game) return;
    if (engineType !== "other") return;

    setBottlesLoading(true);
    try {
      const status = await getIntegrationStatus("bottles");
      const options = status.options ?? {};
      setBottlesAvailable(status.available);
      if (!status.available) {
        setBottlesInstalled(false);
        setBottlesEnabled(false);
        setBottlesList([]);
        setDefaultBottle("");
        return;
      }
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

  function isAbsolutePath(value: string): boolean {
    return value.startsWith("/") || /^[A-Za-z]:[\\/]/.test(value);
  }

  function joinPath(base: string, sub: string): string {
    if (base.includes("\\")) {
      return `${base.replace(/\\+$/, "")}\\${sub.replace(/^\\+/, "")}`;
    }
    return `${base.replace(/\/+$/, "")}/${sub.replace(/^\/+/, "")}`;
  }

  function toAbsoluteEntryPath(value: string): string {
    const trimmed = value.trim();
    if (!trimmed || !game) return trimmed;
    if (isAbsolutePath(trimmed)) return trimmed;
    return joinPath(game.path, trimmed);
  }

  async function pickEntryFile() {
    if (!game) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const res = await open({ title: t("import.pickExecutableTitle"), multiple: false });
      if (!res) return;
      const selected = Array.isArray(res) ? res[0] ?? "" : res;
      if (!selected) return;
      setEntryPath(selected);
    } catch (e) {
      console.error("选择可执行文件失败:", e);
    }
  }

  async function openGameDir() {
    if (!game) return;
    try {
      await openPath(game.path);
    } catch (e) {
      console.error("打开游戏目录失败:", e);
    }
  }

  async function openProfileDir() {
    if (!game) return;
    try {
      const profileDir = await getGameProfileDir(game.id);
      await openPath(profileDir);
    } catch (e) {
      console.error("打开Profile目录失败:", e);
    }
  }

  async function pickCoverFile() {
    if (!game) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const res = await open({ title: t("gameSettings.cover"), multiple: false });
      if (!res) return;
      const selected = Array.isArray(res) ? res[0] ?? "" : res;
      if (!selected) return;
      setCoverFile(selected);
    } catch (e) {
      console.error("选择图标失败:", e);
    }
  }

  function handleRefreshCover() {
    if (!game) return;
    onRefreshCover?.(game.id);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("gameSettings.title")}</DialogTitle>
          <DialogDescription>{t("gameSettings.description")}</DialogDescription>
        </DialogHeader>

        <ScrollArea className="max-h-[60vh] pr-3">
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t("gameSettings.name")}</label>
              <Input
                value={title}
                placeholder={t("gameSettings.namePlaceholder")}
                onChange={(e) => setTitle(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">{t("gameSettings.engineType")}</label>
              <Select value={engineType} onValueChange={(v) => setEngineType(v)}>
                <SelectTrigger>
                  <SelectValue placeholder={t("gameSettings.engineTypePlaceholder")} />
                </SelectTrigger>
                <SelectContent>
                  {ENGINE_PICKER_OPTIONS.map((engine) => (
                    <SelectItem key={engine.value} value={engine.value}>
                      {engine.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">{t("gameSettings.gamePath")}</label>
              <Input
                value={path}
                placeholder={t("gameSettings.gamePathPlaceholder")}
                onChange={(e) => setPath(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">{t("gameSettings.openDirs")}</label>
              <div className="flex flex-wrap gap-2">
                <Button variant="secondary" size="sm" onClick={openGameDir}>
                  {t("gameSettings.openGameDir")}
                </Button>
                <Button variant="secondary" size="sm" onClick={openProfileDir}>
                  {t("gameSettings.openProfileDir")}
                </Button>
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">{t("gameSettings.cover")}</label>
              <div className="flex items-center gap-2">
                <Input
                  value={coverFile}
                  placeholder={t("gameSettings.coverPlaceholder")}
                  onChange={(e) => setCoverFile(e.target.value)}
                />
                <Button
                  variant="secondary"
                  size="icon"
                  className="h-9 w-9"
                  onClick={pickCoverFile}
                >
                  …
                </Button>
              </div>
              <div className="flex flex-wrap gap-2">
                <Button
                  variant="secondary"
                  size="sm"
                  disabled={settingsLoading || !game}
                  onClick={handleRefreshCover}
                >
                  {t("gameSettings.extractCover")}
                </Button>
              </div>
              <div className="text-xs text-muted-foreground">
                {t("gameSettings.coverHint")}
              </div>
            </div>

            {isMvMz && (
              <div className="space-y-2">
                <label className="text-sm font-medium">
                  {t("gameSettings.nwjsVersion")}
                </label>
                <Input
                  value={runtimeVersion}
                  placeholder={t("gameSettings.nwjsVersionPlaceholder")}
                  onChange={(e) => setRuntimeVersion(e.target.value)}
                />
              </div>
            )}

            <div className="rounded-md border p-3">
              <div className="mb-2 text-sm font-medium">{t("gameSettings.runtime")}</div>
              {settingsLoading ? (
                <div className="text-xs text-muted-foreground">
                  {t("gameSettings.loading")}
                </div>
              ) : (
                <div className="space-y-3">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("gameSettings.entryPath")}</label>
                    <div className="flex gap-2">
                      <Input
                        value={entryPath}
                        placeholder={t("gameSettings.entryPathPlaceholder")}
                        onChange={(e) => setEntryPath(e.target.value)}
                      />
                      <Button variant="secondary" size="sm" onClick={pickEntryFile}>
                        {t("common.choose")}
                      </Button>
                    </div>
                    {engineType === "other" && bottlesEnabled && bottlesAvailable && (
                      <div className="text-xs text-muted-foreground">
                        {t("gameSettings.bottlesProgramHint")}
                      </div>
                    )}
                  </div>

                  {engineType === "other" && bottlesAvailable && (
                    <div className="space-y-2">
                      <div className="text-sm font-medium">{t("gameSettings.bottlesBottle")}</div>

                      {!bottlesInstalled ? (
                        <div className="rounded-md border px-3 py-2 text-xs text-muted-foreground">
                          <div className="mb-1">{t("gameSettings.bottlesUnavailable")}</div>
                          <div>{t("gameSettings.bottlesUnavailableHint")}</div>
                        </div>
                      ) : (
                        <div className="space-y-2">
                          <Select
                            value={bottleName}
                            onValueChange={(v) => setBottleName(v)}
                            disabled={
                              bottlesLoading || bottlesList.length === 0 || !bottlesEnabled
                            }
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
                          <div className="text-xs text-muted-foreground">
                            {t("gameSettings.defaultBottleHint")}
                          </div>
                        </div>
                      )}
                    </div>
                  )}

                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("gameSettings.args")}</label>
                    <Input
                      value={argsText}
                      placeholder={t("gameSettings.argsPlaceholder")}
                      onChange={(e) => setArgsText(e.target.value)}
                    />
                  </div>

                  <div className="flex items-center justify-between rounded-md border px-3 py-2">
                    <div>
                      <div className="text-sm font-medium">{t("gameSettings.sandboxHome")}</div>
                      <div className="text-xs text-muted-foreground">
                        {t("gameSettings.sandboxHomeDesc")}
                      </div>
                    </div>
                    <Switch
                      checked={sandboxHome}
                      onCheckedChange={(v) => setSandboxHome(Boolean(v))}
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        </ScrollArea>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange?.(false)}>
            {t("common.cancel")}
          </Button>
          <Button disabled={!canSave || loading} className="gap-2" onClick={handleSave}>
            {loading && <Icon icon="ri:loader-4-line" className="h-4 w-4 animate-spin" />}
            {t("common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
