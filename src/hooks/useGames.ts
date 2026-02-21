import { useState, useCallback } from "react";
import { getGames, addGame, updateGame, deleteGame, launchGame } from "@/lib/api";
import type { GameDto, AddGameInput, UpdateGameInput } from "@/types";

export function useGames() {
  const [games, setGames] = useState<GameDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchGames = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await getGames();
      setGames(data);
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
      setGames((prev) => [...prev, newGame]);
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
        setGames((prev) => prev.map((g) => (g.id === id ? updatedGame : g)));
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
      setGames((prev) => prev.filter((g) => g.id !== id));
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
      await fetchGames();
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
