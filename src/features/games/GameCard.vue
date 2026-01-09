<script setup lang="ts">
import { Icon } from "@iconify/vue";

import Button from "../../components/ui/button/Button.vue";
import Card from "../../components/ui/card/Card.vue";
import { cn } from "../../lib/utils";

defineProps<{
  title: string;
  subtitle: string;
  selected: boolean;
  pathValid: boolean;
}>();

const emit = defineEmits<{
  (e: "select"): void;
  (e: "start"): void;
  (e: "settings"): void;
}>();
</script>

<template>
  <Card
    :class="cn('flex items-center gap-3 p-3 transition', selected && 'ring-2 ring-zinc-900/10 dark:ring-zinc-50/10')">
    <button type="button" class="flex min-w-0 flex-1 items-center gap-3 text-left" @click="emit('select')">
      <div
        class="size-14 shrink-0 overflow-hidden rounded-lg bg-linear-to-br from-zinc-200 to-zinc-100 dark:from-zinc-800 dark:to-zinc-900" />
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <div class="truncate text-sm font-semibold">{{ title }}</div>
          <span v-if="!pathValid"
            class="inline-flex items-center rounded-md bg-red-600 px-2 py-0.5 text-[11px] font-medium text-white">
            路径无效
          </span>
        </div>
        <div class="mt-0.5 truncate text-xs text-zinc-500 dark:text-zinc-400">{{ subtitle }}</div>
      </div>
    </button>

    <div class="ml-auto flex shrink-0 items-center gap-2">
      <Button size="sm" @click.stop="emit('start')">
        <Icon icon="ri:play-line" class="size-4" />
      </Button>
      <Button variant="secondary" size="sm" @click.stop="emit('settings')">
        <Icon icon="ri:settings-3-line" class="size-4" />
      </Button>
    </div>
  </Card>
</template>
