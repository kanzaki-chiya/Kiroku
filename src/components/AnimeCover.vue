<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { BangumiSubject } from '../types/anime'

const props = defineProps<{
  subject: BangumiSubject
}>()

const failed = ref(false)
const initial = computed(
  () => props.subject.nameCn.trim().charAt(0) || props.subject.name.trim().charAt(0) || '?'
)

watch(
  () => props.subject.coverUrl,
  () => {
    failed.value = false
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
  background: var(--sage-tint);
  container-type: inline-size;
}

.cover-img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: inherit;
}

.cover-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: clamp(5px, 7cqw, 20px);
  padding: clamp(6px, 8cqw, 16px);
  background: var(--slate-soft);
}

.cover-fallback::before {
  content: '';
  position: absolute;
  inset: 10px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-xs);
  pointer-events: none;
}

.cover-fallback::after {
  content: '';
  position: absolute;
  inset: 16px auto 16px 16px;
  width: 3px;
  border-left: 1px solid var(--border-strong);
  border-right: 1px solid var(--border-strong);
}

.fallback-initial {
  color: var(--brand-deep);
  font-family: var(--font-display);
  font-size: clamp(20px, 25cqw, 48px);
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
