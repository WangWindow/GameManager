<script setup lang="ts">
import { Input } from '@/components/ui/input'
import { Icon } from '@iconify/vue'

interface Props {
  count: number
  search: string
}

interface Emits {
  (e: 'update:search', value: string): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
  <div class="mb-5 flex items-center justify-between">
    <h1 class="text-2xl font-semibold">游戏库</h1>

    <div class="mx-4 flex flex-1 items-center">
      <div class="relative w-full max-w-lg">
        <Icon icon="ri:search-line"
          class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input :model-value="search" placeholder="搜索游戏..." class="h-8 rounded-md border pl-10"
          @update:model-value="(v) => emit('update:search', String(v))" />
      </div>
    </div>

    <span class="text-xs text-muted-foreground">{{ count }} 项</span>
  </div>
</template>
