/**
 * 游戏管理 Composable
 */
import { ref, computed } from 'vue'
import { getGames, addGame, updateGame, deleteGame, launchGame } from '@/lib/api'
import type { GameDto, AddGameInput, UpdateGameInput } from '@/types'

export function useGames() {
  const games = ref<GameDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  /**
   * 获取游戏列表
   */
  async function fetchGames() {
    loading.value = true
    error.value = null
    try {
      games.value = await getGames()
    } catch (e) {
      error.value = e instanceof Error ? e.message : '获取游戏列表失败'
      console.error('获取游戏列表失败:', e)
    } finally {
      loading.value = false
    }
  }

  /**
   * 添加游戏
   */
  async function handleAddGame(input: AddGameInput): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      const newGame = await addGame(input)
      games.value.push(newGame)
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : '添加游戏失败'
      console.error('添加游戏失败:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新游戏
   */
  async function handleUpdateGame(id: string, input: UpdateGameInput): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      const updatedGame = await updateGame(id, input)
      const index = games.value.findIndex(g => g.id === id)
      if (index !== -1) {
        games.value[index] = updatedGame
      }
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : '更新游戏失败'
      console.error('更新游戏失败:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 删除游戏
   */
  async function handleDeleteGame(id: string): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      await deleteGame(id)
      games.value = games.value.filter(g => g.id !== id)
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : '删除游戏失败'
      console.error('删除游戏失败:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 启动游戏
   */
  async function handleLaunchGame(id: string): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      await launchGame(id)
      // 刷新游戏列表以更新最后游玩时间
      await fetchGames()
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : '启动游戏失败'
      console.error('启动游戏失败:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 根据ID获取游戏
   */
  function getGameById(id: string) {
    return computed(() => games.value.find(g => g.id === id))
  }

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
  }
}
