import { onMounted, onUnmounted, ref, watch } from 'vue'

export type ThemeMode = 'system' | 'light' | 'dark'

export function useThemeMode() {
  const themeMode = ref<ThemeMode>('system')
  const systemDark = ref(false)

  function applyTheme(mode: ThemeMode) {
    const root = document.documentElement
    const isDark = mode === 'dark' || (mode === 'system' && systemDark.value)
    if (isDark) {
      root.classList.add('dark')
    } else {
      root.classList.remove('dark')
    }
  }

  watch(
    themeMode,
    (val) => {
      localStorage.setItem('gm_theme', val)
      applyTheme(val)
    },
    { immediate: true }
  )

  onMounted(() => {
    const theme = localStorage.getItem('gm_theme') as ThemeMode | null
    if (theme) themeMode.value = theme

    try {
      const media = window.matchMedia('(prefers-color-scheme: dark)')
      systemDark.value = media.matches
      applyTheme(themeMode.value)
      const handler = (e: MediaQueryListEvent) => {
        systemDark.value = e.matches
        if (themeMode.value === 'system') applyTheme('system')
      }
      media.addEventListener('change', handler)
      onUnmounted(() => media.removeEventListener('change', handler))
    } catch {
      // ignore
    }
  })

  return { themeMode }
}
