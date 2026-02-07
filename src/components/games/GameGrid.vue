<script setup lang="ts">
import { computed } from 'vue'
import { Card, CardContent } from '@/components/ui/card'
import GameCard from './GameCard.vue'
import type { GameDto } from '@/types'

interface Props {
  games: GameDto[]
  loading?: boolean
  viewMode: 'grid' | 'list'
}

interface Emits {
  (e: 'launch', id: string): void
  (e: 'edit', id: string): void
  (e: 'delete', id: string): void
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
})

const emit = defineEmits<Emits>()

const isEmpty = computed(() => !props.loading && props.games.length === 0)
const isGrid = computed(() => props.viewMode === 'grid')
</script>

<template>
  <div>
    <!-- 加载状态 -->
    <div v-if="loading" class="space-y-3">
      <Card v-for="i in 8" :key="i" class="flex items-center gap-3 p-3">
        <div class="h-14 w-14 animate-pulse rounded-md bg-muted" />
        <div class="flex-1 space-y-2">
          <div class="h-3 w-1/3 animate-pulse rounded bg-muted" />
          <div class="h-2.5 w-1/4 animate-pulse rounded bg-muted" />
        </div>
        <div class="flex gap-1">
          <div class="h-8 w-8 animate-pulse rounded bg-muted" />
          <div class="h-8 w-8 animate-pulse rounded bg-muted" />
        </div>
      </Card>
    </div>

    <!-- 游戏网格 -->
    <div v-else-if="!isEmpty" :class="isGrid ? 'grid gap-3 sm:grid-cols-2 xl:grid-cols-3' : 'space-y-3'">
      <GameCard v-for="game in games" :key="game.id" :game="game" @launch="emit('launch', game.id)"
        @edit="emit('edit', game.id)" @delete="emit('delete', game.id)" />
    </div>

    <!-- 空状态 -->
    <Card v-else class="mx-auto w-full max-w-lg">
      <CardContent class="flex flex-col items-center justify-center text-center">
        <div class="mb-4 text-6xl">🎮</div>
        <h3 class="mb-2 text-lg font-semibold">暂无游戏</h3>
        <p class="text-sm text-muted-foreground">
          点击右上角的按钮导入或扫描游戏
        </p>
      </CardContent>
    </Card>
  </div>
</template>
