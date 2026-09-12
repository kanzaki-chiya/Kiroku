<script setup lang="ts">
import { computed } from 'vue'
import type { LibraryEntry } from '../types/anime'
import { formatLabels, statusLabels } from '../utils/format'

const props = defineProps<{
  entry: LibraryEntry
}>()

const excerpt = computed(
  () => props.entry.personal.review.trim() || props.entry.subject.summary
)
</script>

<template>
  <RouterLink :to="`/anime/${entry.subject.id}`" class="featured" :aria-label="`最近记下的一部：${entry.subject.nameCn}`">
    <div class="featured-text">
      <p class="caption">最近记下的一部</p>
      <h2 class="featured-title">{{ entry.subject.nameCn }}</h2>
      <p class="featured-meta">
        {{ entry.subject.year }} · {{ formatLabels[entry.subject.format] }} ·
        {{ statusLabels[entry.personal.status] }}
      </p>
      <p class="featured-excerpt">{{ excerpt }}</p>
      <span class="featured-link">查看详情 →</span>
    </div>
    <div class="featured-art" aria-hidden="true">
      <img :src="entry.subject.coverUrl" :alt="''" />
    </div>
  </RouterLink>
</template>

<style scoped>
.featured {
  display: flex;
  align-items: stretch;
  max-height: 210px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  overflow: hidden;
  text-decoration: none;
  color: inherit;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.featured:hover {
  text-decoration: none;
  border-color: var(--border-strong);
  box-shadow: var(--shadow-card);
}

.featured-text {
  flex: 1;
  min-width: 0;
  padding: 22px 26px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.caption {
  margin: 0 0 6px;
  font-size: 12px;
  letter-spacing: 0.08em;
  color: var(--accent);
  font-weight: 600;
}

.featured-title {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: var(--text);
}

.featured-meta {
  margin: 4px 0 10px;
  font-size: 13px;
  color: var(--muted);
}

.featured-excerpt {
  margin: 0;
  font-size: 13.5px;
  line-height: 1.7;
  color: var(--text-soft);
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.featured-link {
  margin-top: 10px;
  font-size: 13px;
  font-weight: 600;
  color: var(--brand);
}

.featured-art {
  width: 320px;
  flex-shrink: 0;
  position: relative;
}

.featured-art::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(to right, rgba(255, 255, 255, 0.55), transparent 45%);
  z-index: 1;
}

.featured-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center 22%;
  display: block;
}

@media (max-width: 900px) {
  .featured-art {
    width: 190px;
  }
}

@media (max-width: 640px) {
  .featured-art {
    display: none;
  }
}
</style>
