<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { BarChart3, Library, Settings } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import { useLibraryViewStore } from '../stores/libraryView'
import { tierBadgeStyle, tierDescription } from '../utils/format'

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
      <svg class="brand-mark" viewBox="0 0 28 28" aria-hidden="true">
        <rect x="9.5" y="3" width="15" height="19" rx="2.4" fill="#335d4e" opacity="0.32" />
        <rect x="6" y="5.5" width="15" height="19" rx="2.4" fill="#335d4e" opacity="0.62" />
        <rect x="2.5" y="8" width="15" height="17" rx="2.4" fill="#335d4e" />
        <line x1="6.5" y1="13.5" x2="13.5" y2="13.5" stroke="#eef2ed" stroke-width="1.4" stroke-linecap="round" />
        <line x1="6.5" y1="17" x2="11.5" y2="17" stroke="#eef2ed" stroke-width="1.4" stroke-linecap="round" />
        <circle cx="19.6" cy="6.4" r="2.1" fill="#bd592e" />
      </svg>
      <span class="brand-text">
        <span class="brand-name">Kiroku</span>
        <span class="brand-sub">私人番剧记录</span>
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

    <div class="me">
      <span class="avatar" aria-hidden="true">K</span>
      <span class="me-text">
        <span class="me-name">我的观影空间</span>
        <span class="me-sub">Demo · 本地示例</span>
      </span>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 216px;
  flex-shrink: 0;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border);
  padding: 22px 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 22px;
  position: sticky;
  top: 0;
  height: 100vh;
  overflow-y: auto;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 6px;
  text-decoration: none;
}

.brand:hover {
  text-decoration: none;
}

.brand-mark {
  width: 30px;
  height: 30px;
  flex-shrink: 0;
}

.brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.brand-name {
  font-family: var(--font-logo);
  font-style: italic;
  font-size: 21px;
  color: var(--brand-deep);
  letter-spacing: 0.01em;
}

.brand-sub {
  font-size: 11px;
  color: var(--muted);
  margin-top: 2px;
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  color: var(--text-soft);
  font-size: 14px;
  text-decoration: none;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.65);
  text-decoration: none;
}

.nav-item.is-active {
  background: var(--surface);
  color: var(--brand-deep);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(37, 53, 47, 0.06);
}

.nav-icon {
  color: var(--muted);
  flex-shrink: 0;
}

.nav-item.is-active .nav-icon {
  color: var(--brand);
}

.nav-label {
  flex: 1;
}

.nav-count {
  font-size: 12px;
  color: var(--muted);
  background: rgba(255, 255, 255, 0.7);
  border-radius: 999px;
  padding: 1px 8px;
}

.nav-item.is-active .nav-count {
  color: var(--brand);
  background: var(--sidebar-bg);
}

.tier-section {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.section-title {
  font-size: 12px;
  color: var(--muted);
  letter-spacing: 0.06em;
  margin: 0 10px 6px;
}

.tier-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  text-decoration: none;
  color: var(--text-soft);
  font-size: 13.5px;
}

.tier-item:hover {
  background: rgba(255, 255, 255, 0.65);
  text-decoration: none;
}

.tier-item.is-current {
  background: var(--surface);
  box-shadow: 0 1px 2px rgba(37, 53, 47, 0.06);
}

.tier-desc {
  flex: 1;
}

.tier-count {
  font-size: 12px;
  color: var(--muted);
}

.me {
  margin-top: auto;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 8px 0;
  border-top: 1px solid var(--border);
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--brand);
  color: #fff;
  font-family: var(--font-logo);
  font-style: italic;
  font-size: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.me-text {
  display: flex;
  flex-direction: column;
  line-height: 1.3;
}

.me-name {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text);
}

.me-sub {
  font-size: 11.5px;
  color: var(--muted);
}

@media (min-width: 701px) and (max-width: 1000px) {
  .sidebar {
    width: 188px;
  }
}

@media (max-width: 700px) {
  .sidebar {
    width: 100%;
    height: auto;
    position: static;
    overflow: visible;
    padding: 14px 18px;
    gap: 14px;
    border-right: 0;
    border-bottom: 1px solid var(--border);
  }

  .brand-sub,
  .me,
  .tier-section {
    display: none;
  }

  .nav {
    flex-direction: row;
    gap: 8px;
  }

  .nav-item {
    min-height: 40px;
    flex: 1;
  }

  .brand-mark {
    width: 26px;
    height: 26px;
  }
}
</style>
