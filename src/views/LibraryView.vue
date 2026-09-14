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
  const counts = { all: 0, completed: 0, watching: 0, planned: 0 }
  const tier = filters.value.tier
  for (const entry of store.entries) {
    if (tier === 'unassigned' ? entry.personal.tier !== null : tier !== 'all' && entry.personal.tier !== tier) continue
    counts.all += 1
    counts[entry.personal.status] += 1
  }
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
        <p class="page-sub">共收录 {{ store.count }} 部</p>
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

    <Transition name="swap" mode="out-in">
      <TransitionGroup
        v-if="filtered.length"
        key="shelf"
        name="shelf"
        tag="div"
        class="entries"
        :class="`layout-${layout}`"
        appear
      >
        <AnimeCard
          v-for="(entry, index) in filtered"
          :key="entry.subject.id"
          :entry="entry"
          :layout="layout"
          :style="{ '--i': index }"
        />
      </TransitionGroup>

      <div v-else key="empty" class="empty">
        <SearchX :size="34" class="empty-icon" aria-hidden="true" />
        <p class="empty-title">没有找到符合条件的番剧</p>
        <p class="empty-sub">换个关键词，或者清空筛选再看看。</p>
        <button type="button" class="btn btn-ghost" @click="resetAll">清空筛选条件</button>
      </div>
    </Transition>

    <footer class="page-foot">
      <span>显示 {{ filtered.length }} / {{ store.count }} 部</span>
    </footer>
  </div>
</template>

<style scoped>
.page-head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 24px;
}

.page-title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 34px;
  font-weight: 700;
  line-height: 1.15;
  letter-spacing: -0.025em;
}

.page-sub {
  margin: 8px 0 0;
  font-size: 13px;
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

.shelf-move {
  transition: transform 360ms var(--ease-spring);
}

.shelf-enter-active {
  transition: opacity 240ms var(--ease-snap), transform 240ms var(--ease-snap);
}

.shelf-leave-active {
  transition: opacity 150ms ease-out, transform 150ms ease-out;
  position: absolute;
}

.shelf-enter-from,
.shelf-leave-to {
  opacity: 0;
  transform: scale(0.94);
}

.shelf-appear-active {
  transition: opacity 320ms var(--ease-snap), transform 320ms var(--ease-snap);
  transition-delay: min(360ms, calc(var(--i, 0) * 24ms));
}

.shelf-appear-from {
  opacity: 0;
  transform: translateY(12px) scale(0.97);
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 72px 20px;
  border-radius: var(--radius-lg);
  background: var(--surface);
  box-shadow: var(--shadow-card);
  text-align: center;
}

.empty-icon {
  color: var(--muted);
  width: 40px;
  height: 40px;
  padding: 12px;
  background: var(--fill);
  border-radius: 50%;
  box-sizing: content-box;
}

.empty-title {
  margin: 8px 0 0;
  font: 700 20px/1.3 var(--font-display);
  letter-spacing: -0.02em;
}

.empty-sub {
  margin: 0 0 14px;
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
