<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { LibraryEntry } from '../types/anime'
import { formatLabels, statusLabels } from '../utils/format'

const props = defineProps<{
  entry: LibraryEntry
}>()

const excerpt = computed(
  () => props.entry.personal.review.trim() || props.entry.subject.summary
)

const imgLoaded = ref(false)
watch(
  () => props.entry.subject.coverUrl,
  () => {
    imgLoaded.value = false
  }
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
      <span class="featured-link">查看详情 ›</span>
    </div>
    <div class="featured-art" aria-hidden="true">
      <img :src="entry.subject.coverUrl" :alt="''" :class="{ 'is-loaded': imgLoaded }" @load="imgLoaded = true" />
    </div>
  </RouterLink>
</template>

<style scoped>
.featured {
  display: flex;
  align-items: stretch;
  height: 150px;
  border-radius: var(--radius-lg);
  background: var(--surface);
  overflow: hidden;
  text-decoration: none;
  color: inherit;
  box-shadow: var(--shadow-card);
  transition: transform 280ms var(--ease-spring), box-shadow 280ms var(--ease-spring);
}

.featured:hover {
  text-decoration: none;
  transform: translateY(-2px);
  box-shadow: var(--shadow-lift);
}

.featured:active {
  transform: scale(0.99);
  transition-duration: 100ms;
}

.featured-text {
  position: relative;
  flex: 1;
  min-width: 0;
  padding: 20px 24px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  grid-template-rows: auto auto 1fr;
  align-items: center;
  gap: 4px 18px;
}

.caption {
  margin: 0;
  font-size: 11px;
  letter-spacing: 0.04em;
  color: var(--brand);
  font-weight: 600;
  grid-column: 1;
}

.featured-title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 24px;
  line-height: 1.25;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text);
  grid-column: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.featured-meta {
  margin: 0;
  font-size: 12px;
  color: var(--muted);
  grid-column: 2;
  grid-row: 1;
}

.featured-excerpt {
  margin: 6px 0 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-soft);
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  grid-column: 1;
  grid-row: 3;
}

.featured-link {
  font-size: 12px;
  font-weight: 500;
  color: var(--brand);
  grid-column: 2;
  grid-row: 3;
  align-self: end;
  justify-self: end;
}

.featured-art {
  width: 190px;
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
  background: var(--fill);
  -webkit-mask-image: linear-gradient(to right, transparent, #000 22%);
  mask-image: linear-gradient(to right, transparent, #000 22%);
}

.featured-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center 28%;
  display: block;
  opacity: 0;
  transition: opacity var(--motion-med) var(--ease-snap),
    transform 500ms var(--ease-spring);
}

.featured-art img.is-loaded {
  opacity: 1;
}

.featured:hover .featured-art img {
  transform: scale(1.04);
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
    font-size: 20px;
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
