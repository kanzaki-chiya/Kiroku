<script setup lang="ts">
import { computed } from 'vue'
import { Star } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import type { LibraryEntry } from '../types/anime'
import { formatLabels, formatScore, resolveTier, statusLabels, tierBadgeStyle } from '../utils/format'
import AnimeCover from './AnimeCover.vue'

const props = defineProps<{
  entry: LibraryEntry
  layout: 'grid' | 'list'
}>()

const store = useLibraryStore()
const tierStyle = computed(() =>
  tierBadgeStyle(resolveTier(props.entry.personal.tier, store.tiers)?.color)
)
</script>

<template>
  <RouterLink
    :to="`/anime/${entry.subject.id}`"
    class="card"
    :class="`is-${layout}`"
    :aria-label="`${entry.subject.nameCn}，${entry.subject.year} 年，我的评分 ${formatScore(entry.personal.score)}`"
  >
    <div class="cover-wrap">
      <AnimeCover :subject="entry.subject" />
      <span class="tier-badge card-tier" :class="{ 'tier-none': !entry.personal.tier }" :style="tierStyle">
        {{ entry.personal.tier ?? '—' }}
      </span>
      <span class="status-chip">{{ statusLabels[entry.personal.status] }}</span>
    </div>
    <div class="card-body">
      <h3 class="card-title">{{ entry.subject.nameCn }}</h3>
      <p class="card-meta">{{ entry.subject.year }} · {{ formatLabels[entry.subject.format] }}</p>
      <div class="ratings">
        <span class="rating mine" :title="`我的评分 ${formatScore(entry.personal.score)}`">
          <Star :size="12" class="star" aria-hidden="true" />
          <span class="rating-label">我的</span>
          <span class="personal-number">{{ formatScore(entry.personal.score) }}</span>
        </span>
        <span class="rating bgm" :title="`Bangumi 社区评分（Mock）${formatScore(entry.subject.community.score)}`">
          <span class="rating-label">社区</span>
          {{ formatScore(entry.subject.community.score) }}
        </span>
      </div>
    </div>
  </RouterLink>
</template>

<style scoped>
.card {
  display: flex;
  min-width: 0;
  text-decoration: none;
  color: inherit;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-card);
  transition: transform var(--motion-fast), box-shadow var(--motion-fast), border-color var(--motion-fast);
}

.card:hover {
  text-decoration: none;
  transform: translateY(-3px);
  border-color: var(--border-strong);
  box-shadow: var(--shadow-lift);
}

.cover-wrap {
  position: relative;
  min-width: 0;
  flex-shrink: 0;
}

.cover-wrap :deep(.cover) {
  border-radius: var(--radius-sm);
}

.card-tier {
  position: absolute;
  top: 10px;
  left: -5px;
  min-width: 30px;
  min-height: 29px;
  border-radius: 0 var(--radius-xs) var(--radius-xs) 0;
  box-shadow: var(--shadow-card);
  max-width: calc(100% - 10px);
}

.status-chip {
  position: absolute;
  bottom: 8px;
  right: 8px;
  padding: 3px 8px;
  font-size: 10px;
  border-radius: var(--radius-xs);
  background: var(--cover-overlay);
  color: var(--on-brand);
}

.card-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
  line-height: 1.6;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-meta {
  margin: 3px 0 0;
  font-size: 11px;
  color: var(--muted);
}

.ratings {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  margin-top: 12px;
  padding-top: 9px;
  border-top: 1px solid var(--border);
}

.rating {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  font-family: var(--font-number);
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
}

.rating-label {
  font-family: var(--font-body);
  font-size: 11px;
  font-weight: 400;
  color: var(--muted);
}

.rating.mine {
  color: var(--text-soft);
}

.rating.mine .star {
  align-self: center;
  width: 11px;
  color: var(--brand-deep);
  fill: var(--brand-deep);
}

.personal-number {
  font-size: 22px;
  font-weight: 700;
  line-height: 1;
  color: var(--score-gold);
}

.rating.bgm {
  color: var(--slate);
}

.is-grid {
  flex-direction: column;
  padding: 6px;
}

.is-grid .cover-wrap {
  width: 100%;
}

.is-grid .card-body {
  min-width: 0;
  padding: 10px 6px 7px;
}

.is-list {
  flex-direction: row;
  align-items: center;
  gap: 18px;
  padding: 10px;
}

.is-list .cover-wrap {
  width: 58px;
}

.is-list .card-tier {
  top: 4px;
  min-width: 22px;
  min-height: 22px;
  padding: 2px 5px;
  font-size: 10px;
}

.is-list .status-chip {
  bottom: 3px;
  right: 3px;
  padding: 2px 4px;
  font-size: 9px;
}

.is-list .card-body {
  flex: 1;
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto 155px;
  align-items: center;
  gap: 24px;
  padding-right: 10px;
}

.is-list .card-meta {
  margin: 0;
}

.is-list .ratings {
  margin: 0;
  padding: 0 0 0 20px;
  border-top: none;
  border-left: 1px solid var(--border);
}

@media (max-width: 1050px) {
  .is-list .card-body {
    grid-template-columns: minmax(0, 1fr) 145px;
    gap: 4px 16px;
  }

  .is-list .card-meta {
    grid-column: 1;
    grid-row: 2;
  }

  .is-list .ratings {
    grid-column: 2;
    grid-row: 1 / 3;
    padding-left: 12px;
  }
}
</style>
