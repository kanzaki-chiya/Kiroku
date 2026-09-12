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
          {{ formatScore(entry.personal.score) }}
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
  text-decoration: none;
  color: inherit;
  border-radius: var(--radius-md);
}

.card:hover {
  text-decoration: none;
}

.cover-wrap {
  position: relative;
  flex-shrink: 0;
  transition: transform 0.18s ease, box-shadow 0.18s ease;
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-card);
}

.card:hover .cover-wrap,
.card:focus-visible .cover-wrap {
  transform: translateY(-3px);
  box-shadow: var(--shadow-lift);
}

.card:focus-visible {
  outline: 2px solid var(--brand);
  outline-offset: 3px;
}

.card-tier {
  position: absolute;
  top: 7px;
  left: 7px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.28);
}

.status-chip {
  position: absolute;
  bottom: 7px;
  right: 7px;
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 999px;
  background: rgba(37, 53, 47, 0.72);
  color: #eef3ef;
  backdrop-filter: blur(2px);
}

.card-title {
  margin: 0;
  font-size: 14.5px;
  font-weight: 600;
  color: var(--text);
  line-height: 1.4;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
}

.card-meta {
  margin: 2px 0 0;
  font-size: 12.5px;
  color: var(--muted);
}

.ratings {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 6px;
}

.rating {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  font-weight: 600;
}

.rating-label {
  font-size: 11.5px;
  font-weight: 500;
  color: var(--muted);
}

.rating.mine {
  color: var(--accent);
}

.rating.mine .star {
  fill: var(--accent);
}

.rating.bgm {
  color: var(--slate);
}

.is-grid {
  flex-direction: column;
}

.is-grid .cover-wrap {
  width: 100%;
}

.is-grid .card-body {
  padding: 9px 2px 0;
}

.is-list {
  flex-direction: row;
  align-items: center;
  gap: 16px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  background: var(--surface);
  border-radius: var(--radius-md);
}

.is-list:hover {
  border-color: var(--border-strong);
}

.is-list .cover-wrap {
  width: 64px;
}

.is-list .cover-wrap :deep(.cover) {
  border-radius: var(--radius-sm);
}

.is-list .card-body {
  flex: 1;
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) auto auto;
  align-items: center;
  gap: 18px;
}

.is-list .card-title {
  font-size: 15px;
}

.is-list .card-meta {
  margin: 0;
}

.is-list .ratings {
  margin: 0;
}

@media (max-width: 700px) {
  .is-list .card-body {
    grid-template-columns: minmax(0, 1fr);
    gap: 3px;
  }

  .ratings {
    gap: 8px;
    flex-wrap: wrap;
  }

  .rating {
    white-space: nowrap;
  }
}
</style>
