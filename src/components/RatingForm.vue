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
import { dimensionBand, dimensionLabels, statusLabels } from '../utils/format'
import StarRatingInput from './StarRatingInput.vue'

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

const statuses: WatchStatus[] = ['completed', 'watching', 'planned']

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
    <fieldset class="field-group">
      <legend class="group-title">总分</legend>
      <div class="score-row">
        <label class="check">
          <input type="checkbox" :checked="!scoreEnabled" @change="toggleSkipScore" />
          <span>暂不评分</span>
        </label>
        <template v-if="scoreEnabled">
          <input
            class="score-range"
            type="range"
            min="0"
            max="10"
            step="0.1"
            :value="sliderValue(scoreInput)"
            @input="scoreInput = ($event.target as HTMLInputElement).value"
            aria-label="总分滑杆"
          />
          <input
            v-model="scoreInput"
            class="field-input score-number"
            type="number"
            min="0"
            max="10"
            step="0.1"
            inputmode="decimal"
            aria-label="总分（0–10）"
            placeholder="0–10"
          />
        </template>
        <span v-else class="skip-note">这部作品暂时不打总分</span>
      </div>
    </fieldset>

    <div class="field-pair">
      <label class="field-group as-label">
        <span class="group-title">分档</span>
        <select v-model="tier" class="field-select">
          <option value="">未分档</option>
          <option v-for="item in tiers" :key="item.id" :value="item.name">
            {{ item.name }} · {{ item.description }}
          </option>
        </select>
      </label>
      <label class="field-group as-label">
        <span class="group-title">状态</span>
        <select v-model="status" class="field-select">
          <option v-for="s in statuses" :key="s" :value="s">{{ statusLabels[s] }}</option>
        </select>
      </label>
    </div>

    <fieldset class="field-group">
      <legend class="group-title">分项评分</legend>
      <div v-for="key in dimensionKeys" :key="key" class="dim-row">
        <span class="dim-label">{{ dimensionLabels[key] }}</span>
        <StarRatingInput v-model="dimValues[key]" :label="dimensionLabels[key]" />
        <span class="dim-band">{{ dimValues[key] === null ? '未评' : dimensionBand(dimValues[key]) }}</span>
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

.score-row {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
}

.check {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 14px;
  cursor: pointer;
}

.skip-note {
  font-size: 13px;
  color: var(--muted);
}

.score-range {
  flex: 1;
  min-width: 160px;
  accent-color: var(--brand);
}

.score-number {
  width: 88px;
  height: 44px;
  text-align: center;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text);
}

.field-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
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
  .field-pair {
    grid-template-columns: 1fr;
  }

  .score-range {
    min-width: 80px;
  }
}
</style>
