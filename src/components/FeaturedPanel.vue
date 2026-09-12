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
  height: 136px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  overflow: hidden;
  text-decoration: none;
  color: inherit;
  box-shadow: var(--shadow-card);
  transition: border-color var(--motion-fast), box-shadow var(--motion-fast);
}

.featured:hover {
  text-decoration: none;
  border-color: var(--border-strong);
  box-shadow: var(--shadow-lift);
}

.featured-text {
  position: relative;
  flex: 1;
  min-width: 0;
  padding: 18px 24px 18px 28px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  grid-template-rows: auto auto 1fr;
  align-items: center;
  gap: 3px 18px;
  border-left: 3px solid var(--brand);
}

.caption {
  margin: 0;
  font-size: 10px;
  letter-spacing: .12em;
  color: var(--brand-deep);
  font-weight: 600;
  grid-column: 1;
}

.featured-title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 23px;
  line-height: 1.4;
  font-weight: 600;
  color: var(--text);
  grid-column: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.featured-meta {
  margin: 0;
  font-size: 11px;
  color: var(--muted);
  grid-column: 2;
  grid-row: 1;
}

.featured-excerpt {
  margin: 4px 0 0;
  font-size: 12px;
  line-height: 1.7;
  color: var(--text-soft);
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  grid-column: 1;
  grid-row: 3;
}

.featured-link {
  font-size: 11px;
  font-weight: 600;
  color: var(--brand-deep);
  grid-column: 2;
  grid-row: 3;
  align-self: end;
  justify-self: end;
}

.featured-art {
  width: 164px;
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
  background: var(--slate-soft);
}

.featured-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center 28%;
  display: block;
}

@media (max-width: 1050px) {
  .featured {
    height: 152px;
  }

  .featured-text {
    padding: 16px 20px;
    column-gap: 10px;
    grid-template-rows: auto auto auto 1fr;
  }

  .featured-title {
    font-size: 21px;
    grid-column: 1 / 3;
  }

  .featured-meta {
    grid-column: 1 / 3;
    grid-row: 3;
  }

  .featured-excerpt {
    grid-column: 1 / 3;
    grid-row: 4;
    -webkit-line-clamp: 1;
  }

  .featured-art {
    width: 136px;
  }

  .featured-link {
    grid-row: 1;
    align-self: center;
  }
}
</style>
