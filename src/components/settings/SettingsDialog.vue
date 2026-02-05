<script setup lang="ts">
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'

interface Props {
  open: boolean
  themeMode: 'system' | 'light' | 'dark'
}

interface Emits {
  (e: 'update:open', value: boolean): void
  (e: 'update:themeMode', value: 'system' | 'light' | 'dark'): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
  <Dialog :open="open" @update:open="(v) => emit('update:open', v)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>设置</DialogTitle>
        <DialogDescription>基础偏好设置</DialogDescription>
      </DialogHeader>

      <div class="space-y-4">
        <div class="flex items-center justify-between rounded-md border px-3 py-2">
          <div>
            <div class="text-sm font-medium">主题</div>
            <div class="text-xs text-muted-foreground">跟随系统或手动选择</div>
          </div>
          <Select :model-value="themeMode"
            @update:model-value="(v) => emit('update:themeMode', v as 'system' | 'light' | 'dark')">
            <SelectTrigger class="w-28">
              <SelectValue placeholder="主题" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="system">系统</SelectItem>
              <SelectItem value="light">浅色</SelectItem>
              <SelectItem value="dark">深色</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <Separator />
        <div class="text-xs text-muted-foreground">更多设置将逐步补充</div>
      </div>

      <DialogFooter>
        <Button variant="ghost" @click="emit('update:open', false)">关闭</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
