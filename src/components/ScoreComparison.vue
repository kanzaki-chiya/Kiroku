<script setup lang="ts">
import { computed } from 'vue'
import { Star } from 'lucide-vue-next'
import { formatDelta, formatScore } from '../utils/format'

const props = defineProps<{
  personal: number | null
  community: number | null
}>()

const delta = computed(() =>
  props.personal !== null && props.community !== null ? props.personal - props.community : null
)

const deltaText = computed(() => {
  if (delta.value === null) return null
  if (Math.abs(delta.value) < 0.05) return '与社区基本一致'
  return delta.value > 0 ? `我更喜欢 ${formatDelta(delta.value)}` : `我更苛刻 ${formatDelta(delta.value)}`
})
</script>

<template>
  <div class="compare">
    <div class="score mine">
      <span class="score-label">
        <Star :size="13" class="star" aria-hidden="true" />我的评分
      </span>
      <span class="score-value">{{ formatScore(personal) }}</span>
    </div>
    <div class="score community">
      <span class="score-label">Bangumi 社区<span class="mock">Mock</span></span>
      <span class="score-value">{{ formatScore(community) }}</span>
    </div>
    <p v-if="deltaText" class="delta" :class="{ positive: delta! > 0, negative: delta! < 0 }">
      {{ deltaText }}
    </p>
  </div>
</template>

<style scoped>
.compare {
  display: flex;
  align-items: center;
  gap: 26px;
  flex-wrap: wrap;
}

.score {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.score-label {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12.5px;
  color: var(--muted);
}

.score-label .star {
  color: var(--accent);
  fill: var(--accent);
}

.mock {
  font-size: 10.5px;
  padding: 0 6px;
  margin-left: 4px;
  border-radius: 999px;
  background: var(--slate-soft);
  color: var(--slate);
}

.score-value {
  font-size: 26px;
  font-weight: 700;
  line-height: 1.1;
  font-variant-numeric: tabular-nums;
}

.mine .score-value {
  color: var(--accent);
}

.community .score-value {
  color: var(--slate);
}

.delta {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: 999px;
  background: var(--sage-tint);
  color: var(--text-soft);
}

.delta.positive {
  background: var(--accent-soft);
  color: var(--accent);
}

.delta.negative {
  background: var(--slate-soft);
  color: var(--slate);
}
</style>
