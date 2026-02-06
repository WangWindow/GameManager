<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { SUPPORTED_ENGINES, getEngineDisplayName } from '@/constants/engines'
import { getGameProfileDir, getGameSettings, getIntegrationStatus, openPath } from '@/lib/api'
import type { GameConfig, GameDto } from '@/types'
import type { EngineType } from '@/types/engine'

interface Props {
  open: boolean
  game: GameDto | null
}

interface Emits {
  (e: 'update:open', value: boolean): void
  (e: 'save', payload: {
    id: string
    title: string
    engineType: string
    path: string
    runtimeVersion?: string
    settings: GameConfig
  }): void
  (e: 'refreshCover', id: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const title = ref('')
const engineType = ref<string>('')
const path = ref('')
const runtimeVersion = ref('')
const entryPath = ref('')
const argsText = ref('')
const sandboxHome = ref(true)
const coverFile = ref('')
const settingsLoading = ref(false)
const settingsLoaded = ref(false)
const bottlesLoading = ref(false)
const bottlesAvailable = ref(false)
const bottlesInstalled = ref(false)
const bottlesEnabled = ref(false)
const bottlesList = ref<string[]>([])
const defaultBottle = ref('')
const bottleName = ref('')

const isMvMz = computed(() => ['rpgmakermv', 'rpgmakermz'].includes(engineType.value))
const requiresEntryPath = computed(() => engineType.value === 'other')

const canSave = computed(() => {
  const basicValid = !!props.game && title.value.trim().length > 0 && path.value.trim().length > 0
  const entryValid = !requiresEntryPath.value || entryPath.value.trim().length > 0
  return basicValid && entryValid && !settingsLoading.value
})

watch(
  () => props.game,
  (game) => {
    if (!game) return
    title.value = game.title
    engineType.value = game.engineType
    path.value = game.path
    runtimeVersion.value = game.runtimeVersion ?? ''
    entryPath.value = '' // 默认不回退为目录，等待设置加载
    argsText.value = ''
    sandboxHome.value = true
    coverFile.value = ''
    bottleName.value = ''
    settingsLoaded.value = false
  },
  { immediate: true }
)

watch(
  () => props.open,
  async (open) => {
    if (!open || !props.game || settingsLoaded.value) return
    settingsLoading.value = true
    try {
      const config = await getGameSettings(props.game.id)
      engineType.value = config.engineType || props.game.engineType
      entryPath.value = config.entryPath ? toAbsoluteEntryPath(config.entryPath) : ''
      runtimeVersion.value = config.runtimeVersion ?? props.game.runtimeVersion ?? ''
      argsText.value = (config.args ?? []).join(' ')
      sandboxHome.value = config.sandboxHome ?? true
      coverFile.value = config.coverFile ?? ''
      bottleName.value = config.bottleName ?? ''
      await refreshBottlesStatus()
      if (!bottleName.value && defaultBottle.value) {
        bottleName.value = defaultBottle.value
      }
      settingsLoaded.value = true
    } catch (e) {
      console.error('加载游戏设置失败:', e)
    } finally {
      settingsLoading.value = false
    }
  }
)


watch(
  () => engineType.value,
  (val) => {
    if (val === 'other' && props.open) {
      refreshBottlesStatus()
    }
  }
)

function handleSave() {
  if (!props.game) return
  const args = argsText.value
    .split(/\s+/)
    .map((s) => s.trim())
    .filter(Boolean)

  const resolvedEntryPath = toAbsoluteEntryPath(entryPath.value.trim() || path.value.trim())

  const usingBottles =
    engineType.value === 'other' && bottlesAvailable.value && bottlesEnabled.value && bottlesInstalled.value

  const settings: GameConfig = {
    engineType: engineType.value,
    entryPath: resolvedEntryPath,
    runtimeVersion: runtimeVersion.value.trim() || undefined,
    args,
    sandboxHome: sandboxHome.value,
    useBottles: usingBottles,
    bottleName: usingBottles ? (bottleName.value.trim() || undefined) : undefined,
    coverFile: coverFile.value.trim() || undefined,
  }

  emit('save', {
    id: props.game.id,
    title: title.value.trim(),
    engineType: engineType.value,
    path: path.value.trim(),
    runtimeVersion: runtimeVersion.value.trim() || undefined,
    settings,
  })
}

async function refreshBottlesStatus() {
  if (!props.game) return
  if (engineType.value !== 'other') return

  bottlesLoading.value = true
  try {
    const status = await getIntegrationStatus('bottles')
    const options = status.options ?? {}
    bottlesAvailable.value = status.available
    if (!bottlesAvailable.value) {
      bottlesInstalled.value = false
      bottlesEnabled.value = false
      bottlesList.value = []
      defaultBottle.value = ''
      return
    }
    bottlesInstalled.value = options.installed ?? status.available
    bottlesEnabled.value = status.enabled
    bottlesList.value = options.bottles ?? []
    defaultBottle.value = options.defaultBottle ?? ''
  } catch (e) {
    bottlesAvailable.value = false
    bottlesInstalled.value = false
    bottlesEnabled.value = false
    bottlesList.value = []
    defaultBottle.value = ''
    console.error('获取 Bottles 状态失败:', e)
  } finally {
    bottlesLoading.value = false
  }
}

function isAbsolutePath(value: string): boolean {
  return value.startsWith('/') || /^[A-Za-z]:[\\/]/.test(value)
}

function joinPath(base: string, sub: string): string {
  if (base.includes('\\')) {
    return `${base.replace(/\\+$/, '')}\\${sub.replace(/^\\+/, '')}`
  }
  return `${base.replace(/\/+$/, '')}/${sub.replace(/^\/+/, '')}`
}

function toAbsoluteEntryPath(value: string): string {
  const trimmed = value.trim()
  if (!trimmed || !props.game) return trimmed
  if (isAbsolutePath(trimmed)) return trimmed
  return joinPath(props.game.path, trimmed)
}

async function pickEntryFile() {
  if (!props.game) return
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const res = await open({ title: '选择可执行文件', multiple: false })
    if (!res) return
    const selected = Array.isArray(res) ? res[0] ?? '' : res
    if (!selected) return
    entryPath.value = selected
  } catch (e) {
    console.error('选择可执行文件失败:', e)
  }
}

async function openGameDir() {
  if (!props.game) return
  try {
    await openPath(props.game.path)
  } catch (e) {
    console.error('打开游戏目录失败:', e)
  }
}

async function openProfileDir() {
  if (!props.game) return
  try {
    const profileDir = await getGameProfileDir(props.game.id)
    await openPath(profileDir)
  } catch (e) {
    console.error('打开Profile目录失败:', e)
  }
}

async function pickCoverFile() {
  if (!props.game) return
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const res = await open({ title: '选择图标/封面', multiple: false })
    if (!res) return
    const selected = Array.isArray(res) ? res[0] ?? '' : res
    if (!selected) return
    coverFile.value = selected
  } catch (e) {
    console.error('选择图标失败:', e)
  }
}

function handleRefreshCover() {
  if (!props.game) return
  emit('refreshCover', props.game.id)
}
</script>

<template>
  <Dialog :open="open" @update:open="(v) => emit('update:open', v)">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>游戏设置</DialogTitle>
        <DialogDescription>编辑游戏信息与运行参数</DialogDescription>
      </DialogHeader>

      <ScrollArea class="max-h-[60vh] pr-3">
        <div class="space-y-4">
          <div class="space-y-2">
            <label class="text-sm font-medium">游戏名称</label>
            <Input v-model="title" placeholder="游戏名称" />
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium">引擎类型</label>
            <Select v-model="engineType">
              <SelectTrigger>
                <SelectValue placeholder="选择引擎类型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="engine in SUPPORTED_ENGINES" :key="engine" :value="engine">
                  {{ getEngineDisplayName(engine as EngineType) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium">游戏路径</label>
            <Input v-model="path" placeholder="游戏路径" />
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium">打开目录</label>
            <div class="flex flex-wrap gap-2">
              <Button variant="secondary" size="sm" @click="openGameDir">打开游戏目录</Button>
              <Button variant="secondary" size="sm" @click="openProfileDir">打开 Profile 目录</Button>
            </div>
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium">图标/封面（可选）</label>
            <div class="flex items-center gap-2">
              <Input v-model="coverFile" placeholder="如：www/icon/icon.png" />
              <Button variant="secondary" size="icon" class="h-9 w-9" @click="pickCoverFile">
                …
              </Button>
            </div>
            <div class="flex flex-wrap gap-2">
              <Button variant="secondary" size="sm" :disabled="settingsLoading || !props.game"
                @click="handleRefreshCover">
                从可执行文件提取
              </Button>
            </div>
            <div class="text-xs text-muted-foreground">可填相对游戏目录路径</div>
          </div>

          <div v-if="isMvMz" class="space-y-2">
            <label class="text-sm font-medium">NW.js 运行时版本（可选）</label>
            <Input v-model="runtimeVersion" placeholder="如：0.84.0" />
          </div>

          <div class="rounded-md border p-3">
            <div class="mb-2 text-sm font-medium">运行设置</div>
            <div v-if="settingsLoading" class="text-xs text-muted-foreground">加载设置中…</div>
            <div v-else class="space-y-3">
              <div class="space-y-2">
                <label class="text-sm font-medium">入口文件/目录</label>
                <div class="flex gap-2">
                  <Input v-model="entryPath" placeholder="如：Game.exe / launcher.sh" />
                  <Button variant="secondary" size="sm" @click="pickEntryFile">选择</Button>
                </div>
                <div v-if="engineType === 'other' && bottlesEnabled && bottlesAvailable"
                  class="text-xs text-muted-foreground">
                  Bottles 启用时可填写程序名称（如：Bandizip）
                </div>
              </div>

              <div v-if="engineType === 'other' && bottlesAvailable" class="space-y-2">
                <div class="text-sm font-medium">Bottles Bottle</div>

                <div v-if="!bottlesInstalled" class="rounded-md border px-3 py-2 text-xs text-muted-foreground">
                  <div class="mb-1">该功能在你的系统上不可用。</div>
                  <div>请在管理面板启用 Bottles 并安装运行环境。</div>
                </div>

                <div v-else class="space-y-2">
                  <Select v-model="bottleName"
                    :disabled="bottlesLoading || bottlesList.length === 0 || !bottlesEnabled">
                    <SelectTrigger>
                      <SelectValue placeholder="选择 Bottle" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem v-for="name in bottlesList" :key="name" :value="name">
                        {{ name }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                  <div class="text-xs text-muted-foreground">未选择时会使用管理面板设置的默认 Bottle</div>
                </div>
              </div>

              <div class="space-y-2">
                <label class="text-sm font-medium">启动参数（空格分隔）</label>
                <Input v-model="argsText" placeholder="--debug --foo=bar" />
              </div>

              <div class="flex items-center justify-between rounded-md border px-3 py-2">
                <div>
                  <div class="text-sm font-medium">沙盒主目录</div>
                  <div class="text-xs text-muted-foreground">隔离游戏的用户数据</div>
                </div>
                <Switch :model-value="sandboxHome" @update:model-value="(v) => (sandboxHome = Boolean(v))" />
              </div>
            </div>
          </div>
        </div>
      </ScrollArea>

      <DialogFooter>
        <Button variant="ghost" @click="emit('update:open', false)">取消</Button>
        <Button :disabled="!canSave" @click="handleSave">保存</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
