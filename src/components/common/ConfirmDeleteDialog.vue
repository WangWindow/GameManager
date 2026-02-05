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

interface Props {
  open: boolean
  title?: string
}

interface Emits {
  (e: 'update:open', value: boolean): void
  (e: 'confirm'): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
  <Dialog :open="open" @update:open="(v) => emit('update:open', v)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>确认删除</DialogTitle>
        <DialogDescription>
          确定要删除游戏{{ title ? `「${title}」` : '' }}吗？
        </DialogDescription>
      </DialogHeader>
      <DialogFooter>
        <Button variant="ghost" @click="emit('update:open', false)">取消</Button>
        <Button variant="destructive" @click="emit('confirm')">删除</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
