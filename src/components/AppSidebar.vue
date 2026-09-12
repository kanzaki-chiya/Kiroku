<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { BarChart3, Library, Settings } from 'lucide-vue-next'
import { isTauri } from '../runtime'
import { useLibraryStore } from '../stores/library'
import { useLibraryViewStore } from '../stores/libraryView'
import { tierBadgeStyle, tierDescription } from '../utils/format'

const desktop = isTauri()
const store = useLibraryStore()
const viewState = useLibraryViewStore()
const route = useRoute()

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
    <RouterLink to="/library" class="brand">
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

    <div v-if="!desktop" class="me">
      <span class="avatar" aria-hidden="true">K</span>
      <span class="me-text">
        <span class="me-name">本地演示</span>
        <span class="me-sub">Demo 数据 · 不持久化</span>
      </span>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 208px;
  flex-shrink: 0;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--sidebar-line);
  padding: 30px 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 32px;
  position: sticky;
  top: 0;
  height: 100vh;
  overflow-y: auto;
  color: var(--sidebar-text);
  scrollbar-color: var(--sidebar-line) transparent;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 10px;
  color: var(--sidebar-text);
  text-decoration: none;
}

.brand:hover {
  text-decoration: none;
}

.brand-mark {
  width: 30px;
  height: 36px;
  flex-shrink: 0;
}

.brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.brand-name {
  font-family: var(--font-logo);
  font-size: 27px;
  letter-spacing: 0.01em;
  color: var(--sidebar-text);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 42px;
  padding: 9px 12px;
  border-radius: var(--radius-sm);
  color: var(--sidebar-muted);
  font-size: 13px;
  text-decoration: none;
  transition: background var(--motion-fast), color var(--motion-fast);
}

.nav-item:hover {
  background: var(--sidebar-raised);
  color: var(--sidebar-text);
  text-decoration: none;
}

.nav-item.is-active {
  background: var(--sidebar-raised);
  color: var(--sidebar-text);
  font-weight: 600;
}

.nav-item.is-active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 13px;
  bottom: 13px;
  width: 2px;
  border-radius: var(--radius-xs);
  background: var(--sidebar-muted);
}

.nav-icon {
  color: currentColor;
  flex-shrink: 0;
}

.nav-label {
  flex: 1;
}

.nav-count {
  font-family: var(--font-number);
  font-size: 11px;
  color: var(--sidebar-text);
  padding-left: 3px;
}

.tier-section {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding-top: 24px;
  border-top: 1px solid var(--sidebar-line);
}

.section-title {
  font-size: 10px;
  color: var(--sidebar-muted);
  letter-spacing: .12em;
  margin: 0 12px 10px;
}

.tier-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 12px;
  border-radius: var(--radius-sm);
  text-decoration: none;
  color: var(--sidebar-muted);
  font-size: 12px;
  transition: background var(--motion-fast), color var(--motion-fast);
}

.tier-item .tier-badge {
  min-width: 23px;
  min-height: 23px;
  font-size: 11px;
  max-width: 62px;
}

.tier-item:hover, .tier-item.is-current {
  background: var(--sidebar-raised);
  color: var(--sidebar-text);
  text-decoration: none;
}

.tier-desc {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}

.tier-count {
  font: 11px var(--font-number);
  color: var(--sidebar-muted);
}

.me {
  margin-top: auto;
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 18px 8px 0;
  border-top: 1px solid var(--sidebar-line);
}

.avatar {
  width: 34px;
  height: 34px;
  border: 1px solid var(--sidebar-line);
  border-radius: var(--radius-sm);
  background: var(--sidebar-raised);
  color: var(--sidebar-text);
  font-family: var(--font-logo);
  font-size: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.me-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
  line-height: 1.3;
}

.me-name {
  font-size: 11px;
  font-weight: 500;
  color: var(--sidebar-text);
}

.me-sub {
  font-size: 10px;
  color: var(--sidebar-muted);
}

.sidebar :focus-visible {
  outline-color: var(--sidebar-text);
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

  .me {
    padding-top: 12px;
  }
}
</style>
