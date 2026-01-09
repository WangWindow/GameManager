<script setup lang="ts">
import { cn } from "../../../lib/utils";

defineProps<{ class?: string }>();
</script>

<template>
  <div
    :class="cn('rounded-xl border border-zinc-200/70 bg-white/80 p-3 shadow-sm backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/50', $props.class)">
    <slot />
  </div>
</template>
