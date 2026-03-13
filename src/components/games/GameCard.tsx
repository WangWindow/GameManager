import { useMemo } from "react";
import { Icon } from "@iconify/react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { getEngineDisplayName, getEngineIcon } from "@/constants/engines";
import { useI18n } from "@/i18n";
import { formatRelativeTime } from "@/lib/utils";
import type { GameDto } from "@/types";

interface GameCardProps {
  game: GameDto;
  onLaunch?: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
}

export default function GameCard({ game, onLaunch, onEdit, onDelete }: GameCardProps) {
  const { t } = useI18n();
  const coverSrc = useMemo(() => {
    if (!game.coverPath) return "";
    try {
      return convertFileSrc(game.coverPath);
    } catch {
      return `asset://localhost/${game.coverPath}`;
    }
  }, [game.coverPath]);

  return (
    <div className="group relative flex items-center gap-3 rounded-xl border bg-card px-4 py-2.5 text-card-foreground transition-all hover:bg-muted/40 hover:shadow-md">
      {/* 封面图 */}
      <div className="h-12 w-12 shrink-0 overflow-hidden rounded-md bg-muted">
        {game.coverPath ? (
          <img
            src={coverSrc}
            alt={game.title}
            className="h-full w-full object-cover"
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center bg-linear-to-br from-muted to-muted/50">
            <Icon
              icon={getEngineIcon(game.engineType)}
              className="h-6 w-6 text-muted-foreground/40"
            />
          </div>
        )}
      </div>

      {/* 游戏信息 */}
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <div className="flex items-center gap-2">
          <h3 className="truncate text-sm font-semibold" title={game.title}>
            {game.title}
          </h3>
          {!game.pathValid && (
            <Badge variant="destructive" className="h-4 px-2 text-[10px]">
              {t("game.pathInvalid")}
            </Badge>
          )}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Icon icon={getEngineIcon(game.engineType)} className="h-3.5 w-3.5" />
          <span className="truncate">{getEngineDisplayName(game.engineType)}</span>
          {game.lastPlayedAt && (
            <span className="truncate">· {formatRelativeTime(game.lastPlayedAt)}</span>
          )}
        </div>
      </div>

      {/* 操作按钮 */}
      <div className="flex shrink-0 items-center gap-2">
        <Button size="icon" className="h-7 w-7" title={t("game.launch")} onClick={onLaunch}>
          <Icon icon="ri:play-fill" className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="secondary"
          size="icon"
          className="h-7 w-7"
          title={t("game.edit")}
          onClick={onEdit}
        >
          <Icon icon="ri:settings-3-line" className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7 text-muted-foreground hover:text-destructive"
          title={t("game.delete")}
          onClick={onDelete}
        >
          <Icon icon="ri:delete-bin-line" className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}
