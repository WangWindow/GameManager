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

interface Props {
  open: boolean
  loading?: boolean
}

interface Emits {
  (e: 'update:open', value: boolean): void
  (e: 'submit', payload: { root: string; maxDepth: number }): void
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
})

const emit = defineEmits<Emits>()

const root = ref('')
const maxDepth = ref(3)

watch(
  () => props.open,
  (val) => {
    if (!val) {
      root.value = ''
      maxDepth.value = 3
    }
  }
)

async function pickDirectory() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const res = await open({ directory: true, multiple: false, title: '选择扫描根目录' })
    if (!res) return
    root.value = Array.isArray(res) ? res[0] ?? '' : res
  } catch (e) {
    console.error('选择目录失败:', e)
  }
}

function submit() {
  if (!root.value) return
  emit('submit', { root: root.value, maxDepth: Math.max(1, Number(maxDepth.value) || 1) })
}
</script>

<template>
  <Dialog :open="open" @update:open="(v) => emit('update:open', v)">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>扫描游戏</DialogTitle>
        <DialogDescription>选择根目录并设置最大扫描深度</DialogDescription>
      </DialogHeader>

      <div class="space-y-4">
        <div class="space-y-2">
          <label class="text-sm font-medium">扫描根目录</label>
          <div class="flex gap-2">
            <Input v-model="root" placeholder="选择扫描根目录" />
            <Button variant="secondary" class="px-3" @click="pickDirectory">
              <Icon icon="ri:folder-line" class="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div class="space-y-2">
          <label class="text-sm font-medium">最大扫描深度</label>
          <Input v-model.number="maxDepth" type="number" min="1" max="10" />
        </div>
      </div>

      <DialogFooter>
        <Button variant="ghost" @click="emit('update:open', false)">取消</Button>
        <Button :disabled="!root || loading" class="gap-2" @click="submit">
          <Icon v-if="loading" icon="ri:loader-4-line" class="h-4 w-4 animate-spin" />
          开始扫描
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
