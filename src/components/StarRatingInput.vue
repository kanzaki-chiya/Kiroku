<script setup lang="ts">
import { computed, ref } from 'vue'
import { Star } from 'lucide-vue-next'
import { dimensionBand, starFill } from '../utils/format'

const props = defineProps<{
  modelValue: number | null
  label?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: number | null]
}>()

const hover = ref<number | null>(null)
const stars = [1, 2, 3, 4, 5] as const

const displayed = computed(() => hover.value ?? props.modelValue)

const valueText = computed(() => {
  if (props.modelValue === null) return '未评分'
  return `${props.modelValue}星 ${dimensionBand(props.modelValue)}`
})

function select(value: number) {
  emit('update:modelValue', props.modelValue === value ? null : value)
}

function nudge(delta: number) {
  const current = props.modelValue
  if (current === null) {
    if (delta > 0) emit('update:modelValue', 0.5)
    return
  }
  const next = Math.round((current + delta) * 2) / 2
  if (next < 0.5 || next > 5) return
  emit('update:modelValue', next)
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowRight') {
    event.preventDefault()
    nudge(0.5)
  } else if (event.key === 'ArrowLeft') {
    event.preventDefault()
    nudge(-0.5)
  } else if (event.key === 'Delete' || event.key === 'Backspace') {
    event.preventDefault()
    emit('update:modelValue', null)
  }
}

function onHitClick(value: number, event: MouseEvent) {
  const root = (event.currentTarget as HTMLElement).closest('.stars')
  if (root instanceof HTMLElement) root.focus()
  select(value)
}
</script>

<template>
  <div
    class="stars"
    role="slider"
    tabindex="0"
    :aria-label="label"
    aria-valuemin="0.5"
    aria-valuemax="5"
    :aria-valuenow="modelValue === null ? undefined : modelValue"
    :aria-valuetext="valueText"
    @keydown="onKeydown"
    @mouseleave="hover = null"
  >
    <span v-for="n in stars" :key="n" class="star-unit">
      <span class="star-visual" :data-fill="starFill(displayed, n)" aria-hidden="true">
        <Star class="star-icon star-empty" :size="22" />
        <span class="star-clip">
          <Star class="star-icon star-filled" :size="22" />
        </span>
      </span>
      <span
        class="star-hit left"
        :data-star-value="n - 0.5"
        @mouseenter="hover = n - 0.5"
        @click="onHitClick(n - 0.5, $event)"
      />
      <span
        class="star-hit right"
        :data-star-value="n"
        @mouseenter="hover = n"
        @click="onHitClick(n, $event)"
      />
    </span>
  </div>
</template>

<style scoped>
.stars {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border-radius: var(--radius-sm);
}

.stars:focus-visible {
  outline: 2px solid var(--brand);
  outline-offset: 3px;
  box-shadow: var(--focus-ring);
}

.star-unit {
  position: relative;
  width: 28px;
  height: 32px;
  flex-shrink: 0;
}

.star-visual {
  display: block;
  position: relative;
  width: 22px;
  height: 22px;
  margin: 5px 3px;
  transition: transform 160ms var(--ease-spring);
}

.star-unit:hover .star-visual {
  transform: scale(1.14);
}

.star-unit:active .star-visual {
  transform: scale(0.88);
}

.star-icon {
  display: block;
}

.star-empty {
  color: var(--border-strong);
}

.star-filled {
  color: var(--brand);
  fill: var(--brand);
}

.star-clip {
  position: absolute;
  inset: 0 auto 0 0;
  width: 0;
  overflow: hidden;
  transition: width 180ms var(--ease-snap);
}

.star-visual[data-fill='half'] .star-clip {
  width: 50%;
}

.star-visual[data-fill='full'] .star-clip {
  width: 100%;
}

.star-hit {
  position: absolute;
  top: 0;
  width: 50%;
  height: 100%;
  z-index: 1;
  cursor: pointer;
}

.star-hit.left {
  left: 0;
}

.star-hit.right {
  right: 0;
}
</style>
