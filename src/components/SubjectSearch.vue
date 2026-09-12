<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RotateCcw, Search } from 'lucide-vue-next'
import { bangumiApi } from '../api/bangumi'
import { isTauri } from '../runtime'
import { useLibraryStore } from '../stores/library'
import type { BangumiSubject, PersonalDraft } from '../types/anime'
import { emptyDraft } from '../utils/draft'
import { formatLabels, formatScore } from '../utils/format'
import AnimeCover from './AnimeCover.vue'
import RatingForm from './RatingForm.vue'

const emit = defineEmits<{
  saved: [subjectId: number]
  dirty: [dirty: boolean]
}>()

const store = useLibraryStore()
const desktop = isTauri()
const entryInitial = { ...emptyDraft(), score: 7 }

const query = ref('')
const results = ref<BangumiSubject[]>([])
const phase = ref<'idle' | 'loading' | 'done' | 'error'>('idle')
const selected = ref<BangumiSubject | null>(null)
const formDirty = ref(false)
const addError = ref('')

const catalog = ref<BangumiSubject[]>([])
const suggestionsPhase = ref<'loading' | 'done' | 'error'>('loading')

let abort: AbortController | null = null
let suggestAbort: AbortController | null = null
let debounceTimer: ReturnType<typeof setTimeout> | null = null
let sequence = 0

const suggestions = computed(() =>
  catalog.value.filter(subject => !store.hasSubject(subject.id)).slice(0, 6)
)

async function loadSuggestions() {
  suggestAbort?.abort()
  const controller = new AbortController()
  suggestAbort = controller
  suggestionsPhase.value = 'loading'
  try {
    const list = await bangumiApi.getSuggestions(controller.signal)
    if (suggestAbort !== controller) return
    catalog.value = list
    suggestionsPhase.value = 'done'
  } catch (err) {
    if ((err as DOMException).name === 'AbortError') return
    suggestionsPhase.value = 'error'
  }
}

async function runSearch() {
  const text = query.value.trim()
  abort?.abort()
  const seq = ++sequence
  if (!text) {
    phase.value = 'idle'
    results.value = []
    return
  }
  const controller = new AbortController()
  abort = controller
  phase.value = 'loading'
  results.value = []
  try {
    const found = await bangumiApi.searchSubjects(text, controller.signal)
    if (seq !== sequence) return
    results.value = found
    phase.value = 'done'
  } catch (err) {
    if ((err as DOMException).name === 'AbortError' || seq !== sequence) return
    phase.value = 'error'
  }
}

watch(query, () => {
  abort?.abort()
  sequence++
  if (debounceTimer) clearTimeout(debounceTimer)
  const text = query.value.trim()
  if (!text) {
    phase.value = 'idle'
    results.value = []
    return
  }
  phase.value = 'loading'
  results.value = []
  debounceTimer = setTimeout(runSearch, 250)
})

function retry() {
  if (query.value.trim()) runSearch()
  else loadSuggestions()
}

async function pick(subject: BangumiSubject) {
  if (store.hasSubject(subject.id)) return
  addError.value = ''
  formDirty.value = false
  try {
    const full = await bangumiApi.getSubject(subject.id)
    selected.value = full ?? subject
  } catch (err) {
    addError.value = err instanceof Error ? err.message : '无法读取作品资料'
    selected.value = subject
  }
}

function reselect() {
  if (formDirty.value && !window.confirm('当前填写的评分还没有保存，确定放弃吗？')) return
  selected.value = null
  formDirty.value = false
  emit('dirty', false)
}

function onDirty(value: boolean) {
  formDirty.value = value
  emit('dirty', value)
}

async function onSave(draft: PersonalDraft) {
  if (!selected.value) return
  try {
    await store.add(selected.value, draft)
  } catch (err) {
    addError.value = (err as Error).message
    return
  }
  formDirty.value = false
  emit('dirty', false)
  emit('saved', selected.value.id)
}

function onFormCancel() {
  reselect()
}

onMounted(loadSuggestions)

onBeforeUnmount(() => {
  abort?.abort()
  suggestAbort?.abort()
  sequence++
  if (debounceTimer) clearTimeout(debounceTimer)
})
</script>

<template>
  <div class="subject-search">
    <template v-if="!selected">
      <label class="search-box">
        <Search :size="16" class="search-icon" aria-hidden="true" />
        <input
          v-model="query"
          class="search-input"
          type="search"
          placeholder="搜索中文名、原名、标签或年份…"
          aria-label="搜索番剧作品"
        />
      </label>

      <p v-if="phase === 'loading'" class="state-line" role="status">正在搜索…</p>

      <div v-else-if="phase === 'error'" class="state-line error" role="alert">
        <span>搜索出了点问题，请重试。</span>
        <button type="button" class="retry" @click="retry">
          <RotateCcw :size="13" aria-hidden="true" />重试
        </button>
      </div>

      <p v-else-if="phase === 'done' && results.length === 0" class="state-line">
        没有找到「{{ query.trim() }}」相关的作品，换个关键词试试。
      </p>

      <ul v-else-if="phase === 'done'" class="result-list" aria-label="搜索结果">
        <li v-for="subject in results" :key="subject.id">
          <div class="result" :class="{ collected: store.hasSubject(subject.id) }">
            <button
              v-if="!store.hasSubject(subject.id)"
              type="button"
              class="result-main as-button"
              @click="pick(subject)"
            >
              <span class="thumb"><AnimeCover :subject="subject" /></span>
              <span class="result-text">
                <span class="result-title">{{ subject.nameCn }}</span>
                <span class="result-sub">{{ subject.name }}</span>
                <span class="result-meta">
                  {{ subject.year }} · {{ formatLabels[subject.format] }} ·
                  社区 {{ formatScore(subject.community.score) }}
                  <span v-if="!desktop" class="mock">Mock</span>
                </span>
              </span>
              <span class="result-action">选择 →</span>
            </button>
            <template v-else>
              <span class="result-main">
                <span class="thumb"><AnimeCover :subject="subject" /></span>
                <span class="result-text">
                  <span class="result-title">{{ subject.nameCn }}</span>
                  <span class="result-sub">{{ subject.name }}</span>
                  <span class="result-meta">
                    {{ subject.year }} · {{ formatLabels[subject.format] }} ·
                    社区 {{ formatScore(subject.community.score) }}
                    <span v-if="!desktop" class="mock">Mock</span>
                  </span>
                </span>
              </span>
              <RouterLink class="collected-link" :to="`/anime/${subject.id}`">已收录 · 查看详情</RouterLink>
            </template>
          </div>
        </li>
      </ul>

      <div v-else class="idle-block">
        <p v-if="suggestionsPhase === 'loading'" class="state-line" role="status">
          {{ desktop ? '正在读取最近搜索…' : '正在读取推荐…' }}
        </p>
        <div v-else-if="suggestionsPhase === 'error'" class="state-line error" role="alert">
          <span>{{ desktop ? '最近搜索读取失败。' : '推荐列表读取失败。' }}</span>
          <button type="button" class="retry" @click="retry">
            <RotateCcw :size="13" aria-hidden="true" />重试
          </button>
        </div>
        <template v-else>
          <p v-if="suggestions.length === 0" class="state-line">
            {{ desktop ? '搜索 Bangumi 上的作品开始收录。' : '还没想好？这些作品或许值得收录：' }}
          </p>
          <template v-else>
            <p class="state-line">
              {{ desktop ? '最近搜索过的作品：' : '还没想好？这些作品或许值得收录：' }}
            </p>
            <ul class="result-list" :aria-label="desktop ? '最近搜索' : '推荐收录'">
              <li v-for="subject in suggestions" :key="subject.id">
                <button type="button" class="result as-button result-main" @click="pick(subject)">
                  <span class="thumb"><AnimeCover :subject="subject" /></span>
                  <span class="result-text">
                    <span class="result-title">{{ subject.nameCn }}</span>
                    <span class="result-sub">{{ subject.name }}</span>
                    <span class="result-meta">
                      {{ subject.year }} · {{ formatLabels[subject.format] }} ·
                      社区 {{ formatScore(subject.community.score) }}
                      <span v-if="!desktop" class="mock">Mock</span>
                    </span>
                  </span>
                  <span class="result-action">选择 →</span>
                </button>
              </li>
            </ul>
          </template>
        </template>
      </div>
    </template>

    <template v-else>
      <div class="chosen">
        <div class="chosen-head">
          <span class="thumb"><AnimeCover :subject="selected" /></span>
          <div class="chosen-text">
            <p class="chosen-title">{{ selected.nameCn }}</p>
            <p class="chosen-sub">{{ selected.name }}</p>
            <p class="chosen-meta">
              {{ selected.year }} · {{ formatLabels[selected.format] }} · {{ selected.episodes }} 话 ·
              {{ selected.studio }}
            </p>
            <button type="button" class="reselect" @click="reselect">重新选择</button>
          </div>
        </div>
        <p v-if="addError" class="add-error" role="alert">{{ addError }}</p>
        <RatingForm
          :initial="entryInitial"
          :tiers="store.tiers"
          :busy="store.saving"
          submit-label="收录到我的番剧库"
          @save="onSave"
          @cancel="onFormCancel"
          @dirty-change="onDirty"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.subject-search {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
  max-width: 800px;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: var(--muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 46px;
  padding: 0 14px 0 37px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
  font-size: 14.5px;
}

.search-input:focus {
  border-color: var(--brand);
  outline: none;
  box-shadow: var(--focus-ring);
}

.state-line {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
}

.state-line.error {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--danger);
}

.retry {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
  color: var(--brand);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  padding: 3px 12px;
}

.retry:hover {
  border-color: var(--brand);
}

.result-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 800px;
}

.result {
  display: flex;
  align-items: center;
  gap: 0;
  width: 100%;
  text-align: left;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.result-main {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
  min-width: 0;
  padding: 14px 16px;
}

.result-main.as-button {
  border-radius: var(--radius-md);
}

.result-main.as-button:hover {
  background: var(--surface-soft);
}

.result:hover {
  border-color: var(--border-strong);
  box-shadow: var(--shadow-card);
}

.result.collected {
  background: var(--surface-soft);
}

.thumb {
  width: 48px;
  flex-shrink: 0;
}

.thumb :deep(.cover) {
  border-radius: var(--radius-xs);
}

.result-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.result-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
}

.result-sub {
  font-size: 12.5px;
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-meta {
  font-size: 12.5px;
  color: var(--muted);
  margin-top: 3px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.mock {
  font-size: 10px;
  padding: 0 5px;
  border-radius: var(--radius-xs);
  background: var(--slate-soft);
  color: var(--slate);
}

.result-action {
  font-size: 13px;
  font-weight: 600;
  color: var(--brand);
  flex-shrink: 0;
}

.collected-link {
  font-size: 12.5px;
  color: var(--muted);
  padding: 0 14px;
  flex-shrink: 0;
}

.collected-link:hover {
  color: var(--brand);
}

.chosen {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 800px;
}

.chosen-head {
  display: flex;
  gap: 18px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  max-width: 800px;
}

.chosen-head .thumb {
  width: 88px;
}

.chosen-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.chosen-title {
  margin: 0;
  font-size: 17px;
  font-weight: 700;
}

.chosen-sub {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
}

.chosen-meta {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--muted);
}

.reselect {
  align-self: flex-start;
  margin-top: 8px;
  font-size: 12.5px;
  color: var(--brand);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  padding: 3px 12px;
}

.reselect:hover {
  border-color: var(--brand);
}

.add-error {
  margin: 0;
  font-size: 13px;
  color: var(--danger);
}

@media (max-width: 1050px) {
  .result-meta {
    flex-wrap: wrap;
  }

  .result-main {
    gap: 10px;
  }

  .collected-link {
    max-width: 100px;
    white-space: normal;
  }
}

@media (max-width: 520px) {
  .search-box {
    max-width: none;
  }

  .chosen-head {
    gap: 12px;
    padding: 12px 14px;
  }

  .chosen-head .thumb {
    width: 60px;
  }
}
</style>
