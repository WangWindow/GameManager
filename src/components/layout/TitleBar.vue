<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useWindowControls } from '@/hooks/useWindowControls'

const { isTauri, isMaximized, minimize, toggleMaximize, close } = useWindowControls()

const emit = defineEmits<{
  (e: 'manage'): void
  (e: 'import'): void
  (e: 'scan'): void
  (e: 'settings'): void
}>()

const props = defineProps<{ search?: string }>()
</script>

<template>
  <header data-tauri-drag-region
    class="fixed left-0 right-0 top-0 z-50 flex h-10 select-none items-center justify-between border-b bg-background/80 px-3 backdrop-blur supports-backdrop-filter:bg-background/60">
    <div data-tauri-drag-region class="flex min-w-0 items-center gap-2">
      <div class="flex h-7 w-7 items-center justify-center rounded-md bg-primary text-primary-foreground">
        <Icon icon="ri:gamepad-fill" class="h-5 w-5" />
      </div>
      <span data-tauri-drag-region class="text-sm font-semibold tracking-tight">GameManager</span>
    </div>

    <div data-tauri-drag-region="false" class="flex items-center gap-2">
      <Button variant="ghost" size="sm" class="h-8 gap-2 px-2" title="导入" @click="emit('import')">
        <Icon icon="ri:add-line" class="h-4 w-4" />
        <span class="text-xs">导入</span>
      </Button>
      <Button variant="ghost" size="sm" class="h-8 gap-2 px-2" title="扫描" @click="emit('scan')">
        <Icon icon="ri:scan-line" class="h-4 w-4" />
        <span class="text-xs">扫描</span>
      </Button>

      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <Button variant="ghost" size="icon" class="h-8 w-8" title="更多">
            <Icon icon="ri:more-2-line" class="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem @click="emit('manage')">管理中心</DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem @click="emit('settings')">设置</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <Button v-if="isTauri" variant="ghost" size="icon" class="h-8 w-9" title="最小化" @click="minimize">
        <Icon icon="ri:subtract-line" class="h-4 w-4" />
      </Button>
      <Button v-if="isTauri" variant="ghost" size="icon" class="h-8 w-9" title="最大化/还原" @click="toggleMaximize">
        <Icon :icon="isMaximized ? 'ri:checkbox-multiple-blank-line' : 'ri:checkbox-blank-line'" class="h-4 w-4" />
      </Button>
      <Button v-if="isTauri" variant="destructive" size="icon" class="h-8 w-9" title="关闭" @click="close">
        <Icon icon="ri:close-line" class="h-4 w-4" />
      </Button>
    </div>
  </header>
</template>
