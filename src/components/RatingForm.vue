<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from 'vue'
import {
  dimensionKeys,
  type DimensionKey,
  type PersonalDraft,
  type Tier,
  type TierDefinition,
  type WatchStatus
} from '../types/anime'
import { cloneDraft, isValidDimension } from '../utils/draft'
import { dimensionBand, dimensionLabels, scoreBand, statusLabels } from '../utils/format'
import SegmentedControl from './SegmentedControl.vue'
import StarRatingInput from './StarRatingInput.vue'
import TierRail from './TierRail.vue'

const props = defineProps<{
  initial: PersonalDraft
  submitLabel: string
  tiers: TierDefinition[]
  busy?: boolean
}>()

const emit = defineEmits<{
  save: [draft: PersonalDraft]
  cancel: []
  dirtyChange: [dirty: boolean]
}>()

const scoreEnabled = ref(props.initial.score !== null)
const scoreInput = ref(props.initial.score === null ? '' : String(props.initial.score))
const tier = ref<Tier | ''>(props.initial.tier ?? '')
const status = ref<WatchStatus>(props.initial.status)
const review = ref(props.initial.review)

const dimValues = reactive<Record<DimensionKey, number | null>>({
  story: props.initial.dimensions.story,
  characters: props.initial.dimensions.characters,
  direction: props.initial.dimensions.direction,
  animation: props.initial.dimensions.animation,
  music: props.initial.dimensions.music
})

const error = ref('')
const submitting = ref(false)
const dragging = ref(false)

const initialSnapshot = JSON.stringify(cloneDraft(props.initial))

function parseLenient(raw: string | number): number | null {
  if (typeof raw === 'number') return Number.isFinite(raw) ? raw : null
  const trimmed = raw.trim()
  if (trimmed === '') return null
  const value = Number(trimmed)
  return Number.isFinite(value) ? value : null
}

const currentDraft = computed<PersonalDraft>(() => ({
  score: scoreEnabled.value ? parseLenient(scoreInput.value) : null,
  tier: tier.value === '' ? null : tier.value,
  status: status.value,
  dimensions: { ...dimValues },
  review: review.value
}))

const dirty = computed(() => JSON.stringify(currentDraft.value) !== initialSnapshot)

watch(dirty, value => emit('dirtyChange', value), { immediate: true })
watch(currentDraft, () => {
  error.value = ''
})
watch(
  () => props.busy,
  value => {
    if (!value) submitting.value = false
  }
)

const statusOptions: { value: WatchStatus; label: string }[] = [
  { value: 'planned', label: statusLabels.planned },
  { value: 'watching', label: statusLabels.watching },
  { value: 'completed', label: statusLabels.completed }
]

const fillPct = computed(() => (sliderValue(scoreInput.value) / 10) * 100)

const band = computed(() => {
  if (!scoreEnabled.value) return null
  const value = parseLenient(scoreInput.value)
  return value === null || value < 0 || value > 10 ? null : scoreBand(value)
})

function sliderValue(raw: string | number): number {
  const value = parseLenient(raw)
  if (value === null) return 0
  return Math.min(10, Math.max(0, value))
}

function validateScoreField(raw: string | number, label: string): number | string {
  const trimmed = typeof raw === 'number' ? String(raw) : raw.trim()
  if (trimmed === '') return `${label}不能为空，或标记为未评分`
  const value = Number(trimmed)
  if (!Number.isFinite(value)) return `${label}需要是数字`
  if (value < 0 || value > 10) return `${label}需要在 0–10 之间`
  return value
}

function onSubmit() {
  if (props.busy || submitting.value) return
  error.value = ''

  let score: number | null = null
  if (scoreEnabled.value) {
    const result = validateScoreField(scoreInput.value, '总分')
    if (typeof result === 'string') {
      error.value = result
      return
    }
    score = result
  }

  const dimensions = {} as Record<DimensionKey, number | null>
  for (const key of dimensionKeys) {
    const value = dimValues[key]
    if (value === null) {
      dimensions[key] = null
      continue
    }
    if (!isValidDimension(value)) {
      error.value = `${dimensionLabels[key]}需要是 0.5–5 的半星`
      return
    }
    dimensions[key] = value
  }

  submitting.value = true
  emit('save', {
    score,
    tier: tier.value === '' ? null : tier.value,
    status: status.value,
    dimensions,
    review: review.value
  })
  void nextTick(() => {
    if (!props.busy) submitting.value = false
  })
}

function toggleSkipScore() {
  scoreEnabled.value = !scoreEnabled.value
  if (scoreEnabled.value && String(scoreInput.value).trim() === '') {
    scoreInput.value = '7'
  }
}

function onCancel() {
  emit('cancel')
}
</script>

<template>
  <form class="rating-form" @submit.prevent="onSubmit">
    <div class="field-group">
      <div class="group-head">
        <span class="group-title">总分</span>
        <button
          type="button"
          class="skip-switch"
          role="switch"
          :aria-checked="!scoreEnabled"
          @click="toggleSkipScore"
        >
          <span class="skip-text">暂不评分</span>
          <span class="switch-track" aria-hidden="true"><span class="switch-knob" /></span>
        </button>
      </div>
      <div class="score-body" :class="{ off: !scoreEnabled }">
        <input
          v-model="scoreInput"
          class="score-number"
          type="number"
          min="0"
          max="10"
          step="0.1"
          inputmode="decimal"
          aria-label="总分（0–10）"
          placeholder="–"
          :disabled="!scoreEnabled"
        />
        <div class="score-slider">
          <input
            class="score-range"
            type="range"
            min="0"
            max="10"
            step="0.1"
            :value="sliderValue(scoreInput)"
            :disabled="!scoreEnabled"
            :class="{ dragging }"
            :style="{ '--pos': `${fillPct}%` }"
            @input="scoreInput = ($event.target as HTMLInputElement).value"
            @pointerdown="dragging = true"
            @pointerup="dragging = false"
            @pointercancel="dragging = false"
            aria-label="总分滑杆"
          />
          <div class="ticks" aria-hidden="true">
            <span v-for="i in 11" :key="i" />
          </div>
          <div class="scale" aria-hidden="true">
            <span class="lo">0</span>
            <span class="mid">5</span>
            <span class="hi">10</span>
          </div>
        </div>
        <span class="band-slot">
          <Transition name="band" mode="out-in">
            <span v-if="band" :key="band.label" class="score-band" :style="{ color: band.color }">{{ band.label }}</span>
          </Transition>
        </span>
      </div>
    </div>

    <div class="field-group">
      <span class="group-title">分档</span>
      <TierRail v-model="tier" :tiers="tiers" />
    </div>

    <div class="field-group">
      <span class="group-title">状态</span>
      <SegmentedControl v-model="status" :options="statusOptions" aria-label="观看状态" />
    </div>

    <fieldset class="field-group">
      <legend class="group-title">分项评分</legend>
      <div v-for="key in dimensionKeys" :key="key" class="dim-row">
        <span class="dim-label">{{ dimensionLabels[key] }}</span>
        <StarRatingInput v-model="dimValues[key]" :label="dimensionLabels[key]" />
        <span class="dim-band">
          <Transition name="band" mode="out-in">
            <span :key="dimValues[key] === null ? '未评' : dimensionBand(dimValues[key]!)">{{
              dimValues[key] === null ? '未评' : dimensionBand(dimValues[key]!)
            }}</span>
          </Transition>
        </span>
      </div>
    </fieldset>

    <label class="field-group as-label">
      <span class="group-title">短评<span class="counter">{{ review.length }}/5000</span></span>
      <textarea
        v-model="review"
        class="review-area"
        rows="4"
        maxlength="5000"
        placeholder="写点短评……"
      ></textarea>
    </label>

    <p v-if="error" class="form-error" role="alert">{{ error }}</p>

    <div class="form-actions">
      <button type="button" class="btn btn-ghost" @click="onCancel">取消</button>
      <button type="submit" class="btn btn-primary" :disabled="submitting">{{ submitLabel }}</button>
    </div>
  </form>
</template>

<style scoped>
.rating-form {
  display: flex;
  flex-direction: column;
  gap: 24px;
  max-width: 800px;
  width: 100%;
  padding: 26px;
  background: var(--surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.field-group {
  border: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.field-group.as-label {
  cursor: default;
}

.group-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-soft);
  padding: 0;
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.counter {
  font-weight: 400;
  color: var(--muted);
  font-size: 12px;
}

.group-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.skip-switch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border-radius: var(--radius-pill);
}

.skip-text {
  font-size: 12.5px;
  color: var(--muted);
}

.switch-track {
  position: relative;
  width: 40px;
  height: 24px;
  border-radius: var(--radius-pill);
  background: var(--fill-strong);
  transition: background 200ms var(--ease-snap);
}

.switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #fff;
  box-shadow: var(--shadow-thumb), 0 1px 4px rgba(0, 0, 0, 0.16);
  transition: transform 240ms var(--ease-spring);
}

.skip-switch[aria-checked='true'] .switch-track {
  background: var(--brand);
}

.skip-switch[aria-checked='true'] .switch-knob {
  transform: translateX(16px);
}

.skip-switch:hover .skip-text {
  color: var(--text-soft);
}

.score-body {
  display: flex;
  align-items: center;
  gap: 18px;
  transition: opacity 200ms var(--ease-snap);
}

.score-body.off {
  opacity: 0.35;
  pointer-events: none;
}

.score-number {
  width: 84px;
  height: 58px;
  flex-shrink: 0;
  text-align: center;
  font-size: 30px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text);
  background: var(--fill);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  appearance: textfield;
  -moz-appearance: textfield;
  transition: background var(--motion-fast) var(--ease-snap),
    border-color var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap);
}

.score-number::-webkit-outer-spin-button,
.score-number::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.score-number:hover:not(:focus):not(:disabled) {
  background: var(--fill-strong);
}

.score-number:focus {
  background: var(--surface);
  border-color: var(--brand);
  outline: none;
  box-shadow: var(--focus-ring);
}

.score-number::placeholder {
  color: var(--muted);
}

.score-slider {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

@property --pos {
  syntax: '<percentage>';
  inherits: false;
  initial-value: 0%;
}

.score-range {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 28px;
  margin: 0;
  background: transparent;
  transition: --pos 260ms var(--ease-snap);
}

.score-range.dragging {
  transition: none;
}

.score-range::-webkit-slider-runnable-track {
  height: 6px;
  border-radius: var(--radius-pill);
  background: linear-gradient(
    to right,
    var(--brand) var(--pos, 0%),
    var(--fill-strong) var(--pos, 0%)
  );
}

.score-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 22px;
  height: 22px;
  margin-top: -8px;
  border: none;
  border-radius: 50%;
  background: #fff;
  box-shadow: var(--shadow-thumb), 0 1px 5px rgba(0, 0, 0, 0.14);
  cursor: grab;
  transition: transform 140ms ease-out;
}

.score-range:hover::-webkit-slider-thumb {
  transform: scale(1.08);
}

.score-range:active::-webkit-slider-thumb {
  transform: scale(1.14);
  cursor: grabbing;
}

.score-range::-moz-range-track {
  height: 6px;
  border-radius: var(--radius-pill);
  background: var(--fill-strong);
}

.score-range::-moz-range-progress {
  height: 6px;
  border-radius: var(--radius-pill);
  background: var(--brand);
}

.score-range::-moz-range-thumb {
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 50%;
  background: #fff;
  box-shadow: var(--shadow-thumb), 0 1px 5px rgba(0, 0, 0, 0.14);
  cursor: grab;
}

.score-range:focus-visible {
  outline: none;
}

.score-range:focus-visible::-webkit-slider-thumb {
  box-shadow: var(--shadow-thumb), var(--focus-ring);
}

.score-range:focus-visible::-moz-range-thumb {
  box-shadow: var(--shadow-thumb), var(--focus-ring);
}

.ticks {
  display: flex;
  justify-content: space-between;
  margin: 3px 11px 0;
}

.ticks span {
  width: 1.5px;
  height: 4px;
  border-radius: 1px;
  background: var(--border-strong);
}

.ticks span:first-child,
.ticks span:last-child {
  height: 6px;
}

.scale {
  position: relative;
  height: 15px;
  margin: 0 11px;
}

.scale span {
  position: absolute;
  top: 0;
  font-size: 10.5px;
  color: var(--muted);
  transform: translateX(-50%);
}

.scale .lo {
  left: 0;
}

.scale .mid {
  left: 50%;
}

.scale .hi {
  left: 100%;
}

.band-slot {
  width: 44px;
  flex-shrink: 0;
}

.score-band {
  display: inline-block;
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
}

.band-enter-active,
.band-leave-active {
  transition: opacity 180ms var(--ease-snap), transform 180ms var(--ease-snap);
}

.band-enter-from {
  opacity: 0;
  transform: translateY(5px);
}

.band-leave-to {
  opacity: 0;
  transform: translateY(-5px);
}

.dim-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 0;
  border-bottom: 1px solid var(--border);
}

.dim-label {
  width: 42px;
  font-size: 14px;
  color: var(--text-soft);
  flex-shrink: 0;
}

.dim-band {
  margin-left: auto;
  min-width: 36px;
  font-size: 12px;
  color: var(--muted);
  flex-shrink: 0;
}

.dim-band > span {
  display: inline-block;
}

.review-area {
  width: 100%;
  min-height: 132px;
  padding: 14px;
  border: none;
  border-radius: var(--radius-md);
  background: var(--fill);
  resize: vertical;
  line-height: 1.7;
  color: var(--text);
  transition: background var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap);
}

.review-area:focus {
  background: var(--surface);
  outline: none;
  box-shadow: var(--focus-ring), inset 0 0 0 1px var(--brand);
}

.form-error {
  margin: 0;
  font-size: 13px;
  color: var(--danger);
  background: var(--danger-soft);
  padding: 10px 14px;
  border-radius: var(--radius-sm);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  border-top: 1px solid var(--border);
  padding-top: 20px;
}


@media (max-width: 1050px) {
  .rating-form {
    padding: 18px;
  }
}

@media (max-width: 640px) {
  .score-body {
    gap: 12px;
  }

  .score-number {
    width: 68px;
    height: 50px;
    font-size: 24px;
  }
}
</style>
