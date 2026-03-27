/**
 * 游戏数据管理 hook
 *
 * 提供游戏列表的 CRUD 操作和启动功能，包含本地缓存机制以提升性能
 */

import { useState, useCallback } from "react";
import { getGames, addGame, updateGame, deleteGame, launchGame } from "@/lib/api";
import type { GameDto, AddGameInput, UpdateGameInput } from "@/types";

/** 游戏列表缓存过期时间 (10秒) */
const GAMES_CACHE_TTL = 10_000;

/** 游戏列表缓存结构 */
interface GamesCache {
  expiresAt: number;
  data: GameDto[];
}

let gamesCache: GamesCache | null = null;

/** 写入缓存 */
function writeGamesCache(data: GameDto[]) {
  gamesCache = {
    data,
    expiresAt: Date.now() + GAMES_CACHE_TTL,
  };
}

/** 读取缓存 (过期则返回 null) */
function readGamesCache(): GameDto[] | null {
  if (!gamesCache) return null;
  if (gamesCache.expiresAt < Date.now()) {
    gamesCache = null;
    return null;
  }
  return gamesCache.data;
}

/** 使缓存失效 */
function invalidateGamesCache() {
  gamesCache = null;
}

/** 按最后游玩时间排序游戏列表 */
function mergeAndSortGames(games: GameDto[]): GameDto[] {
  return [...games].sort((a, b) => {
    const left = a.lastPlayedAt ?? a.createdAt;
    const right = b.lastPlayedAt ?? b.createdAt;
    return right - left;
  });
}

/**
 * 游戏数据管理 hook
 *
 * @returns 游戏列表状态和操作函数
 *
 * @example
 * const { games, loading, fetchGames, handleLaunchGame } = useGames()
 *
 * // 获取游戏列表
 * useEffect(() => { fetchGames() }, [fetchGames])
 *
 * // 启动游戏
 * const handleClick = () => handleLaunchGame(game.id)
 */
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
