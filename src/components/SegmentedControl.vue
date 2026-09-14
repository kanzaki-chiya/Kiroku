<script setup lang="ts" generic="T extends string">
import { computed, nextTick, ref } from 'vue'

const props = defineProps<{
  options: { value: T; label: string }[]
  modelValue: T
  ariaLabel: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: T]
}>()

const root = ref<HTMLElement | null>(null)

const activeIndex = computed(() => {
  const index = props.options.findIndex(o => o.value === props.modelValue)
  return index < 0 ? 0 : index
})

async function go(index: number) {
  const count = props.options.length
  if (count === 0) return
  const next = ((index % count) + count) % count
  emit('update:modelValue', props.options[next].value)
  await nextTick()
  root.value?.querySelectorAll<HTMLButtonElement>('.seg-btn')[next]?.focus()
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
    event.preventDefault()
    void go(activeIndex.value + 1)
  } else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
    event.preventDefault()
    void go(activeIndex.value - 1)
  } else if (event.key === 'Home') {
    event.preventDefault()
    void go(0)
  } else if (event.key === 'End') {
    event.preventDefault()
    void go(props.options.length - 1)
  }
}
</script>

<template>
  <div
    ref="root"
    class="seg"
    role="radiogroup"
    :aria-label="ariaLabel"
    :style="{ '--seg-count': String(options.length) }"
    @keydown="onKeydown"
  >
    <span
      class="seg-thumb"
      :style="{ transform: `translateX(${activeIndex * 100}%)` }"
      aria-hidden="true"
    />
    <button
      v-for="(opt, i) in options"
      :key="opt.value"
      type="button"
      class="seg-btn"
      role="radio"
      :aria-checked="opt.value === modelValue"
      :tabindex="i === activeIndex ? 0 : -1"
      @click="emit('update:modelValue', opt.value)"
    >
      {{ opt.label }}
    </button>
  </div>
</template>

<style scoped>
.seg {
  position: relative;
  display: flex;
  width: 100%;
  max-width: 340px;
  padding: 2px;
  border-radius: 10px;
  background: var(--fill);
}

.seg-thumb {
  position: absolute;
  top: 2px;
  bottom: 2px;
  left: 2px;
  width: calc((100% - 4px) / var(--seg-count));
  border-radius: 8px;
  background: var(--surface);
  box-shadow: var(--shadow-thumb);
  transition: transform 260ms var(--ease-spring);
  pointer-events: none;
}

.seg-btn {
  position: relative;
  z-index: 1;
  flex: 1 1 0;
  min-width: 0;
  height: 30px;
  border-radius: 8px;
  font-size: 13px;
  color: var(--muted);
  white-space: nowrap;
  transition: color var(--motion-fast) var(--ease-snap),
    transform 120ms ease-out;
}

.seg-btn:active {
  transform: scale(0.94);
}

.seg-btn:hover {
  color: var(--text-soft);
}

.seg-btn[aria-checked='true'] {
  color: var(--text);
  font-weight: 600;
}

.seg-btn:focus-visible {
  outline: 2px solid var(--brand);
  outline-offset: 2px;
}
</style>
