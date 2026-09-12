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
  font-size: 12px;
  font-weight: 500;
  color: var(--muted);
}

.score-label .star {
  color: var(--score-gold);
  fill: var(--score-gold);
}

.mock {
  font-size: 9px;
  padding: 1px 6px;
  border-radius: var(--radius-pill);
  background: var(--fill);
  color: var(--muted);
}

.score-value {
  font-size: 40px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.03em;
}

.mine .score-value {
  color: var(--score-gold);
}

.community .score-value {
  color: var(--slate);
}

.delta {
  margin: 0 0 0 auto;
  font-size: 12px;
  font-weight: 600;
  padding: 6px 12px;
  border-radius: var(--radius-pill);
  background: var(--fill);
  color: var(--text-soft);
}

.delta.positive {
  background: var(--brand-soft);
  color: var(--brand);
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
