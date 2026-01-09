<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Icon } from "@iconify/vue";
import { convertFileSrc } from "@tauri-apps/api/core";

import Button from "../../components/ui/button/Button.vue";
import Card from "../../components/ui/card/Card.vue";
import { invokeResult } from "../../lib/tauri";

type EngineType = "rmmz" | "rmmv" | "unknown";

type GameEntry = {
  id: string;
  title: string;
  engineType: EngineType;
  path: string;
  pathValid: boolean;
  coverPath?: string | null;
};

const props = defineProps<{
  open: boolean;
  game: GameEntry | null;
  engineLabel: (t: EngineType) => string;
}>();

const emit = defineEmits<{
  (e: "update:open", v: boolean): void;
  (e: "deleted", gameId: string): void;
  (e: "titleUpdated", gameId: string): void;
  (e: "changed", gameId: string): void;
  (e: "openPath", path: string): void;
}>();

const titleDraft = ref("");
const containerInfo = ref<{ profileDir: string; userDataDir: string; settingsToml: string } | null>(null);
const launchArgsText = ref("");
const sandboxHome = ref(true);

const canSaveTitle = computed(() => titleDraft.value.trim().length > 0);

const coverSrc = computed(() => {
  const p = props.game?.coverPath ?? null;
  if (!p) return null;
  try {
    return convertFileSrc(p);
  } catch {
    return null;
  }
});

function close() {
  emit("update:open", false);
}

async function loadContainerInfo(gameId: string) {
  const res = await invokeResult<{ profileDir: string; userDataDir: string; settingsToml: string }>(
    "get_game_container_info",
    { gameId },
  );
  containerInfo.value = res.ok ? res.data : null;
}

async function loadLaunchConfig(gameId: string) {
  const res = await invokeResult<{ args: string[]; sandboxHome: boolean }>("get_game_launch_config", { gameId });
  if (!res.ok) {
    launchArgsText.value = "";
    sandboxHome.value = true;
    return;
  }

  sandboxHome.value = res.data.sandboxHome;
  launchArgsText.value = (res.data.args ?? []).join("\n");
}

watch(
  () => [props.open, props.game?.id] as const,
  async ([open, gameId]) => {
    if (!open || !gameId || !props.game) {
      containerInfo.value = null;
      launchArgsText.value = "";
      return;
    }
    titleDraft.value = props.game.title;
    await loadContainerInfo(gameId);
    await loadLaunchConfig(gameId);
  },
  { immediate: true },
);

async function saveTitle() {
  if (!props.game) return;
  const title = titleDraft.value.trim();
  if (!title) return;

  const res = await invokeResult<void>("update_game_title", { gameId: props.game.id, title });
  if (!res.ok) return;

  emit("titleUpdated", props.game.id);
}

async function deleteGame() {
  if (!props.game) return;
  const { confirm } = await import("@tauri-apps/plugin-dialog");
  const ok = await confirm(`确认删除“${props.game.title}”的条目？\n\n不会删除游戏文件。`, {
    title: "确认删除",
    kind: "warning",
  });
  if (!ok) return;

  const res = await invokeResult<void>("delete_game", { gameId: props.game.id });
  if (!res.ok) return;

  emit("deleted", props.game.id);
  close();
}

function openContainerDir() {
  if (!containerInfo.value) return;
  emit("openPath", containerInfo.value.profileDir);
}

async function saveLaunchConfig() {
  if (!props.game) return;
  const args = launchArgsText.value
    .split(/\r?\n/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const res = await invokeResult<void>("update_game_launch_config", {
    input: { gameId: props.game.id, args, sandboxHome: sandboxHome.value },
  });
  if (!res.ok) return;
}

async function pickCover() {
  if (!props.game) return;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const res = await open({
    multiple: false,
    directory: false,
    title: "选择封面图片",
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }],
  });
  if (!res) return;
  const path = Array.isArray(res) ? res[0] : res;
  if (!path) return;

  const saveRes = await invokeResult<void>("set_game_cover", {
    input: { gameId: props.game.id, sourcePath: path },
  });
  if (!saveRes.ok) return;
  emit("changed", props.game.id);
}

async function clearCover() {
  if (!props.game) return;
  const res = await invokeResult<void>("clear_game_cover", { gameId: props.game.id });
  if (!res.ok) return;
  emit("changed", props.game.id);
}
</script>

<template>
  <div v-if="open && game" class="fixed inset-0 z-50">
    <button class="absolute inset-0 bg-zinc-950/30 dark:bg-black/50" type="button" aria-label="关闭" @click="close" />

    <div class="absolute inset-0 grid place-items-center p-4">
      <Card class="w-full max-w-lg p-4">
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <div class="truncate text-sm font-semibold">游戏设置</div>
            <div class="mt-0.5 truncate text-xs text-zinc-500 dark:text-zinc-400">
              {{ engineLabel(game.engineType) }}
            </div>
          </div>
          <button
            class="inline-flex h-8 w-8 items-center justify-center rounded-md transition-colors hover:bg-zinc-100 dark:hover:bg-zinc-900"
            type="button" aria-label="关闭" @click="close">
            <Icon icon="ri:close-line" class="size-4" />
          </button>
        </div>

        <div class="mt-4 space-y-3">
          <div>
            <div class="flex items-center justify-between gap-3">
              <div class="text-xs font-medium text-zinc-500 dark:text-zinc-400">封面</div>
              <div class="flex items-center gap-2">
                <Button variant="secondary" size="sm" @click="pickCover">
                  <Icon icon="ri:image-add-line" class="size-4" />
                  更换封面
                </Button>
                <Button variant="secondary" size="sm" :disabled="!game.coverPath" @click="clearCover">
                  <Icon icon="ri:delete-back-2-line" class="size-4" />
                  清除
                </Button>
              </div>
            </div>
            <div class="mt-2 flex items-center gap-3">
              <div
                class="size-16 shrink-0 overflow-hidden rounded-lg bg-linear-to-br from-zinc-200 to-zinc-100 dark:from-zinc-800 dark:to-zinc-900">
                <img v-if="coverSrc" :src="coverSrc" alt="" class="h-full w-full object-cover" />
              </div>
              <div class="min-w-0 text-xs text-zinc-500 dark:text-zinc-400">
                <div class="truncate">{{ game.coverPath ?? "未设置（会从 icon/icons 自动选择）" }}</div>
              </div>
            </div>
          </div>

          <div>
            <div class="text-xs font-medium text-zinc-500 dark:text-zinc-400">显示名称</div>
            <div class="mt-1 flex gap-2">
              <input v-model="titleDraft"
                class="h-9 w-full rounded-md border border-zinc-200 bg-white px-3 text-sm dark:border-zinc-800 dark:bg-zinc-950"
                type="text" placeholder="游戏名称" />
              <Button :disabled="!canSaveTitle" variant="secondary" @click="saveTitle">
                <Icon icon="ri:save-3-line" class="size-4" />
                保存
              </Button>
            </div>
          </div>

          <div>
            <div class="text-xs font-medium text-zinc-500 dark:text-zinc-400">路径</div>
            <div class="mt-1 rounded-md border border-zinc-200 px-3 py-2 text-xs dark:border-zinc-800">
              <div class="truncate">{{ game.path }}</div>
            </div>
            <div v-if="!game.pathValid" class="mt-1 text-xs text-red-600">路径无效：请重新导入或修复路径。</div>
          </div>

          <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
            <Button variant="secondary" class="justify-start" @click="openContainerDir">
              <Icon icon="ri:folder-open-line" class="size-4" />
              打开容器目录
            </Button>
          </div>

          <div class="h-px bg-zinc-200 dark:bg-zinc-800" />

          <div>
            <div class="flex items-center justify-between gap-3">
              <div class="text-xs font-medium text-zinc-500 dark:text-zinc-400">启动参数（每行一个）</div>
              <Button variant="secondary" size="sm" @click="saveLaunchConfig">
                <Icon icon="ri:save-3-line" class="size-4" />
                保存参数
              </Button>
            </div>

            <textarea v-model="launchArgsText"
              class="mt-1 h-28 w-full resize-none rounded-md border border-zinc-200 bg-white px-3 py-2 text-xs dark:border-zinc-800 dark:bg-zinc-950"
              placeholder="例如：\n--disable-gpu\n--force-device-scale-factor=1" />

            <label class="mt-2 flex items-center gap-2 text-xs text-zinc-600 dark:text-zinc-300">
              <input v-model="sandboxHome" type="checkbox" class="size-4" />
              隔离 HOME/XDG（把 ~/.config 等重定向到容器目录）
            </label>
          </div>

          <div class="h-px bg-zinc-200 dark:bg-zinc-800" />

          <div class="flex items-center justify-between">
            <div class="text-xs text-zinc-500 dark:text-zinc-400">删除条目位于游戏设置中</div>
            <Button variant="destructive" @click.stop="deleteGame">
              <Icon icon="ri:delete-bin-6-line" class="size-4" />
              删除
            </Button>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
