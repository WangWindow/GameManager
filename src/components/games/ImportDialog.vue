<script setup lang="ts">
import { ref, watch } from 'vue'
import { Icon } from '@iconify/vue'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { SUPPORTED_ENGINES, getEngineDisplayName } from '@/constants/engines'
import type { EngineType } from '@/types/engine'

interface Props {
  open: boolean
  loading?: boolean
}

interface Emits {
  (e: 'update:open', value: boolean): void
  (e: 'submit', payload: { path: string; engineType: string }): void
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
})

const emit = defineEmits<Emits>()

const path = ref('')
const engineType = ref<string>(SUPPORTED_ENGINES[0])

watch(
  () => props.open,
  (val) => {
    if (!val) {
      path.value = ''
      engineType.value = SUPPORTED_ENGINES[0]
    }
  }
)

async function pickDirectory() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const res = await open({ directory: true, multiple: false, title: '选择游戏目录' })
    if (!res) return
    path.value = Array.isArray(res) ? res[0] ?? '' : res
  } catch (e) {
    console.error('选择目录失败:', e)
  }
}

function submit() {
  if (!path.value) return
  emit('submit', { path: path.value, engineType: engineType.value })
}
</script>

<template>
  <Dialog :open="open" @update:open="(v) => emit('update:open', v)">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>导入游戏</DialogTitle>
        <DialogDescription>选择游戏目录并指定引擎类型</DialogDescription>
      </DialogHeader>

      <div class="space-y-4">
        <div class="space-y-2">
          <label class="text-sm font-medium">游戏目录</label>
          <div class="flex gap-2">
            <Input v-model="path" placeholder="选择游戏目录" />
            <Button variant="secondary" class="px-3" @click="pickDirectory">
              <Icon icon="ri:folder-line" class="h-4 w-4" />
            </Button>
          </div>
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
      </div>

      <DialogFooter>
        <Button variant="ghost" @click="emit('update:open', false)">取消</Button>
        <Button :disabled="!path || loading" class="gap-2" @click="submit">
          <Icon v-if="loading" icon="ri:loader-4-line" class="h-4 w-4 animate-spin" />
          导入
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
