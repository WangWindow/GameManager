<script setup lang="ts">
import { Icon } from "@iconify/vue";

import Button from "../../components/ui/button/Button.vue";
import Card from "../../components/ui/card/Card.vue";

defineProps<{
  open: boolean;
  showStatusBar: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", v: boolean): void;
  (e: "update:showStatusBar", v: boolean): void;
  (e: "downloadNwjs"): void;
  (e: "cleanupContainers"): void;
}>();

function close() {
  emit("update:open", false);
}

function onBackdropClick() {
  close();
}
</script>

<template>
  <div v-if="open" class="fixed inset-0 z-50">
    <button class="absolute inset-0 bg-zinc-950/30 dark:bg-black/50" type="button" aria-label="关闭设置"
      @click="onBackdropClick" />

    <div class="absolute inset-0 grid place-items-center p-4">
      <Card class="w-full max-w-md p-4">
        <div class="flex items-center justify-between gap-3">
          <div class="text-sm font-semibold">设置</div>
          <button
            class="inline-flex h-8 w-8 items-center justify-center rounded-md transition-colors hover:bg-zinc-100 dark:hover:bg-zinc-900"
            type="button" aria-label="关闭" @click="close">
            <Icon icon="ri:close-line" class="size-4" />
          </button>
        </div>

        <div class="mt-4 space-y-3">
          <label
            class="flex cursor-pointer items-center justify-between gap-3 rounded-md border border-zinc-200 px-3 py-2 text-sm dark:border-zinc-800">
            <div class="min-w-0">
              <div class="font-medium">显示状态栏</div>
              <div class="text-xs text-zinc-500 dark:text-zinc-400">后台任务与错误信息展示</div>
            </div>
            <input :checked="showStatusBar" type="checkbox" class="size-4 shrink-0 accent-zinc-900 dark:accent-zinc-50"
              @change="emit('update:showStatusBar', ($event.target as HTMLInputElement).checked)" />
          </label>

          <div class="grid grid-cols-1 gap-2">
            <Button variant="secondary" class="w-full justify-start" @click="() => { close(); emit('downloadNwjs'); }">
              <Icon icon="ri:download-2-line" class="size-4" />
              下载 NW.js
            </Button>

            <Button variant="secondary" class="w-full justify-start"
              @click="() => { close(); emit('cleanupContainers'); }">
              <Icon icon="ri:delete-bin-6-line" class="size-4" />
              清除无用容器
            </Button>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
