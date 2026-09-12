<script setup lang="ts">
import { computed, ref } from 'vue'
import { ArrowUpDown, LayoutGrid, List, RotateCcw, Search, SlidersHorizontal } from 'lucide-vue-next'
import type { LibraryFilters, TierDefinition, WatchStatus } from '../types/anime'
import { statusLabels } from '../utils/format'

const filters = defineModel<LibraryFilters>({ required: true })
const layout = defineModel<'grid' | 'list'>('layout', { required: true })

const props = defineProps<{
  years: number[]
  statusCounts: Record<WatchStatus | 'all', number>
  tiers: TierDefinition[]
}>()

const expanded = ref(false)

const statusTabs: { value: WatchStatus | 'all'; label: string }[] = [
  { value: 'all', label: '全部作品' },
  { value: 'completed', label: statusLabels.completed },
  { value: 'watching', label: statusLabels.watching },
  { value: 'planned', label: statusLabels.planned }
]

const sortOptions: { value: LibraryFilters['sort']; label: string }[] = [
  { value: 'updated', label: '最近更新' },
  { value: 'personal', label: '我的评分' },
  { value: 'community', label: 'Bangumi 评分' },
  { value: 'title', label: '标题' }
]

const tierOptions = computed(() => [
  { value: 'all', label: '全部分档' },
  ...props.tiers.map(tier => ({
    value: tier.name,
    label: `${tier.name} · ${tier.description}`
  })),
  { value: 'unassigned', label: '未分档' }
])

const hasActiveFilters = computed(
  () =>
    filters.value.query.trim() !== '' ||
    filters.value.tier !== 'all' ||
    filters.value.status !== 'all' ||
    filters.value.year !== 'all'
)

function clearFilters() {
  filters.value = { ...filters.value, query: '', tier: 'all', status: 'all', year: 'all' }
}

function toggleDirection() {
  filters.value = {
    ...filters.value,
    direction: filters.value.direction === 'asc' ? 'desc' : 'asc'
  }
}
</script>

<template>
  <div class="toolbar">
    <div class="tabs" role="group" aria-label="按观看状态筛选">
      <button
        v-for="tab in statusTabs"
        :key="tab.value"
        type="button"
        :aria-pressed="filters.status === tab.value"
        class="tab"
        :class="{ 'is-active': filters.status === tab.value }"
        @click="filters = { ...filters, status: tab.value }"
      >
        {{ tab.label }}
        <span class="tab-count">{{ statusCounts[tab.value] }}</span>
      </button>
    </div>

    <div class="controls">
      <label class="search-box">
        <Search :size="15" class="search-icon" aria-hidden="true" />
        <span class="sr-only">搜索标题或标签</span>
        <input
          v-model="filters.query"
          class="search-input"
          type="search"
          placeholder="搜索标题、原名或标签…"
          aria-label="搜索番剧"
        />
      </label>

      <button
        class="btn btn-ghost filter-toggle"
        :aria-expanded="expanded"
        :class="{ 'is-on': expanded || hasActiveFilters }"
        @click="expanded = !expanded"
      >
        <SlidersHorizontal :size="14" aria-hidden="true" />筛选
      </button>

      <label class="sort-field">
        <span class="sr-only">排序方式</span>
        <select v-model="filters.sort" class="field-select" aria-label="排序方式">
          <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
            按{{ opt.label }}
          </option>
        </select>
      </label>

      <button
        class="btn btn-ghost icon-btn"
        :title="filters.direction === 'desc' ? '降序，点击切换为升序' : '升序，点击切换为降序'"
        :aria-label="filters.direction === 'desc' ? '当前降序' : '当前升序'"
        @click="toggleDirection"
      >
        <ArrowUpDown :size="14" aria-hidden="true" />
      </button>

      <div class="layout-switch" role="group" aria-label="切换布局">
        <button
          class="layout-btn"
          :class="{ 'is-active': layout === 'grid' }"
          title="网格视图"
          aria-label="网格视图"
          :aria-pressed="layout === 'grid'"
          @click="layout = 'grid'"
        >
          <LayoutGrid :size="15" aria-hidden="true" />
        </button>
        <button
          class="layout-btn"
          :class="{ 'is-active': layout === 'list' }"
          title="列表视图"
          aria-label="列表视图"
          :aria-pressed="layout === 'list'"
          @click="layout = 'list'"
        >
          <List :size="15" aria-hidden="true" />
        </button>
      </div>
    </div>

    <div v-if="expanded" class="filter-row">
      <label class="filter-field">
        <span class="filter-label">分档</span>
        <select v-model="filters.tier" class="field-select">
          <option v-for="opt in tierOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </label>
      <label class="filter-field">
        <span class="filter-label">年份</span>
        <select v-model="filters.year" class="field-select">
          <option value="all">全部年份</option>
          <option v-for="year in props.years" :key="year" :value="year">{{ year }}</option>
        </select>
      </label>
      <button class="btn btn-ghost clear-btn" :disabled="!hasActiveFilters" @click="clearFilters">
        <RotateCcw :size="13" aria-hidden="true" />清空筛选
      </button>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
  white-space: nowrap;
}

.tabs {
  display: inline-flex;
  align-items: center;
  align-self: flex-start;
  gap: 2px;
  padding: 2px;
  background: var(--fill);
  border-radius: var(--radius-md);
}

.tab {
  position: relative;
  flex-shrink: 0;
  padding: 6px 14px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-soft);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border-radius: 10px;
  transition: color var(--motion-fast) var(--ease-snap),
    background var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap),
    transform 100ms ease-out;
}

.tab:active {
  transform: scale(0.97);
}

.tab:hover {
  color: var(--text);
}

.tab.is-active {
  color: var(--text);
  font-weight: 600;
  background: var(--surface);
  box-shadow: var(--shadow-thumb);
}

.tab-count {
  font-size: 10px;
  color: var(--muted);
  background: var(--fill);
  border-radius: var(--radius-pill);
  padding: 1px 6px;
}

.tab.is-active .tab-count {
  color: var(--brand);
  background: var(--brand-soft);
}

.controls {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 170px;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: var(--muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 34px;
  padding: 0 12px 0 34px;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--fill);
  font-size: 13px;
  transition: background var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap);
}

.search-input:focus {
  background: var(--surface);
  outline: none;
  box-shadow: var(--focus-ring), inset 0 0 0 1px var(--brand);
}

.filter-toggle {
  font-size: 13px;
}

.filter-toggle.is-on {
  background: var(--brand-soft);
  color: var(--brand);
}

.sort-field .field-select {
  font-size: 13px;
  height: 34px;
  border: none;
  background: var(--fill);
  width: 132px;
  border-radius: var(--radius-sm);
}

.sort-field .field-select:focus {
  box-shadow: var(--focus-ring);
}

.icon-btn {
  width: 34px;
  min-height: 34px;
  padding: 0;
  justify-content: center;
}

.layout-switch {
  display: flex;
  padding: 2px;
  gap: 2px;
  border-radius: var(--radius-sm);
  background: var(--fill);
}

.layout-btn {
  width: 30px;
  height: 28px;
  border-radius: var(--radius-xs);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--muted);
  transition: background var(--motion-fast) var(--ease-snap),
    color var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap);
}

.layout-btn:hover {
  color: var(--text);
}

.layout-btn.is-active {
  background: var(--surface);
  color: var(--text);
  box-shadow: var(--shadow-thumb);
}

.filter-row {
  display: flex;
  align-items: flex-end;
  gap: 16px;
  padding: 16px;
  background: var(--surface);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-card);
  flex-wrap: wrap;
}

.filter-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
}

.filter-field .field-select {
  max-width: 100%;
  font-size: 13px;
}

.filter-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--muted);
}

.clear-btn {
  margin-left: auto;
  font-size: 13px;
}
</style>
