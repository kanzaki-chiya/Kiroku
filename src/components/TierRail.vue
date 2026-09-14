<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import type { TierDefinition } from '../types/anime'
import { tierBadgeStyle } from '../utils/format'

const props = defineProps<{
  tiers: TierDefinition[]
  /** '' 表示未分档 */
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const root = ref<HTMLElement | null>(null)

const options = computed(() => [
  { key: '', badge: '–', desc: '未分档', color: '', none: true },
  ...props.tiers.map(t => ({ key: t.name, badge: t.name, desc: t.description, color: t.color, none: false }))
])

const activeIndex = computed(() => {
  const index = options.value.findIndex(o => o.key === props.modelValue)
  return index < 0 ? 0 : index
})

async function go(index: number) {
  const count = options.value.length
  const next = ((index % count) + count) % count
  emit('update:modelValue', options.value[next].key)
  await nextTick()
  root.value?.querySelectorAll<HTMLButtonElement>('.tier-chip')[next]?.focus()
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
    void go(options.value.length - 1)
  }
}
</script>

<template>
  <div ref="root" class="tier-rail" role="radiogroup" aria-label="分档" @keydown="onKeydown">
    <button
      v-for="(opt, i) in options"
      :key="opt.key || '__none'"
      type="button"
      class="tier-chip"
      role="radio"
      :data-tier="opt.key"
      :aria-checked="modelValue === opt.key"
      :tabindex="i === activeIndex ? 0 : -1"
      :style="opt.color ? { '--chip-color': opt.color } : undefined"
      @click="emit('update:modelValue', opt.key)"
    >
      <span
        class="tier-badge"
        :class="{ 'tier-none': opt.none }"
        :style="tierBadgeStyle(opt.color || undefined)"
        aria-hidden="true"
      >{{ opt.badge }}</span>
      <span v-if="opt.desc" class="chip-desc">{{ opt.desc }}</span>
    </button>
  </div>
</template>

<style scoped>
.tier-rail {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tier-chip {
  --chip-color: var(--muted);
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 36px;
  padding: 5px 14px 5px 6px;
  border-radius: var(--radius-pill);
  background: var(--fill);
  transition: background var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap),
    transform 120ms ease-out;
}

.tier-chip:active {
  transform: scale(0.95);
}

.tier-chip:hover {
  background: var(--fill-strong);
}

.tier-chip[aria-checked='true'] {
  background: var(--fill-strong);
  box-shadow: inset 0 0 0 1.5px var(--chip-color);
}

@supports (background: color-mix(in srgb, red 10%, transparent)) {
  .tier-chip[aria-checked='true'] {
    background: color-mix(in srgb, var(--chip-color) 13%, transparent);
  }
}

.tier-chip:focus-visible {
  outline: 2px solid var(--brand);
  outline-offset: 2px;
}

.tier-chip .tier-badge {
  min-width: 24px;
  min-height: 24px;
  padding: 1px 6px;
  font-size: 11px;
}

@keyframes chip-pop {
  0% {
    transform: scale(0.82);
  }

  55% {
    transform: scale(1.1);
  }

  100% {
    transform: scale(1);
  }
}

.tier-chip[aria-checked='true'] .tier-badge {
  animation: chip-pop 320ms var(--ease-spring);
}

.chip-desc {
  font-size: 12.5px;
  color: var(--text-soft);
  white-space: nowrap;
}

.tier-chip[aria-checked='true'] .chip-desc {
  color: var(--text);
  font-weight: 500;
}
</style>
