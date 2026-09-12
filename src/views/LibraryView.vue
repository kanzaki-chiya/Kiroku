<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { Plus, SearchX } from 'lucide-vue-next'
import AnimeCard from '../components/AnimeCard.vue'
import FeaturedPanel from '../components/FeaturedPanel.vue'
import LibraryToolbar from '../components/LibraryToolbar.vue'
import { useLibraryStore } from '../stores/library'
import { useLibraryViewStore } from '../stores/libraryView'
import type { WatchStatus } from '../types/anime'
import { filterLibrary } from '../utils/library'

const store = useLibraryStore()
const viewState = useLibraryViewStore()
const { filters, layout } = storeToRefs(viewState)
const route = useRoute()
const router = useRouter()

const allowedTiers = computed(() => new Set([...store.tiers.map(tier => tier.name), 'unassigned']))

watch(
  () => [route.query.tier, allowedTiers.value] as const,
  () => {
    const tier = route.query.tier
    if (tier === undefined) return
    if (typeof tier === 'string' && allowedTiers.value.has(tier) && filters.value.tier !== tier) {
      filters.value = { ...filters.value, tier }
    }
  },
  { immediate: true }
)

watch(
  () => filters.value.tier,
  tier => {
    if (route.path !== '/library') return
    if (String(route.query.tier ?? 'all') === tier) return
    router.replace({ query: { ...route.query, tier: tier === 'all' ? undefined : tier } })
  }
)

const statusCounts = computed<Record<WatchStatus | 'all', number>>(() => {
  const counts = { all: store.entries.length, completed: 0, watching: 0, planned: 0 }
  for (const entry of store.entries) counts[entry.personal.status] += 1
  return counts
})

const years = computed(() =>
  [...new Set(store.entries.map(e => e.subject.year))].sort((a, b) => b - a)
)

const filtered = computed(() => filterLibrary(store.entries, filters.value))

const featured = computed(() => {
  const list = store.entries
  if (!list.length) return null
  return [...list].sort(
    (a, b) => b.personal.updatedAt.localeCompare(a.personal.updatedAt)
  )[0]
})

function resetAll() {
  viewState.resetFilters()
  router.replace({ path: '/library' })
}
</script>

<template>
  <div class="page">
    <nav class="breadcrumb" aria-label="面包屑">
      <RouterLink to="/library">我的空间</RouterLink>
      <span class="sep">/</span>
      <span aria-current="page">番剧库</span>
    </nav>

    <header class="page-head">
      <div>
        <h1 class="page-title">我的番剧库</h1>
        <p class="page-sub">共收录 {{ store.count }} 部 · 每一部，都有自己的回响。</p>
      </div>
      <RouterLink to="/add" class="btn btn-primary">
        <Plus :size="15" aria-hidden="true" />收录番剧
      </RouterLink>
    </header>

    <FeaturedPanel v-if="featured" :entry="featured" class="featured-panel" />

    <LibraryToolbar
      v-model="filters"
      v-model:layout="layout"
      :years="years"
      :status-counts="statusCounts"
      :tiers="store.tiers"
      class="toolbar-block"
    />

    <div v-if="filtered.length" class="entries" :class="`layout-${layout}`">
      <AnimeCard
        v-for="entry in filtered"
        :key="entry.subject.id"
        :entry="entry"
        :layout="layout"
      />
    </div>

    <div v-else class="empty">
      <SearchX :size="34" class="empty-icon" aria-hidden="true" />
      <p class="empty-title">没有找到符合条件的番剧</p>
      <p class="empty-sub">换个关键词，或者清空筛选再看看。</p>
      <button type="button" class="btn btn-ghost" @click="resetAll">清空筛选条件</button>
    </div>

    <footer class="page-foot">
      <span>显示 {{ filtered.length }} / {{ store.count }} 部</span>
    </footer>
  </div>
</template>

<style scoped>
.page-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 22px;
}

.page-title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 32px;
  font-weight: 600;
  line-height: 1.4;
  letter-spacing: .04em;
}

.page-sub {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--muted);
}

.featured-panel {
  margin-bottom: 24px;
}

.toolbar-block {
  margin-bottom: 20px;
}

.entries.layout-grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 22px 18px;
}

.entries.layout-list {
  display: flex;
  flex-direction: column;
  gap: 9px;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 64px 20px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-md);
  background: var(--surface);
  text-align: center;
}

.empty-icon {
  color: var(--brand);
}

.empty-title {
  margin: 6px 0 0;
  font: 600 22px var(--font-display);
}

.empty-sub {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--muted);
}

.page-foot {
  margin-top: 28px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  gap: 16px;
  font-size: 11px;
  color: var(--muted);
}

@media (min-width: 1450px) {
  .entries.layout-grid {
    grid-template-columns: repeat(6, minmax(0, 1fr));
  }
}

@media (max-width: 1120px) {
  .entries.layout-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 18px 14px;
  }
}

@media (max-width: 950px) {
  .entries.layout-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .page-title {
    font-size: 29px;
  }
}
</style>
