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
import { cloneDraft } from '../utils/draft'
import { dimensionLabels, statusLabels } from '../utils/format'

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

const dimEnabled = reactive<Record<DimensionKey, boolean>>({
  story: props.initial.dimensions.story !== null,
  characters: props.initial.dimensions.characters !== null,
  direction: props.initial.dimensions.direction !== null,
  animation: props.initial.dimensions.animation !== null,
  music: props.initial.dimensions.music !== null
})

const dimInputs = reactive<Record<DimensionKey, string>>({
  story: props.initial.dimensions.story === null ? '' : String(props.initial.dimensions.story),
  characters: props.initial.dimensions.characters === null ? '' : String(props.initial.dimensions.characters),
  direction: props.initial.dimensions.direction === null ? '' : String(props.initial.dimensions.direction),
  animation: props.initial.dimensions.animation === null ? '' : String(props.initial.dimensions.animation),
  music: props.initial.dimensions.music === null ? '' : String(props.initial.dimensions.music)
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
  dimensions: {
    story: dimEnabled.story ? parseLenient(dimInputs.story) : null,
    characters: dimEnabled.characters ? parseLenient(dimInputs.characters) : null,
    direction: dimEnabled.direction ? parseLenient(dimInputs.direction) : null,
    animation: dimEnabled.animation ? parseLenient(dimInputs.animation) : null,
    music: dimEnabled.music ? parseLenient(dimInputs.music) : null
  },
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

function enableDimension(key: DimensionKey) {
  dimEnabled[key] = true
  if (dimInputs[key] === '') dimInputs[key] = '7'
}

function clearDimension(key: DimensionKey) {
  dimEnabled[key] = false
  dimInputs[key] = ''
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
    if (!dimEnabled[key]) {
      dimensions[key] = null
      continue
    }
    const result = validateScoreField(dimInputs[key], dimensionLabels[key])
    if (typeof result === 'string') {
      error.value = result
      return
    }
    dimensions[key] = result
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
        <template v-if="dimEnabled[key]">
          <input
            class="dim-range"
            type="range"
            min="0"
            max="10"
            step="0.1"
            :value="sliderValue(dimInputs[key])"
            :aria-label="`${dimensionLabels[key]}滑杆`"
            @input="dimInputs[key] = ($event.target as HTMLInputElement).value"
          />
          <input
            v-model="dimInputs[key]"
            class="field-input dim-number"
            type="number"
            min="0"
            max="10"
            step="0.1"
            inputmode="decimal"
            :aria-label="`${dimensionLabels[key]}（0–10）`"
          />
          <button type="button" class="dim-clear" @click="clearDimension(key)">未评分</button>
        </template>
        <template v-else>
          <span class="dim-empty">未评分</span>
          <button type="button" class="dim-enable" @click="enableDimension(key)">评一下</button>
        </template>
      </div>
    </fieldset>

    <label class="field-group as-label">
      <span class="group-title">短评<span class="counter">{{ review.length }}/5000</span></span>
      <textarea
        v-model="review"
        class="review-area"
        rows="4"
        maxlength="5000"
        placeholder="这部作品留给你的，是一句话、一个场景，还是一段心情？"
      ></textarea>
    </label>

    <p v-if="error" class="form-error" role="alert">{{ error }}</p>

    <p class="form-note">总分独立填写；分项与 Bangumi 评分不参与总分计算。</p>

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
  gap: 22px;
  max-width: 640px;
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
  font-size: 13.5px;
  color: var(--muted);
}

.score-range,
.dim-range {
  flex: 1;
  min-width: 160px;
  accent-color: var(--brand);
}

.dim-range {
  min-width: 0;
}

.score-number,
.dim-number {
  width: 84px;
  text-align: center;
}

.dim-number {
  width: 70px;
  flex-shrink: 0;
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
  padding: 6px 0;
}

.dim-label {
  width: 42px;
  font-size: 14px;
  color: var(--text-soft);
  flex-shrink: 0;
}

.dim-empty {
  flex: 1;
  font-size: 13.5px;
  color: var(--muted);
}

.dim-enable,
.dim-clear {
  font-size: 12.5px;
  color: var(--brand);
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--border-strong);
  flex-shrink: 0;
}

.dim-clear {
  padding: 4px 7px;
}

.dim-enable:hover,
.dim-clear:hover {
  border-color: var(--brand);
}

.review-area {
  width: 100%;
  padding: 11px 13px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
  resize: vertical;
  min-height: 96px;
  line-height: 1.7;
}

.review-area:focus {
  border-color: var(--brand);
  outline: none;
  box-shadow: 0 0 0 3px rgba(51, 93, 78, 0.14);
}

.form-error {
  margin: 0;
  font-size: 13.5px;
  color: var(--danger);
  background: #faece7;
  border: 1px solid #eccabc;
  padding: 9px 14px;
  border-radius: var(--radius-sm);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.form-note {
  margin: 0;
  font-size: 12.5px;
  color: var(--muted);
}

@media (max-width: 640px) {
  .field-pair {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 520px) {
  .dim-row {
    display: grid;
    grid-template-columns: 36px minmax(0, 1fr) 64px 58px;
    gap: 8px;
  }

  .dim-number {
    width: 64px;
    padding: 0 4px;
  }

  .dim-empty {
    grid-column: 2 / 4;
  }

  .dim-enable {
    grid-column: 4;
  }

  .score-range {
    min-width: 80px;
  }

  .dim-clear {
    font-size: 12px;
  }
}
</style>
