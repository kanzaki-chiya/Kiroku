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
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--border);
  overflow-x: auto;
}

.tab {
  flex-shrink: 0;
  padding: 8px 14px;
  font-size: 14px;
  color: var(--muted);
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.tab:hover {
  color: var(--text);
}

.tab.is-active {
  color: var(--brand-deep);
  font-weight: 600;
  border-bottom-color: var(--brand);
}

.tab-count {
  font-size: 11.5px;
  color: var(--muted);
  background: var(--sage-tint);
  border-radius: 999px;
  padding: 0 7px;
}

.tab.is-active .tab-count {
  color: var(--brand);
}

.controls {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 220px;
  max-width: none;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 11px;
  color: var(--muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 36px;
  padding: 0 12px 0 33px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
}

.search-input:focus {
  border-color: var(--brand);
  outline: none;
  box-shadow: 0 0 0 3px rgba(51, 93, 78, 0.14);
}

.filter-toggle.is-on {
  border-color: var(--brand);
  color: var(--brand);
}

.icon-btn {
  width: 36px;
  padding: 0;
  justify-content: center;
}

.layout-switch {
  display: flex;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--surface);
}

.layout-btn {
  width: 36px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--muted);
}

.layout-btn + .layout-btn {
  border-left: 1px solid var(--border);
}

.layout-btn.is-active {
  background: var(--sage-tint);
  color: var(--brand-deep);
}

.filter-row {
  display: flex;
  align-items: flex-end;
  gap: 16px;
  padding: 14px 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
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
}

.filter-label {
  font-size: 12.5px;
  color: var(--muted);
}

.clear-btn {
  margin-left: auto;
}

@media (max-width: 520px) {
  .tab {
    padding: 8px 9px;
    font-size: 13px;
  }

  .controls {
    gap: 8px;
  }

  .search-box {
    flex-basis: 100%;
    min-width: 0;
  }

  .filter-row {
    padding: 12px;
    gap: 12px;
  }

  .clear-btn {
    margin-left: 0;
  }

  .filter-field {
    flex: 1 1 120px;
  }
}
</style>
