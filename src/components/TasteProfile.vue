<script setup lang="ts">
import { computed, useId } from 'vue'
import { dimensionKeys } from '../types/anime'
import { dimensionLabels } from '../utils/format'
import type { StatisticsDimension } from '../utils/statistics'

const props = defineProps<{
  dimensions: StatisticsDimension[]
  ready: boolean
}>()

const rated = computed(() => props.dimensions.filter(
  (dimension): dimension is StatisticsDimension & { mean: number } => dimension.mean !== null
))
const summary = computed(() => {
  const values = rated.value
  if (!values.length) return ''
  if (values.length === 1) {
    const only = values[0]!
    return `目前仅评了${dimensionLabels[only.key]} · 均分 ${only.mean.toFixed(1)} 星`
  }
  const highest = Math.max(...values.map(dimension => dimension.mean))
  const lowest = Math.min(...values.map(dimension => dimension.mean))
  if (highest === lowest) return `已评维度均分相同 · ${highest.toFixed(1)} 星`
  const highestLabels = values.filter(dimension => dimension.mean === highest).map(dimension => dimensionLabels[dimension.key]).join('、')
  const lowestLabels = values.filter(dimension => dimension.mean === lowest).map(dimension => dimensionLabels[dimension.key]).join('、')
  return `均分最高：${highestLabels} ${highest.toFixed(1)} 星 · 均分最低：${lowestLabels} ${lowest.toFixed(1)} 星`
})

const chartId = useId()
const center = { x: 230, y: 180 }
const radius = 112
function point(index: number, distance: number) {
  const angle = -Math.PI / 2 + index * Math.PI * 2 / dimensionKeys.length
  return {
    x: Number((center.x + Math.cos(angle) * distance).toFixed(3)),
    y: Number((center.y + Math.sin(angle) * distance).toFixed(3))
  }
}
const axes = dimensionKeys.map((key, index) => ({
  key,
  end: point(index, radius),
  label: point(index, 152),
  anchor: index === 0 ? 'middle' : index < 3 ? 'start' : 'end'
}))
const rings = [1, 2, 3, 4, 5].map(stars => ({
  stars,
  points: dimensionKeys.map((_, index) => {
    const p = point(index, radius * stars / 5)
    return `${p.x},${p.y}`
  }).join(' ')
}))
const chartDimensions = computed(() => axes.map((axis, index) => {
  const dimension = props.dimensions.find(item => item.key === axis.key)
  const mean = dimension?.mean ?? null
  return {
    ...axis,
    mean,
    count: dimension?.count ?? 0,
    value: point(index, radius * (mean ?? 0) / 5)
  }
}))
const polygonPoints = computed(() => chartDimensions.value.map(dimension =>
  `${dimension.value.x},${dimension.value.y}`
).join(' '))
const ratedPoints = computed(() => chartDimensions.value.filter(dimension => dimension.mean !== null))
const chartDescription = computed(() => chartDimensions.value.map(dimension =>
  `${dimensionLabels[dimension.key]}：${dimension.mean === null ? '未评' : `${dimension.mean.toFixed(1)} 星`}，已评 ${dimension.count} 部`
).join('；') + '。满分 5 星，未评维度在图中按 0 展示，不计入均分。')
</script>

<template>
  <section class="taste-profile" aria-labelledby="taste-profile-title">
    <div class="profile-head">
      <h2 id="taste-profile-title" class="panel-title">口味画像</h2>
      <span class="profile-scale">五维均分 · 满分 5 星</span>
    </div>
    <p v-if="rated.length" class="profile-summary">{{ summary }}</p>
    <p v-else class="profile-empty">还没有分项评分。在作品详情中为剧情、角色、演出、作画或音乐打星，看看你的口味画像。</p>
    <svg class="radar" viewBox="0 0 460 380" role="img" :aria-labelledby="`${chartId}-title ${chartId}-desc`">
      <title :id="`${chartId}-title`">五维分项评分雷达图</title>
      <desc :id="`${chartId}-desc`">{{ chartDescription }}</desc>
      <g aria-hidden="true">
        <polygon v-for="ring in rings" :key="ring.stars" class="radar-ring" :points="ring.points" />
        <line v-for="axis in axes" :key="axis.key" class="radar-axis" :x1="center.x" :y1="center.y" :x2="axis.end.x" :y2="axis.end.y" />
        <text v-for="ring in rings" :key="ring.stars" class="radar-tick" :x="center.x + 7" :y="center.y - radius * ring.stars / 5 + 12">{{ ring.stars }}</text>
        <g v-if="ratedPoints.length" class="radar-values" :class="{ 'is-ready': ready }">
          <polygon class="radar-area" :points="polygonPoints" />
          <circle v-for="dimension in ratedPoints" :key="dimension.key" class="radar-point" :data-dimension="dimension.key" :cx="dimension.value.x" :cy="dimension.value.y" r="3.5" />
        </g>
        <text v-for="dimension in chartDimensions" :key="dimension.key" class="radar-label" :x="dimension.label.x" :y="dimension.label.y" :text-anchor="dimension.anchor" :data-dimension="dimension.key">
          <tspan class="dimension-label" :x="dimension.label.x">{{ dimensionLabels[dimension.key] }}</tspan>
          <tspan class="dimension-value" :x="dimension.label.x" dy="18">{{ dimension.mean === null ? '未评' : `${dimension.mean.toFixed(1)} 星` }}</tspan>
          <tspan class="dimension-count" :x="dimension.label.x" dy="16">已评 {{ dimension.count }} 部</tspan>
        </text>
      </g>
    </svg>
  </section>
</template>

<style scoped>
.taste-profile {
  background: var(--surface);
  border-radius: var(--radius-lg);
  padding: 26px;
  margin-bottom: 22px;
  box-shadow: var(--shadow-card);
  min-width: 0;
}

.profile-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 12px;
}

.panel-title {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.profile-scale {
  color: var(--muted);
  font-size: 12px;
}

.profile-summary,
.profile-empty {
  margin: 0 0 24px;
  font-size: 13px;
  line-height: 1.7;
  overflow-wrap: anywhere;
}

.profile-summary {
  color: var(--text-soft);
}

.profile-empty {
  color: var(--muted);
}

.radar {
  display: block;
  width: 100%;
  max-width: 540px;
  height: auto;
  margin: 0 auto;
  overflow: visible;
}

.radar-ring {
  fill: none;
  stroke: var(--border-strong);
  stroke-width: 1;
}

.radar-axis {
  stroke: var(--border);
  stroke-width: 1;
}

.radar-tick {
  fill: var(--muted);
  font-size: 10px;
}

.radar-values {
  transform: scale(0);
  transform-origin: 230px 180px;
  transition: transform 500ms var(--ease-spring);
}

.radar-values.is-ready {
  transform: scale(1);
}

.radar-area {
  fill: var(--brand-soft);
  stroke: var(--brand);
  stroke-width: 2;
  stroke-linejoin: round;
}

.radar-point {
  fill: var(--brand);
  stroke: var(--surface);
  stroke-width: 1.5;
}

.dimension-label {
  fill: var(--text);
  font-size: 14px;
  font-weight: 500;
}

.dimension-value {
  fill: var(--text-soft);
  font-size: 13px;
  font-variant-numeric: tabular-nums;
}

.dimension-count {
  fill: var(--muted);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

@media (max-width: 1050px) {
  .taste-profile {
    padding: 18px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .radar-values {
    transform: scale(1);
    transition: none;
  }
}
</style>
