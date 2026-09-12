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
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--sage-tint);
}

.cover-img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-fallback {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 14px;
  background:
    radial-gradient(circle at 30% 25%, rgba(255, 255, 255, 0.5), transparent 55%),
    linear-gradient(160deg, #dfe7df 0%, #c9d6cc 55%, #b4c4bb 100%);
}

.fallback-initial {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.75);
  color: var(--brand-deep);
  font-family: var(--font-logo);
  font-size: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.fallback-title {
  font-size: 12.5px;
  color: var(--text-soft);
  text-align: center;
  line-height: 1.4;
}
</style>
