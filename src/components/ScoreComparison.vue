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
  gap: 22px;
  flex-wrap: wrap;
}

.score {
  display: flex;
  flex-direction: column;
  gap: 9px;
  min-width: 0;
}

.score-label {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  color: var(--muted);
}

.score-label .star {
  color: var(--brand-deep);
  fill: var(--brand-deep);
}

.mock {
  font-size: 9px;
  padding: 0 5px;
  border: 1px solid var(--border);
  border-radius: var(--radius-xs);
  background: var(--slate-soft);
  color: var(--slate);
}

.score-value {
  font: 700 38px/1 var(--font-number);
  letter-spacing: -0.05em;
}

.mine .score-value {
  color: var(--score-gold);
}

.community .score-value {
  color: var(--slate);
}

.delta {
  margin: 0 0 0 auto;
  font-size: 11px;
  font-weight: 600;
  padding: 6px 10px;
  border-radius: var(--radius-xs);
  background: var(--sage-tint);
  color: var(--text-soft);
}

.delta.positive {
  background: var(--brand-soft);
  color: var(--brand-deep);
}

.delta.negative {
  background: var(--slate-soft);
  color: var(--slate);
}

@media (max-width: 1050px) {
  .compare {
    gap: 16px;
  }

  .delta {
    margin-left: 0;
  }

  .score-value {
    font-size: 32px;
  }
}
</style>
