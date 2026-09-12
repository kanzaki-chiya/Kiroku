<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'
import { Crown } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import { formatDelta, formatScore } from '../utils/format'
import type { LibraryStatistics } from '../utils/statistics'

const store = useLibraryStore()
const stats = shallowRef<LibraryStatistics | null>(null)

watch(
  () => store.count,
  async () => {
    stats.value = await store.getStatistics()
  },
  { immediate: true }
)

const maxBin = computed(() => Math.max(1, ...(stats.value?.bins.map(b => b.count) ?? [0])))

function fmt(value: number | null): string {
  return value === null ? '—' : value.toFixed(1)
}
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
        <span class="stat-value">{{ stats.total }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">已评分</span>
        <span class="stat-value">{{ stats.ratedCount }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">我的均分</span>
        <span class="stat-value mine">{{ fmt(stats.personalMean) }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Bangumi 均分<span class="mock">Mock</span></span>
        <span class="stat-value community">{{ fmt(stats.communityMean) }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">平均差值</span>
        <span class="stat-value" :class="{ mine: (stats.meanDifference ?? 0) > 0, community: (stats.meanDifference ?? 0) < 0 }">
          {{ stats.meanDifference === null ? '—' : formatDelta(stats.meanDifference) }}
        </span>
      </div>
    </div>

    <section class="panel">
      <h2 class="panel-title">我的评分分布</h2>
      <ul class="bins" aria-label="评分分布">
        <li v-for="bin in stats.bins" :key="bin.label" class="bin-row">
          <span class="bin-label">{{ bin.label }}</span>
          <div class="bin-track" role="img" :aria-label="`${bin.label} 分区间：${bin.count} 部`">
            <div class="bin-fill" :style="{ width: `${(bin.count / maxBin) * 100}%` }"></div>
          </div>
          <span class="bin-count">{{ bin.count }} 部</span>
        </li>
      </ul>
    </section>

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
          <span class="highest-score">{{ formatScore(stats.highest.personal.score) }}</span>
        </RouterLink>
      </section>
    </div>

    <p class="footnote">
      统计范围：当前番剧库全部 {{ stats.total }} 部。我的均分 = 所有已打总分作品（{{ stats.ratedCount }}
      部）的平均；Bangumi 社区均分 = 有社区评分的 {{ stats.communityCount }} 部的平均（Mock
      快照，未按票数加权）；平均差值 = 两者皆有评分的 {{ stats.pairedCount }} 部中「我的 −
      Bangumi 社区」的均值，正数代表我更喜欢。分项评分不参与总分计算。
    </p>
    </template>
  </div>
</template>

<style scoped>
.page-title {
  margin: 0;
  font-size: 32px;
  font-weight: 700;
}

.page-sub {
  margin: 6px 0 26px;
  font-size: 14px;
  color: var(--muted);
}

.stat-cards {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 14px;
  margin-bottom: 26px;
}

.stat {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 12.5px;
  color: var(--muted);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.mock {
  font-size: 10px;
  padding: 0 5px;
  border-radius: 999px;
  background: var(--slate-soft);
  color: var(--slate);
}

.stat-value {
  font-size: 26px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.stat-value.mine {
  color: var(--accent);
}

.stat-value.community {
  color: var(--slate);
}

.panel {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 20px 24px;
  margin-bottom: 22px;
}

.panel-title {
  margin: 0 0 16px;
  font-size: 16px;
  font-weight: 700;
}

.panel-empty {
  margin: 0;
  font-size: 13.5px;
  color: var(--muted);
}

.bins {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 620px;
}

.bin-row {
  display: flex;
  align-items: center;
  gap: 14px;
}

.bin-label {
  width: 52px;
  font-size: 13px;
  color: var(--text-soft);
  flex-shrink: 0;
}

.bin-track {
  flex: 1;
  height: 18px;
  border-radius: 5px;
  background: var(--sage-tint);
  overflow: hidden;
}

.bin-fill {
  height: 100%;
  border-radius: 5px;
  background: var(--brand);
  min-width: 0;
}

.bin-count {
  width: 48px;
  font-size: 13px;
  color: var(--muted);
  text-align: right;
  flex-shrink: 0;
}

.two-col {
  display: grid;
  grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
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
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
  text-decoration: none;
  color: inherit;
  font-size: 14px;
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
  flex: 1;
  min-width: 0;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.diff-scores {
  font-size: 12.5px;
  color: var(--muted);
  flex-shrink: 0;
}

.diff-delta {
  width: 52px;
  text-align: right;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.diff-delta.positive {
  color: var(--accent);
}

.diff-delta.negative {
  color: var(--slate);
}

.highest {
  display: flex;
  align-items: center;
  gap: 12px;
  text-decoration: none;
  color: inherit;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}

.highest:hover {
  text-decoration: none;
  border-color: var(--border-strong);
  box-shadow: var(--shadow-card);
}

.crown {
  color: var(--accent);
  flex-shrink: 0;
}

.highest-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.highest-name {
  font-size: 15px;
  font-weight: 600;
}

.highest-sub {
  font-size: 12px;
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.highest-score {
  font-size: 22px;
  font-weight: 700;
  color: var(--accent);
}

.footnote {
  margin: 26px 0 0;
  font-size: 12.5px;
  line-height: 1.8;
  color: var(--muted);
  max-width: 720px;
}

@media (max-width: 1100px) {
  .stat-cards {
    grid-template-columns: repeat(3, 1fr);
  }

  .two-col {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 520px) {
  .stat-cards {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .panel {
    padding: 18px 14px;
  }

  .diff-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 48px;
    gap: 4px 8px;
  }

  .diff-title {
    grid-column: 1;
  }

  .diff-scores {
    grid-column: 1;
    grid-row: 2;
    white-space: normal;
  }

  .diff-delta {
    grid-column: 2;
    grid-row: 1 / 3;
  }
}
</style>
