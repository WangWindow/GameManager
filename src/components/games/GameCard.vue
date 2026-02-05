<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { getEngineDisplayName, getEngineIcon } from '@/constants/engines'
import { formatRelativeTime } from '@/utils/format'
import type { GameDto } from '@/types'

interface Props {
  game: GameDto
}

interface Emits {
  (e: 'launch'): void
  (e: 'edit'): void
  (e: 'delete'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const coverSrc = computed(() => {
  if (!props.game.coverPath) return ''
  try {
    return convertFileSrc(props.game.coverPath)
  } catch {
    return `asset://localhost/${props.game.coverPath}`
  }
})
</script>

<template>
  <div
    class="group relative flex items-center gap-3 rounded-xl border bg-card px-4 py-2.5 text-card-foreground transition-all hover:bg-muted/40 hover:shadow-md">
    <!-- 封面图 -->
    <div class="h-12 w-12 shrink-0 overflow-hidden rounded-md bg-muted">
      <img v-if="game.coverPath" :src="coverSrc" :alt="game.title" class="h-full w-full object-cover" />
      <div v-else class="flex h-full w-full items-center justify-center bg-linear-to-br from-muted to-muted/50">
        <Icon :icon="getEngineIcon(game.engineType)" class="h-6 w-6 text-muted-foreground/40" />
      </div>
    </div>

    <!-- 游戏信息 -->
    <div class="flex min-w-0 flex-1 flex-col gap-1">
      <div class="flex items-center gap-2">
        <h3 class="truncate text-sm font-semibold" :title="game.title">
          {{ game.title }}
        </h3>
        <Badge v-if="!game.pathValid" variant="destructive" class="h-4 px-2 text-[10px]">
          路径无效
        </Badge>
      </div>
      <div class="flex items-center gap-2 text-xs text-muted-foreground">
        <Icon :icon="getEngineIcon(game.engineType)" class="h-3.5 w-3.5" />
        <span class="truncate">{{ getEngineDisplayName(game.engineType) }}</span>
        <span v-if="game.lastPlayedAt" class="truncate">· {{ formatRelativeTime(game.lastPlayedAt) }}</span>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="flex shrink-0 items-center gap-2">
      <Button size="icon" class="h-7 w-7" title="启动游戏" @click="emit('launch')">
        <Icon icon="ri:play-fill" class="h-3.5 w-3.5" />
      </Button>
      <Button variant="secondary" size="icon" class="h-7 w-7" title="编辑" @click="emit('edit')">
        <Icon icon="ri:settings-3-line" class="h-3.5 w-3.5" />
      </Button>
      <Button variant="ghost" size="icon" class="h-7 w-7 text-muted-foreground hover:text-destructive" title="删除"
        @click="emit('delete')">
        <Icon icon="ri:delete-bin-line" class="h-3.5 w-3.5" />
      </Button>
    </div>
  </div>
</template>
