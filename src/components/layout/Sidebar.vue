<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const menuItems = [
  {
    id: 'library',
    label: '游戏库',
    icon: 'ri:gamepad-line',
    path: '/',
  },
  {
    id: 'settings',
    label: '设置',
    icon: 'ri:settings-3-line',
    path: '/settings',
  },
]

function isActive(path: string) {
  return route.path === path
}

function navigate(path: string) {
  router.push(path)
}
</script>

<template>
  <aside class="flex w-60 flex-col border-r bg-card">
    <!-- Logo -->
    <div class="flex h-16 items-center gap-3 px-6">
      <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary text-primary-foreground">
        <Icon icon="ri:gamepad-fill" class="h-6 w-6" />
      </div>
      <h1 class="text-lg font-semibold">GameManager</h1>
    </div>

    <Separator />

    <!-- 菜单 -->
    <nav class="flex-1 p-4">
      <ul class="space-y-1">
        <li v-for="item in menuItems" :key="item.id">
          <Button variant="ghost" class="w-full justify-start gap-3"
            :class="isActive(item.path) ? 'bg-muted text-foreground' : 'text-muted-foreground'"
            @click="navigate(item.path)">
            <Icon :icon="item.icon" class="h-5 w-5" />
            {{ item.label }}
          </Button>
        </li>
      </ul>
    </nav>

    <!-- 底部信息 -->
    <Separator />
    <div class="p-4">
      <p class="text-xs text-muted-foreground">
        GameManager v1.0.0
      </p>
    </div>
  </aside>
</template>
