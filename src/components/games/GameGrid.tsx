import { Card, CardContent } from "@/components/ui/card";
import GameCard from "./GameCard";
import type { GameDto } from "@/types";

interface GameGridProps {
  games: GameDto[];
  loading?: boolean;
  viewMode: "grid" | "list";
  onLaunch?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

export default function GameGrid({
  games,
  loading = false,
  viewMode,
  onLaunch,
  onEdit,
  onDelete,
}: GameGridProps) {
  const isEmpty = !loading && games.length === 0;
  const isGrid = viewMode === "grid";

  if (loading) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 8 }).map((_, i) => (
          <Card key={i} className="flex items-center gap-3 p-3">
            <div className="h-14 w-14 animate-pulse rounded-md bg-muted" />
            <div className="flex-1 space-y-2">
              <div className="h-3 w-1/3 animate-pulse rounded bg-muted" />
              <div className="h-2.5 w-1/4 animate-pulse rounded bg-muted" />
            </div>
            <div className="flex gap-1">
              <div className="h-8 w-8 animate-pulse rounded bg-muted" />
              <div className="h-8 w-8 animate-pulse rounded bg-muted" />
            </div>
          </Card>
        ))}
      </div>
    );
  }

  if (!isEmpty) {
    return (
      <div
        className={isGrid ? "grid gap-3 sm:grid-cols-2 xl:grid-cols-3" : "space-y-3"}
      >
        {games.map((game) => (
          <GameCard
            key={game.id}
            game={game}
            onLaunch={() => onLaunch?.(game.id)}
            onEdit={() => onEdit?.(game.id)}
            onDelete={() => onDelete?.(game.id)}
          />
        ))}
      </div>
    );
  }

  return (
    <Card className="mx-auto w-full max-w-lg">
      <CardContent className="flex flex-col items-center justify-center text-center">
        <div className="mb-4 text-6xl">🎮</div>
        <h3 className="mb-2 text-lg font-semibold">暂无游戏</h3>
        <p className="text-sm text-muted-foreground">
          点击右上角的按钮导入或扫描游戏
        </p>
      </CardContent>
    </Card>
  );
}
