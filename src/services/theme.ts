import { ref, watch } from 'vue'

export type ThemeMode = 'system' | 'light' | 'dark'

const STORAGE_KEY = 'kiroku-theme'

export const themeMode = ref<ThemeMode>(readMode())

const media =
  typeof window.matchMedia === 'function'
    ? window.matchMedia('(prefers-color-scheme: dark)')
    : { matches: false, addEventListener: () => {} }

function readMode(): ThemeMode {
  const saved = localStorage.getItem(STORAGE_KEY)
  return saved === 'light' || saved === 'dark' ? saved : 'system'
}

function resolved(): 'light' | 'dark' {
  if (themeMode.value === 'system') return media.matches ? 'dark' : 'light'
  return themeMode.value
}

function apply() {
  const value = resolved()
  document.documentElement.dataset.theme = value
  document.documentElement.style.colorScheme = value
}

export function initTheme() {
  apply()
  watch(themeMode, mode => {
    localStorage.setItem(STORAGE_KEY, mode)
    apply()
  })
  media.addEventListener('change', () => {
    if (themeMode.value === 'system') apply()
  })
}
