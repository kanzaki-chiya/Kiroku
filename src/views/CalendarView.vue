<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef } from 'vue'
import { useRouter } from 'vue-router'
import { RotateCcw } from 'lucide-vue-next'
import { bangumiApi } from '../api/bangumi'
import AnimeCover from '../components/AnimeCover.vue'
import { useLibraryStore } from '../stores/library'
import type { CalendarDay } from '../types/anime'
import { formatLabels, formatScore } from '../utils/format'

const store = useLibraryStore()
const router = useRouter()

const days = shallowRef<CalendarDay[]>([])
const phase = shallowRef<'loading' | 'done' | 'error'>('loading')
let abort: AbortController | null = null

const todayWeekday = ((new Date().getDay() + 6) % 7) + 1

const visibleDays = computed(() => days.value.filter(day => day.items.length > 0))
const totalShows = computed(() => days.value.reduce((sum, day) => sum + day.items.length, 0))

async function load() {
  abort?.abort()
  const controller = new AbortController()
  abort = controller
  phase.value = 'loading'
  try {
    const list = await bangumiApi.getCalendar(controller.signal)
    days.value = list
    phase.value = 'done'
  } catch (error) {
    if (controller.signal.aborted) return
    days.value = []
    phase.value = 'error'
    void error
  }
}

function collect(subjectId: number) {
  router.push({ path: '/add', query: { subject: String(subjectId), status: 'planned' } })
}

onMounted(load)
onBeforeUnmount(() => abort?.abort())
</script>

<template>
  <div class="page">
    <nav class="breadcrumb" aria-label="面包屑">
      <RouterLink to="/library">我的空间</RouterLink>
      <span class="sep">/</span>
      <span aria-current="page">每日放送</span>
    </nav>

    <header class="page-head">
      <div>
        <h1 class="page-title">每日放送</h1>
        <p class="page-sub">Bangumi 本周放送表 · 共 {{ totalShows }} 部在播</p>
      </div>
      <button type="button" class="btn btn-ghost" :disabled="phase === 'loading'" @click="load">
        <RotateCcw :size="14" aria-hidden="true" />刷新
      </button>
    </header>

    <p v-if="phase === 'loading'" class="cal-state" role="status">正在读取放送表…</p>

    <div v-else-if="phase === 'error'" class="cal-state error" role="alert">
      <span>放送表读取失败，请稍后再试。</span>
      <button type="button" class="cal-retry" @click="load">
        <RotateCcw :size="12" aria-hidden="true" />重试
      </button>
    </div>

    <p v-else-if="visibleDays.length === 0" class="cal-state">本周没有在播作品。</p>

    <div v-else class="cal-grid">
      <section
        v-for="day in visibleDays"
        :key="day.weekday"
        class="cal-day"
        :class="{ 'is-today': day.weekday === todayWeekday }"
      >
        <h2 class="cal-day-title">
          {{ day.label }}
          <span v-if="day.weekday === todayWeekday" class="today-chip">今天</span>
          <span class="cal-count">{{ day.items.length }}</span>
        </h2>
        <ul class="cal-list">
          <li v-for="item in day.items" :key="item.id" class="cal-row">
            <span class="cal-thumb"><AnimeCover :subject="item" /></span>
            <span class="cal-text">
              <span class="cal-title">{{ item.nameCn || item.name }}</span>
              <span class="cal-meta">
                <template v-if="item.community.score !== null">
                  <span class="cal-score">{{ formatScore(item.community.score) }}</span>
                </template>
                {{ item.year || '—' }} · {{ formatLabels[item.format] ?? item.format }}
                <template v-if="item.episodes > 0">· 共 {{ item.episodes }} 话</template>
              </span>
            </span>
            <RouterLink
              v-if="store.hasSubject(item.id)"
              :to="`/anime/${item.id}`"
              class="cal-action"
            >
              已收录
            </RouterLink>
            <button
              v-else
              type="button"
              class="cal-action as-btn"
              @click="collect(item.id)"
            >
              想看
            </button>
          </li>
        </ul>
      </section>
    </div>
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

.cal-state {
  margin: 0;
  padding: 48px 20px;
  font-size: 13px;
  color: var(--muted);
  text-align: center;
  background: var(--surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.cal-state.error {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--danger);
}

.cal-retry {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--brand);
  background: var(--fill);
  border-radius: var(--radius-sm);
  padding: 5px 12px;
  transition: background var(--motion-fast) var(--ease-snap);
}

.cal-retry:hover {
  background: var(--fill-strong);
}

.cal-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.cal-day {
  padding: 18px 20px;
  background: var(--surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  min-width: 0;
}

.cal-day.is-today {
  outline: 1.5px solid var(--brand);
  outline-offset: -1.5px;
}

.cal-day-title {
  margin: 0 0 8px;
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.today-chip {
  padding: 1px 7px;
  border-radius: var(--radius-pill);
  background: var(--brand);
  color: #fff;
  font-size: 10.5px;
  font-weight: 500;
}

.cal-count {
  margin-left: auto;
  font-size: 11.5px;
  font-weight: 500;
  color: var(--muted);
}

.cal-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}

.cal-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 0;
  border-bottom: 1px solid var(--border);
}

.cal-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.cal-thumb {
  width: 38px;
  flex-shrink: 0;
}

.cal-thumb :deep(.cover) {
  border-radius: var(--radius-xs);
}

.cal-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
  flex: 1;
}

.cal-title {
  font-size: 13.5px;
  font-weight: 600;
  letter-spacing: -0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cal-meta {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11.5px;
  color: var(--muted);
}

.cal-score {
  color: var(--brand);
  font-weight: 600;
}

.cal-action {
  flex-shrink: 0;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--brand);
  text-decoration: none;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  transition: background var(--motion-fast) var(--ease-snap);
}

.cal-action:hover {
  background: var(--fill);
  text-decoration: none;
}

.cal-action.as-btn {
  background: var(--fill);
}

.cal-action.as-btn:hover {
  background: var(--fill-strong);
}

@media (max-width: 900px) {
  .cal-grid {
    grid-template-columns: 1fr;
  }
}
</style>
