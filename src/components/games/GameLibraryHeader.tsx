import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useI18n } from "@/i18n";
import { Icon } from "@iconify/react";

interface GameLibraryHeaderProps {
  count: number;
  search: string;
  selectedEngine: string;
  engineOptions: Array<{ value: string; label: string }>;
  viewMode: "grid" | "list";
  onSearchChange?: (value: string) => void;
  onEngineChange?: (value: string) => void;
  onViewModeChange?: (mode: "grid" | "list") => void;
}

export default function GameLibraryHeader({
  count,
  search,
  selectedEngine,
  engineOptions,
  viewMode,
  onSearchChange,
  onEngineChange,
  onViewModeChange,
}: GameLibraryHeaderProps) {
  const { t } = useI18n();

  return (
    <div className="mb-5 flex items-center justify-between">
      <h1 className="text-2xl font-semibold">{t("library.title")}</h1>

      <div className="mx-4 flex flex-1 items-center">
        <div className="relative w-full max-w-lg">
          <Icon
            icon="ri:search-line"
            className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
          />
          <Input
            value={search}
            placeholder={t("library.searchPlaceholder")}
            className="h-8 rounded-md border pl-10"
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => onSearchChange?.(e.target.value)}
          />
        </div>
        <div className="ml-2 w-36 shrink-0">
          <Select value={selectedEngine} onValueChange={onEngineChange}>
            <SelectTrigger className="h-8">
              <SelectValue placeholder={t("library.enginePlaceholder")} />
            </SelectTrigger>
            <SelectContent>
              {engineOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="ml-2 flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            className={`h-8 w-8 ${viewMode === "grid" ? "bg-muted/70" : ""}`}
            title={t("library.gridView")}
            aria-pressed={viewMode === "grid"}
            onClick={() => onViewModeChange?.("grid")}
          >
            <Icon icon="ri:layout-grid-line" className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className={`h-8 w-8 ${viewMode === "list" ? "bg-muted/70" : ""}`}
            title={t("library.listView")}
            aria-pressed={viewMode === "list"}
            onClick={() => onViewModeChange?.("list")}
          >
            <Icon icon="ri:list-unordered" className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <span className="text-xs text-muted-foreground">{t("library.count", { count })}</span>
    </div>
  );
}
