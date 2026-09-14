<script setup lang="ts">
import { computed, nextTick, shallowRef, ref, watch, type Ref } from 'vue'
import { Crown, RotateCcw } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import TasteProfile from '../components/TasteProfile.vue'
import { formatDelta, formatScore } from '../utils/format'
import type { LibraryStatistics } from '../utils/statistics'

const store = useLibraryStore()
const stats = shallowRef<LibraryStatistics | null>(null)
const statsError = shallowRef('')
const barsReady = ref(false)

const reducedMotion =
  typeof window !== 'undefined' &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches

function useCountUp(source: () => number | null, round = false): Readonly<Ref<number>> {
  const shown = ref(0)
  let raf = 0
  watch(
    source,
    target => {
      cancelAnimationFrame(raf)
      if (target === null || reducedMotion) {
        shown.value = target ?? 0
        return
      }
      const from = shown.value
      const started = performance.now()
      const tick = (now: number) => {
        const p = Math.min(1, (now - started) / 650)
        const ease = 1 - Math.pow(1 - p, 3)
        shown.value = from + (target - from) * ease
        if (p < 1) raf = requestAnimationFrame(tick)
      }
      raf = requestAnimationFrame(tick)
    },
    { immediate: true }
  )
  return round ? computed(() => Math.round(shown.value)) : shown
}

const totalCount = useCountUp(() => stats.value?.total ?? null, true)
const ratedCount = useCountUp(() => stats.value?.ratedCount ?? null, true)
const myMean = useCountUp(() => stats.value?.personalMean ?? null)
const communityMean = useCountUp(() => stats.value?.communityMean ?? null)
const meanDiff = useCountUp(() => stats.value?.meanDifference ?? null)
const highestScore = useCountUp(() => stats.value?.highest?.personal.score ?? null)

async function load() {
  try {
    statsError.value = ''
    stats.value = await store.getStatistics()
  } catch (error) {
    stats.value = null
    statsError.value = error instanceof Error ? error.message : '统计读取失败'
  }
}

watch(() => store.count, () => { void load() }, { immediate: true })

watch(
  () => stats.value,
  value => {
    barsReady.value = false
    if (!value) return
    nextTick(() => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          barsReady.value = true
        })
      })
    })
  }
)

const maxBin = computed(() => Math.max(1, ...(stats.value?.bins.map(b => b.count) ?? [0])))
</script>

<template>
  <div class="page">
    <nav class="breadcrumb" aria-label="面包屑">
      <RouterLink to="/library">我的空间</RouterLink>
      <span class="sep">/</span>
      <span aria-current="page">评分洞察</span>
    </nav>

    <header class="page-head">
      <h1 class="page-title">评分洞察</h1>
      <p class="page-sub">看看我的口味，和社区差了多少。</p>
    </header>

    <template v-if="stats">
    <div class="stat-cards">
      <div class="stat">
        <span class="stat-label">收录总数</span>
        <span class="stat-value">{{ totalCount }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">已评分</span>
        <span class="stat-value">{{ ratedCount }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">我的均分</span>
        <span class="stat-value mine">{{ stats.personalMean === null ? '—' : myMean.toFixed(1) }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Bangumi 均分<span class="mock">Mock</span></span>
        <span class="stat-value community">{{ stats.communityMean === null ? '—' : communityMean.toFixed(1) }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">平均差值</span>
        <span class="stat-value" :class="{ mine: (stats.meanDifference ?? 0) > 0, community: (stats.meanDifference ?? 0) < 0 }">
          {{ stats.meanDifference === null ? '—' : formatDelta(meanDiff) }}
        </span>
      </div>
    </div>

    <section class="panel">
      <h2 class="panel-title">我的评分分布</h2>
      <ul class="bins" aria-label="评分分布">
        <li v-for="bin in stats.bins" :key="bin.label" class="bin-row">
          <span class="bin-label">{{ bin.label }}</span>
          <div class="bin-track" role="img" :aria-label="`${bin.label} 分区间：${bin.count} 部`">
            <div class="bin-fill" :style="{ width: barsReady ? `${(bin.count / maxBin) * 100}%` : '0%' }"></div>
          </div>
          <span class="bin-count">{{ bin.count }} 部</span>
        </li>
      </ul>
    </section>

    <TasteProfile :dimensions="stats.dimensions" :ready="barsReady || reducedMotion" />

    <div class="two-col">
      <section class="panel">
        <h2 class="panel-title">与社区分歧最大</h2>
        <p v-if="!stats.differences.length" class="panel-empty">还没有可比较的作品。</p>
        <ul v-else class="diff-list">
          <li v-for="diff in stats.differences.slice(0, 8)" :key="diff.entry.subject.id">
            <RouterLink :to="`/anime/${diff.entry.subject.id}`" class="diff-row">
              <span class="diff-title">{{ diff.entry.subject.nameCn }}</span>
              <span class="diff-scores">
                我的 {{ formatScore(diff.entry.personal.score) }} /
                社区 {{ formatScore(diff.entry.subject.community.score) }}
              </span>
              <span class="diff-delta" :class="{ positive: diff.delta > 0, negative: diff.delta < 0 }">
                {{ formatDelta(diff.delta) }}
              </span>
            </RouterLink>
          </li>
        </ul>
      </section>

      <section class="panel">
        <h2 class="panel-title">私心最高</h2>
        <p v-if="!stats.highest" class="panel-empty">还没有打过分的作品。</p>
        <RouterLink v-else :to="`/anime/${stats.highest.subject.id}`" class="highest">
          <Crown :size="16" class="crown" aria-hidden="true" />
          <span class="highest-text">
            <span class="highest-name">{{ stats.highest.subject.nameCn }}</span>
            <span class="highest-sub">{{ stats.highest.subject.name }}</span>
          </span>
          <span class="highest-score">{{ formatScore(highestScore) }}</span>
        </RouterLink>
      </section>
    </div>

    <p class="footnote">
      统计范围：当前番剧库全部 {{ stats.total }} 部。我的均分 = 所有已打总分作品（{{ stats.ratedCount }}
      部）的平均；Bangumi 社区均分 = 有社区评分的 {{ stats.communityCount }} 部的平均（Mock
      快照，未按票数加权）；平均差值 = 两者皆有评分的 {{ stats.pairedCount }} 部中「我的 −
      Bangumi 社区」的均值，正数代表我更喜欢。口味画像按各维已评作品独立计算均分（0–5 星），未评不计入样本；不要求已打总分，不区分观看状态。分项仅用于画像，总分仍由你独立填写。
    </p>
    </template>
    <div v-else-if="statsError" class="state-line error" role="alert">
      <span>{{ statsError }}</span>
      <button type="button" class="retry" @click="load">
        <RotateCcw :size="13" aria-hidden="true" />重试
      </button>
    </div>
  </div>
</template>

<style scoped>
.page-title {
  margin: 0;
  font: 700 34px/1.15 var(--font-display);
  letter-spacing: -0.025em;
}

.page-sub {
  margin: 8px 0 26px;
  font-size: 13px;
  color: var(--muted);
}

.stat-cards {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  background: var(--surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  margin-bottom: 24px;
  padding: 24px 0;
}

.stat {
  padding: 0 22px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.stat + .stat {
  border-left: 1px solid var(--border);
}

.stat-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted);
  display: inline-flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
}

.mock {
  font-size: 9px;
  padding: 1px 6px;
  border-radius: var(--radius-pill);
  background: var(--fill);
  color: var(--muted);
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
  line-height: 1.1;
  letter-spacing: -0.03em;
}

.stat-value.mine {
  color: var(--score-gold);
}

.stat-value.community {
  color: var(--slate);
}

.panel {
  background: var(--surface);
  border-radius: var(--radius-lg);
  padding: 26px;
  margin-bottom: 22px;
  box-shadow: var(--shadow-card);
  min-width: 0;
}

.panel-title {
  margin: 0 0 20px;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.01em;
  display: flex;
  align-items: center;
  gap: 9px;
}

.panel-empty {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
}

.bins {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  grid-template-columns: 1fr;
  gap: 10px;
}

.bin-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.bin-label {
  width: 46px;
  font-size: 11px;
  color: var(--text-soft);
  flex-shrink: 0;
}

.bin-track {
  flex: 1;
  height: 20px;
  border-radius: var(--radius-pill);
  background: var(--fill);
  overflow: hidden;
}

.bin-fill {
  height: 100%;
  border-radius: var(--radius-pill);
  background: var(--brand);
  min-width: 0;
  transition: width 500ms var(--ease-spring);
}

.bin-count {
  width: 34px;
  font-size: 11px;
  color: var(--muted);
  text-align: right;
  flex-shrink: 0;
}

.two-col {
  display: grid;
  grid-template-columns: minmax(0, 1.55fr) minmax(0, 1fr);
  gap: 22px;
}

.two-col .panel {
  margin-bottom: 0;
}

.diff-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}

.diff-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto 44px;
  align-items: center;
  gap: 10px;
  padding: 11px 0;
  border-bottom: 1px solid var(--border);
  text-decoration: none;
  color: inherit;
  font-size: 13px;
}

.diff-list li:last-child .diff-row {
  border-bottom: none;
}

.diff-row:hover {
  text-decoration: none;
}

.diff-row:hover .diff-title {
  color: var(--brand);
}

.diff-title {
  min-width: 0;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.diff-scores {
  font-size: 11px;
  color: var(--muted);
}

.diff-delta {
  text-align: right;
  font-size: 13px;
  font-weight: 600;
}

.diff-delta.positive {
  color: var(--brand);
}

.diff-delta.negative {
  color: var(--slate);
}

.highest {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr);
  gap: 12px;
  text-decoration: none;
  color: inherit;
  padding: 22px;
  border-radius: var(--radius-md);
  background: var(--surface-soft);
  transition: transform 280ms var(--ease-spring), box-shadow 280ms var(--ease-spring);
}

.highest:hover {
  text-decoration: none;
  transform: translateY(-2px);
  box-shadow: var(--shadow-card);
}

.crown {
  color: var(--score-gold);
  margin-top: 5px;
}

.highest-text {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.highest-name {
  font: 700 20px/1.3 var(--font-display);
  letter-spacing: -0.02em;
  overflow-wrap: anywhere;
}

.highest-sub {
  font-size: 11px;
  color: var(--muted);
  overflow-wrap: anywhere;
}

.highest-score {
  grid-column: 2;
  font-size: 44px;
  font-weight: 700;
  line-height: 1.05;
  letter-spacing: -0.03em;
  color: var(--score-gold);
}

.footnote {
  margin: 24px 0 0;
  font-size: 11px;
  line-height: 1.9;
  color: var(--muted);
  max-width: 920px;
}

.state-line.error {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--danger);
  font-size: 13px;
}

.retry {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
  font-weight: 500;
  color: var(--brand);
  background: var(--fill);
  border-radius: var(--radius-sm);
  padding: 6px 12px;
  transition: background var(--motion-fast) var(--ease-snap);
}

.retry:hover {
  background: var(--fill-strong);
}

@media (max-width: 1050px) {
  .stat {
    padding: 0 12px;
  }

  .stat-value {
    font-size: 27px;
  }

  .panel {
    padding: 18px;
  }

  .bins {
    grid-template-columns: 1fr;
    gap: 9px;
  }

  .two-col {
    grid-template-columns: minmax(0, 1fr);
    gap: 20px;
  }

  .highest {
    grid-template-columns: 20px minmax(0, 1fr) auto;
    align-items: center;
  }

  .highest-score {
    grid-column: 3;
    grid-row: 1;
  }
}
</style>
