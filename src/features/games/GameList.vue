<script setup lang="ts">
import GameCard from "./GameCard.vue";

type EngineType = "rmmz" | "rmmv" | "unknown";

type GameEntry = {
  id: string;
  title: string;
  engineType: EngineType;
  version?: string;
  path: string;
  pathValid: boolean;
  coverPath?: string | null;
};

defineProps<{
  games: GameEntry[];
  selectedGameId: string;
  engineLabel: (t: EngineType) => string;
}>();

const emit = defineEmits<{
  (e: "select", id: string): void;
  (e: "start", id: string): void;
  (e: "settings", id: string): void;
}>();
</script>

<template>
  <section class="min-w-0 flex-1">
    <div class="mb-3 flex items-center justify-between">
      <div class="text-sm font-semibold">游戏库</div>
      <div class="text-xs text-zinc-500 dark:text-zinc-400">{{ games.length }} 项</div>
    </div>

    <div class="grid grid-cols-1 gap-3">
      <GameCard v-for="g in games" :key="g.id" :title="g.title"
        :subtitle="`${g.version ?? ''} ${engineLabel(g.engineType)}`" :selected="selectedGameId === g.id"
        :path-valid="g.pathValid" :cover-path="g.coverPath ?? null" @select="emit('select', g.id)"
        @start="emit('start', g.id)" @settings="emit('settings', g.id)" />
    </div>
  </section>
</template>
