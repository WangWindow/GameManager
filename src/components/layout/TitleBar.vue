<script setup lang="ts">
import { Icon } from "@iconify/vue";

import Button from "../../components/ui/button/Button.vue";
import { useWindowControls } from "../../features/window/useWindowControls";

defineProps<{
  title: string;
  subtitle: string;
}>();

const emit = defineEmits<{
  (e: "import"): void;
  (e: "scan"): void;
  (e: "settings"): void;
}>();

const { isTauri, isMaximized, minimize, toggleMaximize, close } = useWindowControls();
</script>

<template>
  <header class="border-b border-zinc-200/70 bg-white/70 backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/50">
    <div class="flex h-12 items-center gap-3 px-3">
      <div class="flex min-w-0 items-center gap-3" data-tauri-drag-region>
        <div
          class="grid size-9 place-items-center rounded-lg bg-zinc-900 text-zinc-50 dark:bg-zinc-50 dark:text-zinc-900">
          <Icon icon="ri:gamepad-line" class="size-5" />
        </div>
        <div class="min-w-0 leading-tight">
          <div class="truncate text-sm font-semibold">{{ title }}</div>
          <div class="truncate text-xs text-zinc-500 dark:text-zinc-400">{{ subtitle }}</div>
        </div>
      </div>

      <div class="flex-1 self-stretch" data-tauri-drag-region />

      <div class="flex items-center gap-2" data-tauri-drag-region="false">
        <Button variant="secondary" size="sm" @click="emit('import')">
          <Icon icon="ri:add-line" class="size-4" />
          导入
        </Button>
        <Button variant="secondary" size="sm" @click="emit('scan')">
          <Icon icon="ri:search-line" class="size-4" />
          扫描
        </Button>

        <button
          class="inline-flex h-9 w-9 items-center justify-center rounded-md transition-colors hover:bg-zinc-100 dark:hover:bg-zinc-900"
          type="button" aria-label="设置" @click="emit('settings')">
          <Icon icon="ri:more-2-line" class="size-5" />
        </button>

        <div v-if="isTauri" class="ml-1 flex items-center gap-1">
          <button
            class="inline-flex h-8 w-8 items-center justify-center rounded-md transition-colors hover:bg-zinc-100 dark:hover:bg-zinc-900"
            type="button" aria-label="最小化" @click="minimize">
            <Icon icon="ri:subtract-line" class="size-4" />
          </button>
          <button
            class="inline-flex h-8 w-8 items-center justify-center rounded-md transition-colors hover:bg-zinc-100 dark:hover:bg-zinc-900"
            type="button" aria-label="最大化/还原" @click="toggleMaximize">
            <Icon :icon="isMaximized ? 'ri:checkbox-multiple-blank-line' : 'ri:checkbox-blank-line'" class="size-4" />
          </button>
          <button
            class="inline-flex h-8 w-8 items-center justify-center rounded-md transition-colors hover:bg-zinc-100 dark:hover:bg-zinc-900"
            type="button" aria-label="关闭" @click="close">
            <Icon icon="ri:close-line" class="size-4" />
          </button>
        </div>
      </div>
    </div>
  </header>
</template>
