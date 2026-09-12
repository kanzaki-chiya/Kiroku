<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { BarChart3, Library, Monitor, Moon, Settings, Sun } from 'lucide-vue-next'
import { isTauri } from '../runtime'
import { themeMode, type ThemeMode } from '../services/theme'
import { useLibraryStore } from '../stores/library'
import { useLibraryViewStore } from '../stores/libraryView'
import { tierBadgeStyle, tierDescription } from '../utils/format'

const desktop = isTauri()
const store = useLibraryStore()
const viewState = useLibraryViewStore()
const route = useRoute()

const themeOptions: { value: ThemeMode; label: string; icon: typeof Monitor }[] = [
  { value: 'system', label: '跟随系统', icon: Monitor },
  { value: 'light', label: '浅色', icon: Sun },
  { value: 'dark', label: '深色', icon: Moon }
]

const tierCounts = computed(() => {
  const counts = new Map<string, number>()
  for (const tier of store.tiers) counts.set(tier.name, 0)
  for (const entry of store.entries) {
    if (entry.personal.tier) {
      counts.set(entry.personal.tier, (counts.get(entry.personal.tier) ?? 0) + 1)
    }
  }
  return counts
})
</script>

<template>
  <aside class="sidebar">
    <div v-if="desktop" class="drag-pad" data-tauri-drag-region>
      <svg class="pad-mark" viewBox="0 0 30 36" fill="none" aria-hidden="true">
        <path d="M7 5h17v25H7z" stroke="currentColor" stroke-width="1.4" />
        <path d="M3 9v25h17M11 5v25M15 11h5M15 15h5" stroke="currentColor" stroke-width="1.4" />
        <path d="M19 3v6l2-1.4L23 9V3z" fill="currentColor" />
      </svg>
      <span class="pad-name">Kiroku</span>
    </div>
    <RouterLink v-else to="/library" class="brand">
      <svg class="brand-mark" viewBox="0 0 30 36" fill="none" aria-hidden="true">
        <path d="M7 5h17v25H7z" stroke="currentColor" stroke-width="1.2" />
        <path d="M3 9v25h17M11 5v25M15 11h5M15 15h5" stroke="currentColor" stroke-width="1.2" />
        <path d="M19 3v6l2-1.4L23 9V3z" fill="currentColor" />
      </svg>
      <span class="brand-text">
        <span class="brand-name">Kiroku</span>
      </span>
    </RouterLink>

    <nav class="nav" aria-label="主导航">
      <RouterLink
        to="/library"
        class="nav-item"
        active-class="is-active"
        @click="viewState.resetFilters()"
      >
        <Library :size="16" class="nav-icon" aria-hidden="true" />
        <span class="nav-label">我的番剧库</span>
        <span class="nav-count">{{ store.count }}</span>
      </RouterLink>
      <RouterLink to="/insights" class="nav-item" active-class="is-active">
        <BarChart3 :size="16" class="nav-icon" aria-hidden="true" />
        <span class="nav-label">评分洞察</span>
      </RouterLink>
      <RouterLink to="/settings" class="nav-item" active-class="is-active">
        <Settings :size="16" class="nav-icon" aria-hidden="true" />
        <span class="nav-label">备份与数据</span>
      </RouterLink>
    </nav>

    <div class="tier-section">
      <p class="section-title">我的分档</p>
      <RouterLink
        v-for="tier in store.tiers"
        :key="tier.id"
        :to="{ path: '/library', query: { tier: tier.name } }"
        class="tier-item"
        :class="{ 'is-current': route.path === '/library' && viewState.filters.tier === tier.name }"
        :aria-current="route.path === '/library' && viewState.filters.tier === tier.name ? 'true' : undefined"
        @click="viewState.filters.tier = tier.name"
      >
        <span class="tier-badge" :style="tierBadgeStyle(tier.color)">{{ tier.name }}</span>
        <span class="tier-desc">{{ tierDescription(tier.name, store.tiers) }}</span>
        <span class="tier-count">{{ tierCounts.get(tier.name) ?? 0 }}</span>
      </RouterLink>
    </div>

    <div class="side-foot">
      <div class="theme-switch" role="group" aria-label="外观主题">
        <button
          v-for="opt in themeOptions"
          :key="opt.value"
          type="button"
          class="theme-btn"
          :class="{ 'is-active': themeMode === opt.value }"
          :aria-pressed="themeMode === opt.value"
          :aria-label="opt.label"
          :title="opt.label"
          @click="themeMode = opt.value"
        >
          <component :is="opt.icon" :size="13" aria-hidden="true" />
        </button>
      </div>
      <div v-if="!desktop" class="me">
        <span class="avatar" aria-hidden="true">K</span>
        <span class="me-text">
          <span class="me-name">本地演示</span>
          <span class="me-sub">Demo 数据 · 不持久化</span>
        </span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 216px;
  flex-shrink: 0;
  background: var(--sidebar-bg);
  -webkit-backdrop-filter: blur(24px) saturate(180%);
  backdrop-filter: blur(24px) saturate(180%);
  border-right: 1px solid var(--sidebar-line);
  padding: 28px 12px 20px;
  display: flex;
  flex-direction: column;
  gap: 30px;
  position: sticky;
  top: 0;
  height: 100vh;
  overflow-y: auto;
  color: var(--sidebar-text);
  scrollbar-color: rgba(0, 0, 0, 0.2) transparent;
}

.drag-pad {
  height: 38px;
  margin: -28px -12px 0;
  padding: 0 12px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  user-select: none;
  -webkit-user-select: none;
}

.pad-mark {
  width: 13px;
  height: 16px;
  color: var(--brand);
  flex-shrink: 0;
}

.pad-name {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--muted);
  white-space: nowrap;
}

.brand {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 0 12px;
  color: var(--sidebar-text);
  text-decoration: none;
}

.brand:hover {
  text-decoration: none;
}

.brand-mark {
  width: 26px;
  height: 32px;
  flex-shrink: 0;
  color: var(--brand);
}

.brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.brand-name {
  font-family: var(--font-logo);
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--sidebar-text);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 9px;
  min-height: 34px;
  padding: 7px 12px;
  border-radius: var(--radius-sm);
  color: var(--sidebar-text);
  font-size: 13px;
  text-decoration: none;
  transition: background var(--motion-fast) var(--ease-snap),
    transform 100ms ease-out;
}

.nav-item:active {
  transform: scale(0.98);
}

.nav-item:hover {
  background: var(--sidebar-raised);
  text-decoration: none;
}

.nav-item.is-active {
  background: var(--sidebar-active);
  font-weight: 600;
}

.nav-item.is-active .nav-icon {
  color: var(--brand);
}

.nav-icon {
  color: var(--sidebar-muted);
  flex-shrink: 0;
}

.nav-item:hover .nav-icon {
  color: var(--sidebar-text);
}

.nav-label {
  flex: 1;
}

.nav-count {
  font-size: 11px;
  color: var(--sidebar-muted);
  padding-left: 3px;
}

.tier-section {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-top: 22px;
  border-top: 1px solid var(--sidebar-line);
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--sidebar-muted);
  margin: 0 12px 8px;
}

.tier-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  text-decoration: none;
  color: var(--sidebar-muted);
  font-size: 12px;
  transition: background var(--motion-fast) var(--ease-snap),
    transform 100ms ease-out;
}

.tier-item:active {
  transform: scale(0.98);
}

.tier-item .tier-badge {
  min-width: 22px;
  min-height: 22px;
  font-size: 11px;
  max-width: 62px;
}

.tier-item:hover, .tier-item.is-current {
  background: var(--sidebar-raised);
  color: var(--sidebar-text);
  text-decoration: none;
}

.tier-item.is-current {
  background: var(--sidebar-active);
}

.tier-desc {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}

.tier-count {
  font-size: 11px;
  color: var(--sidebar-muted);
}

.side-foot {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px 12px 0;
  border-top: 1px solid var(--sidebar-line);
}

.theme-switch {
  display: flex;
  gap: 2px;
  padding: 2px;
  background: var(--sidebar-raised);
  border-radius: var(--radius-md);
}

.theme-btn {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 26px;
  border-radius: 9px;
  color: var(--sidebar-muted);
  transition: background var(--motion-fast) var(--ease-snap),
    color var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap),
    transform 100ms ease-out;
}

.theme-btn:hover {
  color: var(--sidebar-text);
}

.theme-btn:active {
  transform: scale(0.94);
}

.theme-btn.is-active {
  color: var(--sidebar-text);
  background: var(--surface);
  box-shadow: var(--shadow-thumb);
}

.me {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 0;
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--sidebar-raised);
  color: var(--sidebar-muted);
  font-family: var(--font-logo);
  font-size: 16px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.me-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  line-height: 1.3;
}

.me-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--sidebar-text);
}

.me-sub {
  font-size: 10px;
  color: var(--sidebar-muted);
}

.sidebar :focus-visible {
  outline-color: var(--brand);
}

@media (max-width: 1050px) {
  .sidebar {
    width: 184px;
    padding: 24px 12px 16px;
    gap: 26px;
  }

  .brand {
    padding: 0 7px;
    gap: 9px;
  }

  .brand-name {
    font-size: 25px;
  }

  .nav-item {
    padding: 9px 10px;
    gap: 8px;
  }

  .tier-item {
    padding: 6px 10px;
    gap: 8px;
  }
}

@media (max-height: 680px) {
  .sidebar {
    gap: 20px;
    padding-top: 22px;
  }

  .tier-section {
    padding-top: 16px;
    gap: 1px;
  }

  .side-foot {
    gap: 8px;
    padding-top: 10px;
  }
}
</style>
