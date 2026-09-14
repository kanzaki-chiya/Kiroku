<script lang="ts">
const loadedCoverUrls = new Set<string>()
</script>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { BangumiSubject } from '../types/anime'

const props = defineProps<{
  subject: BangumiSubject
}>()

const failed = ref(false)
const loaded = ref(!!props.subject.coverUrl && loadedCoverUrls.has(props.subject.coverUrl))
const initial = computed(
  () => props.subject.nameCn.trim().charAt(0) || props.subject.name.trim().charAt(0) || '?'
)

function onImgLoad() {
  loaded.value = true
  if (props.subject.coverUrl) loadedCoverUrls.add(props.subject.coverUrl)
}

watch(
  () => props.subject.coverUrl,
  url => {
    failed.value = false
    loaded.value = !!url && loadedCoverUrls.has(url)
  }
)
</script>

<template>
  <div class="cover" role="img" :aria-label="`${subject.nameCn} 封面`">
    <img
      v-if="subject.coverUrl && !failed"
      class="cover-img"
      :src="subject.coverUrl"
      :alt="subject.nameCn"
      width="200"
      height="300"
      loading="lazy"
      :class="{ 'is-loaded': loaded }"
      @load="onImgLoad"
      @error="failed = true"
    />
    <div v-else class="cover-fallback" aria-hidden="true">
      <span class="fallback-initial">{{ initial }}</span>
      <span class="fallback-title">{{ subject.nameCn }}</span>
    </div>
  </div>
</template>

<style scoped>
.cover {
  position: relative;
  width: 100%;
  aspect-ratio: 2 / 3;
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--fill);
  container-type: inline-size;
}

.cover-img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: inherit;
  opacity: 0;
  transform: scale(1.015);
  transition: opacity var(--motion-med) var(--ease-snap),
    transform 380ms var(--ease-snap);
}

.cover-img.is-loaded {
  opacity: 1;
  transform: none;
}

.cover-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: clamp(5px, 7cqw, 16px);
  padding: clamp(6px, 8cqw, 16px);
  background: var(--cover-fallback);
}

.fallback-initial {
  color: var(--muted);
  font-family: var(--font-display);
  font-weight: 700;
  letter-spacing: -0.02em;
  font-size: clamp(20px, 26cqw, 46px);
  line-height: 1;
}

.fallback-title {
  font-size: clamp(9px, 7cqw, 11px);
  color: var(--text-soft);
  text-align: center;
  line-height: 1.7;
  max-width: 100%;
  overflow-wrap: anywhere;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
