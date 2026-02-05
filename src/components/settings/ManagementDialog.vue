<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { getBottlesStatus, getEngines, setBottlesEnabled, setDefaultBottle } from '@/lib/api'
import { ScrollArea } from '@/components/ui/scroll-area'
import type { EngineDto } from '@/types'

interface Props {
  open: boolean
  showStatusBar: boolean
}

interface Emits {
  (e: 'update:open', value: boolean): void
  (e: 'update:showStatusBar', value: boolean): void
  (e: 'downloadNwjs'): void
  (e: 'cleanupContainers'): void
  (e: 'updateEngine', engine: EngineDto): void
  (e: 'removeEngine', engine: EngineDto): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const engines = ref<EngineDto[]>([])
const loading = ref(false)
const bottlesLoading = ref(false)
const bottlesInstalled = ref(false)
const bottlesEnabled = ref(false)
const bottlesList = ref<string[]>([])
const defaultBottle = ref('')
const bottlesInstallCommand = 'flatpak install flathub com.usebottles.bottles'

async function fetchEngines() {
  loading.value = true
  try {
    engines.value = await getEngines()
  } catch (e) {
    console.error('获取运行器失败:', e)
  } finally {
    loading.value = false
  }
}

async function fetchBottlesStatus() {
  bottlesLoading.value = true
  try {
    const status = await getBottlesStatus()
    bottlesInstalled.value = status.installed
    bottlesEnabled.value = status.enabled
    bottlesList.value = status.bottles
    defaultBottle.value = status.defaultBottle ?? ''
  } catch (e) {
    bottlesInstalled.value = false
    bottlesEnabled.value = false
    bottlesList.value = []
    defaultBottle.value = ''
    console.error('获取 Bottles 状态失败:', e)
  } finally {
    bottlesLoading.value = false
  }
}

async function updateBottlesEnabled(value: boolean) {
  bottlesEnabled.value = value
  try {
    await setBottlesEnabled({ enabled: value })
  } catch (e) {
    console.error('设置 Bottles 启用状态失败:', e)
  }
}

async function updateDefaultBottle(value: string) {
  defaultBottle.value = value
  try {
    await setDefaultBottle({ defaultBottle: value || undefined })
  } catch (e) {
    console.error('设置默认 Bottle 失败:', e)
  }
}

onMounted(() => {
  fetchEngines()
  fetchBottlesStatus()
  window.addEventListener('gm:refresh-engines', fetchEngines)
})

onUnmounted(() => {
  window.removeEventListener('gm:refresh-engines', fetchEngines)
})

watch(
  () => props.open,
  (open) => {
    if (open) {
      fetchEngines()
      fetchBottlesStatus()
    }
  }
)
</script>

<template>
  <Dialog :open="open" @update:open="(v) => emit('update:open', v)">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>管理</DialogTitle>
        <DialogDescription>应用运行与维护相关操作</DialogDescription>
      </DialogHeader>

      <ScrollArea class="max-h-[70vh] pr-3">
        <div class="space-y-4">
          <div class="flex items-center justify-between rounded-md border px-3 py-2">
            <div>
              <div class="text-sm font-medium">显示状态栏</div>
              <div class="text-xs text-muted-foreground">后台任务与进度信息展示</div>
            </div>
            <Switch :model-value="showStatusBar"
              @update:model-value="(v) => emit('update:showStatusBar', Boolean(v))" />
          </div>

          <Separator />

          <div class="space-y-2">
            <Button variant="secondary" class="w-full justify-start" @click="emit('downloadNwjs')">
              下载 NW.js
            </Button>
            <Button variant="secondary" class="w-full justify-start" @click="emit('cleanupContainers')">
              清理无用容器
            </Button>
          </div>

          <Separator />

          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <div class="text-sm font-medium">已安装运行器</div>
              <Button variant="ghost" size="sm" :disabled="loading" @click="fetchEngines">
                刷新
              </Button>
            </div>
            <div v-if="engines.length === 0" class="text-xs text-muted-foreground">
              暂无已安装运行器
            </div>
            <div v-else class="space-y-2">
              <div v-for="engine in engines" :key="engine.id"
                class="flex items-center justify-between rounded-md border px-3 py-2 text-sm">
                <div class="min-w-0">
                  <div class="truncate font-medium">{{ engine.name }}</div>
                  <div class="text-xs text-muted-foreground">
                    {{ engine.engineType }} · {{ engine.version }}
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <Button variant="ghost" size="sm" @click="emit('updateEngine', engine)">更新</Button>
                  <Button variant="destructive" size="sm" @click="emit('removeEngine', engine)">卸载</Button>
                </div>
              </div>
            </div>
            <div class="text-xs text-muted-foreground">
              RPG Maker MV/MZ 使用 NW.js 运行时
            </div>
          </div>

          <Separator />

          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <div class="text-sm font-medium">Bottles</div>
              <Button variant="ghost" size="sm" :disabled="bottlesLoading" @click="fetchBottlesStatus">
                刷新
              </Button>
            </div>

            <div class="flex items-center justify-between rounded-md border px-3 py-2">
              <div>
                <div class="text-sm font-medium">启用 Bottles</div>
                <div class="text-xs text-muted-foreground">用于 Windows EXE（Linux）</div>
              </div>
              <Switch :model-value="bottlesEnabled" :disabled="bottlesLoading || !bottlesInstalled"
                @update:model-value="(v) => updateBottlesEnabled(Boolean(v))" />
            </div>

            <div v-if="!bottlesInstalled" class="rounded-md border px-3 py-2 text-xs text-muted-foreground">
              <div class="mb-1">该功能在你的系统上不可用。</div>
              <div>To add it, please run:</div>
              <div class="mt-2 rounded bg-muted px-2 py-1 font-mono">{{ bottlesInstallCommand }}</div>
            </div>

            <div v-else class="space-y-2">
              <div class="text-xs text-muted-foreground">选择默认 Bottle，用于 Windows EXE</div>
              <Select :model-value="defaultBottle"
                :disabled="bottlesLoading || bottlesList.length === 0 || !bottlesEnabled"
                @update:model-value="(v) => updateDefaultBottle(String(v))">
                <SelectTrigger>
                  <SelectValue placeholder="选择 Bottle" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="name in bottlesList" :key="name" :value="name">
                    {{ name }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </ScrollArea>

      <DialogFooter>
        <Button variant="ghost" @click="emit('update:open', false)">关闭</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
