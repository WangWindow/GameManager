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
import { getEngineRegistryDetail, getEngineProfileDetail, setEngineEnabled } from "@/lib/api";
import { useI18n } from "@/i18n";
import type { EngineDetail, EngineProfileDetail } from "@/types";

interface PluginsDialogProps {
  open: boolean;
  onOpenChange?: (open: boolean) => void;
}

export default function PluginsDialog({ open, onOpenChange }: PluginsDialogProps) {
  const { t } = useI18n();
  const [plugins, setPlugins] = useState<EngineDetail[]>([]);
  const [loading, setLoading] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [detailCache, setDetailCache] = useState<Map<string, EngineProfileDetail>>(new Map());
  const [detailLoading, setDetailLoading] = useState<string | null>(null);

  async function fetchPlugins() {
    setLoading(true);
    try {
      const list = await getEngineRegistryDetail();
      setPlugins(list);
    } catch (e) {
      console.error("获取插件列表失败:", e);
    } finally {
      setLoading(false);
    }
  }

  async function togglePlugin(id: string, enabled: boolean) {
    try {
      await setEngineEnabled(id, enabled);
      setPlugins((prev) => prev.map((p) => (p.id === id ? { ...p, enabled } : p)));
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    } catch (e) {
      console.error("切换插件状态失败:", e);
    }
  }

  async function toggleDetail(id: string) {
    if (expandedId === id) {
      setExpandedId(null);
      return;
    }
    setExpandedId(id);
    if (!detailCache.has(id)) {
      setDetailLoading(id);
      try {
        const detail = await getEngineProfileDetail(id);
        setDetailCache((prev) => new Map(prev).set(id, detail));
      } catch (e) {
        console.error("获取插件详情失败:", e);
      } finally {
        setDetailLoading(null);
      }
    }
  }

  useEffect(() => {
    if (!open) return;
    setExpandedId(null);
    setDetailLoading(null);
    setDetailCache(new Map());
    void fetchPlugins();
  }, [open]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Icon icon="ri:plugin-line" className="h-5 w-5" />
            {t("plugins.title")}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-2">
          <div className="flex items-center justify-end">
            <Button variant="ghost" size="sm" className="h-6 px-2 text-xs" onClick={fetchPlugins} disabled={loading}>
              <Icon icon={loading ? "ri:loader-4-line" : "ri:refresh-line"} className={loading ? "animate-spin h-3 w-3" : "h-3 w-3"} />
            </Button>
          </div>

          {plugins.length === 0 ? (
            <p className="py-4 text-center text-sm text-muted-foreground">{t("plugins.empty")}</p>
          ) : (
            <div className="max-h-96 space-y-1.5 overflow-y-auto">
              {plugins.map((p) => {
                const isExpanded = expandedId === p.id;
                const detail = detailCache.get(p.id);
                const isLoading = detailLoading === p.id;

                return (
                  <div key={p.id}>
                    <div className="flex items-center justify-between rounded-md border px-3 py-2.5 text-sm">
                      <button
                        className="flex flex-1 items-center gap-2 min-w-0 text-left"
                        onClick={() => toggleDetail(p.id)}
                      >
                        <Icon icon={p.icon} className="h-4 w-4 shrink-0 text-muted-foreground" />
                        <div className="min-w-0">
                          <div className="flex items-center gap-1.5">
                            <span className="truncate font-medium">{p.name}</span>
                            {!p.valid && (
                              <span className="shrink-0 rounded bg-destructive/10 px-1.5 py-0.5 text-[10px] text-destructive" title={p.errors.join("; ")}>
                                {t("plugins.invalid")}
                              </span>
                            )}
                          </div>
                          <span className="text-[10px] text-muted-foreground">
                            {t("plugins.ruleCountStrategy", { count: p.ruleCount, strategy: p.strategy })}
                          </span>
                        </div>
                        <Icon
                          icon={isExpanded ? "ri:arrow-up-s-line" : "ri:arrow-down-s-line"}
                          className="h-4 w-4 shrink-0 text-muted-foreground ml-1"
                        />
                      </button>
                      <Switch
                        checked={p.enabled}
                        onCheckedChange={(v) => togglePlugin(p.id, v)}
                        disabled={!p.valid}
                        className="ml-2 shrink-0"
                      />
                    </div>

                    {isExpanded && (
                      <div className="mx-4 mt-1 rounded-md border bg-muted/40 px-3 py-2 text-xs">
                        {isLoading ? (
                          <div className="flex items-center gap-2 py-2 text-muted-foreground">
                            <Icon icon="ri:loader-4-line" className="animate-spin h-3 w-3" />
                            {t("common.loading")}
                          </div>
                        ) : detail ? (
                          <div className="space-y-2">
                            <div>
                              <span className="font-semibold">{t("plugins.detectionRules")}</span>
                              <span className="ml-1 text-muted-foreground">{t("plugins.minScore", { score: detail.detection.minScore })}</span>
                              <div className="mt-1 space-y-0.5">
                                {detail.detection.rules.map((r, i) => (
                                  <div key={i} className="flex items-center gap-2 text-muted-foreground">
                                    <span className="shrink-0 rounded bg-muted px-1 py-0.5 text-[9px]">{r.ruleType}</span>
                                    <span className="truncate">{r.path || r.pattern || r.ext}</span>
                                    <span className="shrink-0 ml-auto">×{r.weight}</span>
                                  </div>
                                ))}
                              </div>
                            </div>
                            <div>
                              <span className="font-semibold">{t("plugins.launchConfig")}</span>
                              <div className="mt-1 space-y-0.5 text-muted-foreground">
                                <div>{t("plugins.strategy")}: {detail.launch.strategy}</div>
                                {detail.launch.entryPatterns.length > 0 && (
                                  <div>{t("plugins.entry")}: {detail.launch.entryPatterns.join(", ")}</div>
                                )}
                                {detail.launch.excludePatterns.length > 0 && (
                                  <div>{t("plugins.exclude")}: {detail.launch.excludePatterns.join(", ")}</div>
                                )}
                                {detail.launch.runtimeId && <div>{t("plugins.runtime")}: {detail.launch.runtimeId}</div>}
                                {detail.launch.program && <div>{t("plugins.externalProgram")}: {detail.launch.program}</div>}
                              </div>
                            </div>
                            {detail.errors.length > 0 && (
                              <div className="text-destructive">{t("plugins.errors")}: {detail.errors.join("; ")}</div>
                            )}
                          </div>
                        ) : (
                          <span className="text-muted-foreground">{t("plugins.loadFailed")}</span>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
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
