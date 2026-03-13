import { useState, useCallback } from "react";
import { getGames, addGame, updateGame, deleteGame, launchGame } from "@/lib/api";
import type { GameDto, AddGameInput, UpdateGameInput } from "@/types";

const GAMES_CACHE_TTL = 10_000;

let gamesCache: {
  expiresAt: number;
  data: GameDto[];
} | null = null;

function writeGamesCache(data: GameDto[]) {
  gamesCache = {
    data,
    expiresAt: Date.now() + GAMES_CACHE_TTL,
  };
}

function readGamesCache(): GameDto[] | null {
  if (!gamesCache) return null;
  if (gamesCache.expiresAt < Date.now()) {
    gamesCache = null;
    return null;
  }
  return gamesCache.data;
}

function invalidateGamesCache() {
  gamesCache = null;
}

function mergeAndSortGames(games: GameDto[]): GameDto[] {
  return [...games].sort((a, b) => {
    const left = a.lastPlayedAt ?? a.createdAt;
    const right = b.lastPlayedAt ?? b.createdAt;
    return right - left;
  });
}

export function useGames() {
  const [games, setGames] = useState<GameDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchGames = useCallback(async (force = false) => {
    if (!force) {
      const cached = readGamesCache();
      if (cached) {
        setGames(cached);
        return;
      }
    }

    setLoading(true);
    setError(null);
    try {
      const data = await getGames();
      const merged = mergeAndSortGames(data);
      setGames(merged);
      writeGamesCache(merged);
    } catch (e) {
      const msg = e instanceof Error ? e.message : "获取游戏列表失败";
      setError(msg);
      console.error("获取游戏列表失败:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const handleAddGame = useCallback(async (input: AddGameInput): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      const newGame = await addGame(input);
      setGames((prev) => {
        const next = mergeAndSortGames([...prev, newGame]);
        writeGamesCache(next);
        return next;
      });
      return true;
    } catch (e) {
      const msg = e instanceof Error ? e.message : "添加游戏失败";
      setError(msg);
      console.error("添加游戏失败:", e);
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const handleUpdateGame = useCallback(
    async (id: string, input: UpdateGameInput): Promise<boolean> => {
      setLoading(true);
      setError(null);
      try {
        const updatedGame = await updateGame(id, input);
        setGames((prev) => {
          const next = mergeAndSortGames(prev.map((g) => (g.id === id ? updatedGame : g)));
          writeGamesCache(next);
          return next;
        });
        return true;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "更新游戏失败";
        setError(msg);
        console.error("更新游戏失败:", e);
        return false;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const handleDeleteGame = useCallback(async (id: string): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      await deleteGame(id);
      setGames((prev) => {
        const next = prev.filter((g) => g.id !== id);
        writeGamesCache(next);
        return next;
      });
      return true;
    } catch (e) {
      const msg = e instanceof Error ? e.message : "删除游戏失败";
      setError(msg);
      console.error("删除游戏失败:", e);
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const handleLaunchGame = useCallback(async (id: string): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      await launchGame(id);
      invalidateGamesCache();
      await fetchGames(true);
      return true;
    } catch (e) {
      const msg = e instanceof Error ? e.message : "启动游戏失败";
      setError(msg);
      console.error("启动游戏失败:", e);
      return false;
    } finally {
      setLoading(false);
    }
  }, [fetchGames]);

  const getGameById = useCallback(
    (id: string) => games.find((g) => g.id === id) ?? null,
    [games],
  );

  return {
    games,
    loading,
    error,
    fetchGames,
    handleAddGame,
    handleUpdateGame,
    handleDeleteGame,
    handleLaunchGame,
    getGameById,
  };
}
