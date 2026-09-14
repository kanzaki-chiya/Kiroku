<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Ghost, Pencil, Plus, RotateCcw, Star, Trash2 } from 'lucide-vue-next'
import { bangumiApi } from '../api/bangumi'
import { isTauri } from '../runtime'
import AnimeCover from '../components/AnimeCover.vue'
import ScoreComparison from '../components/ScoreComparison.vue'
import { canMorph, morphCardId, runMorphNav } from '../services/motion'
import { useLibraryStore } from '../stores/library'
import { useNotices } from '../stores/notices'
import { dimensionKeys, type RelatedSubject } from '../types/anime'
import { cloneDraft } from '../utils/draft'
import { dimensionBand, dimensionLabels, formatLabels, formatScore, resolveTier, starFill, statusLabels, tierBadgeStyle, tierDescription } from '../utils/format'

const route = useRoute()
const router = useRouter()
const desktop = isTauri()
const store = useLibraryStore()
const { push } = useNotices()
const refreshing = shallowRef(false)
const removing = shallowRef(false)
const bumping = shallowRef(false)

const subjectId = computed(() => Number(route.params.id))
const entry = computed(() =>
  Number.isFinite(subjectId.value) ? store.getEntry(subjectId.value) : null
)

const progressText = computed(() => {
  const p = entry.value?.personal.progress
  if (p === null || p === undefined) return '未记录'
  const total = entry.value?.subject.episodes ?? 0
  return total > 0 ? `${p} / ${total} 话` : `${p} 话`
})

const progressPct = computed(() => {
  const p = entry.value?.personal.progress
  const total = entry.value?.subject.episodes ?? 0
  if (p === null || p === undefined || total <= 0) return 0
  return Math.min(100, Math.round((p / total) * 100))
})

const progressFull = computed(() => {
  const total = entry.value?.subject.episodes ?? 0
  const p = entry.value?.personal.progress ?? 0
  return total > 0 && p >= total
})

const related = shallowRef<RelatedSubject[]>([])
const relationsPhase = shallowRef<'loading' | 'done' | 'error'>('loading')
let relationsSeq = 0

async function loadRelations() {
  const id = subjectId.value
  const seq = ++relationsSeq
  if (!entry.value) {
    related.value = []
    relationsPhase.value = 'done'
    return
  }
  relationsPhase.value = 'loading'
  try {
    const list = await bangumiApi.getRelations(id)
    if (seq !== relationsSeq) return
    related.value = list
    relationsPhase.value = 'done'
  } catch {
    if (seq !== relationsSeq) return
    related.value = []
    relationsPhase.value = 'error'
  }
}

watch(subjectId, () => void loadRelations(), { immediate: true })

async function bumpProgress() {
  const current = entry.value
  if (!current || bumping.value) return
  const total = current.subject.episodes
  const draft = cloneDraft(current.personal)
  let next = (draft.progress ?? 0) + 1
  let finished = false
  if (total > 0 && next >= total) {
    next = total
    draft.status = 'completed'
    finished = true
  }
  draft.progress = next
  bumping.value = true
  try {
    await store.update(current.subject.id, draft)
    push(finished ? '已看完最后一话，状态改为已看完' : `已更新到第 ${next} 话`)
  } catch (error) {
    push(error instanceof Error ? error.message : '进度更新失败')
  } finally {
    bumping.value = false
  }
}

async function removeEntry() {
  const current = entry.value
  if (!current || removing.value) return
  const ok = window.confirm(
    `确定把「${current.subject.nameCn}」从番剧库移除吗？评分、分档和短评会一并删除，此操作无法撤销。`
  )
  if (!ok) return
  removing.value = true
  try {
    await router.push('/library')
    await store.remove(current.subject.id)
    push('已移除收藏')
  } catch (error) {
    push(error instanceof Error ? error.message : '移除失败')
  } finally {
    removing.value = false
  }
}

function collectRelated(subjectId: number) {
  router.push({ path: '/add', query: { subject: String(subjectId) } })
}

const updatedText = computed(() => {
  if (!entry.value) return ''
  return new Date(entry.value.personal.updatedAt).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
})

async function refreshMetadata() {
  if (!entry.value || refreshing.value) return
  refreshing.value = true
  try {
    await store.refreshSubject(entry.value.subject.id)
    push('已刷新 Bangumi 资料')
  } catch (error) {
    push(error instanceof Error ? error.message : '刷新失败，仍保留上次快照')
  } finally {
    refreshing.value = false
  }
}

function goBack(event: MouseEvent) {
  if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return
  if (!canMorph() || !entry.value) return
  event.preventDefault()
  morphCardId.value = entry.value.subject.id
  runMorphNav(router, '/library', () => {
    morphCardId.value = null
  })
}
</script>

<template>
  <div class="page">
    <nav class="breadcrumb" aria-label="面包屑">
      <RouterLink to="/library" class="back-link" @click="goBack">
        <ArrowLeft :size="14" aria-hidden="true" />番剧库
      </RouterLink>
      <span class="sep">/</span>
      <span aria-current="page">{{ entry?.subject.nameCn ?? '详情' }}</span>
    </nav>

    <div v-if="!entry" class="missing">
      <Ghost :size="36" class="missing-icon" aria-hidden="true" />
      <h1 class="missing-title">没有找到这部番剧</h1>
      <p class="missing-sub">它可能还没有被收录，或者链接地址有误。</p>
      <RouterLink to="/library" class="btn btn-primary">回到番剧库</RouterLink>
    </div>

    <div v-else class="detail">
      <aside class="poster-col">
        <AnimeCover :subject="entry.subject" class="poster" />
        <RouterLink :to="`/anime/${entry.subject.id}/edit`" class="btn btn-ghost edit-btn">
          <Pencil :size="14" aria-hidden="true" />编辑我的记录
        </RouterLink>
        <button
          v-if="store.desktop"
          type="button"
          class="btn btn-ghost edit-btn"
          :disabled="refreshing"
          @click="refreshMetadata"
        >
          <RotateCcw :size="14" aria-hidden="true" />刷新资料
        </button>
        <div v-if="entry.personal.status === 'watching'" class="watch-progress">
          <div class="wp-head">
            <span class="wp-label">观看进度</span>
            <span class="wp-value">{{ progressText }}</span>
          </div>
          <div class="wp-track" role="img" :aria-label="`已看 ${progressText}`">
            <i :style="{ width: `${progressPct}%` }" />
          </div>
          <button
            type="button"
            class="wp-plus"
            :disabled="bumping || progressFull"
            @click="bumpProgress"
          >
            <Plus :size="13" aria-hidden="true" />{{ progressFull ? '已看完' : '再看一话' }}
          </button>
        </div>
        <dl class="fact-list">
          <div class="fact">
            <dt>状态</dt>
            <dd>{{ statusLabels[entry.personal.status] }}</dd>
          </div>
          <div class="fact">
            <dt>进度</dt>
            <dd>{{ progressText }}</dd>
          </div>
          <div class="fact">
            <dt>分档</dt>
            <dd v-if="entry.personal.tier" class="tier-line">
              <span
                class="tier-badge"
                :style="tierBadgeStyle(resolveTier(entry.personal.tier, store.tiers)?.color)"
              >
                {{ entry.personal.tier }}
              </span>
              {{ tierDescription(entry.personal.tier, store.tiers) }}
            </dd>
            <dd v-else class="muted">未分档</dd>
          </div>
          <div class="fact">
            <dt>收录于</dt>
            <dd>{{ new Date(entry.personal.createdAt).toLocaleDateString('zh-CN') }}</dd>
          </div>
          <div class="fact">
            <dt>更新于</dt>
            <dd>{{ updatedText }}</dd>
          </div>
        </dl>
        <button
          type="button"
          class="remove-btn"
          :disabled="removing"
          @click="removeEntry"
        >
          <Trash2 :size="13" aria-hidden="true" />移除收藏
        </button>
      </aside>

      <section class="detail-main">
        <header class="detail-head">
          <h1 class="title">{{ entry.subject.nameCn }}</h1>
          <p class="original">{{ entry.subject.name }}</p>
          <p class="meta">
            {{ entry.subject.year }} · {{ formatLabels[entry.subject.format] }} ·
            {{ entry.subject.episodes }} 话 · {{ entry.subject.studio }}
          </p>
          <div class="tags">
            <span v-for="tag in entry.subject.tags" :key="tag" class="chip">{{ tag }}</span>
          </div>
        </header>

        <p class="synopsis">{{ entry.subject.summary }}</p>

        <div class="community-line">
          <span>Bangumi 社区评分{{ desktop ? '' : '（Mock）' }}：{{ formatScore(entry.subject.community.score) }}</span>
          <span>· {{ entry.subject.community.votes.toLocaleString() }} 人评价</span>
          <span v-if="entry.subject.community.rank !== null">
            · 站内排名 #{{ entry.subject.community.rank }}
          </span>
        </div>

        <div class="compare-block">
          <ScoreComparison
            :personal="entry.personal.score"
            :community="entry.subject.community.score"
          />
        </div>

        <section class="dims" aria-label="分项评分">
          <h2 class="section-title">我的分项</h2>
          <div v-for="key in dimensionKeys" :key="key" class="dim-row">
            <span class="dim-label">{{ dimensionLabels[key] }}</span>
            <div
              class="dim-stars"
              role="img"
              :aria-label="entry.personal.dimensions[key] === null
                ? `${dimensionLabels[key]}未评分`
                : `${dimensionLabels[key]} ${entry.personal.dimensions[key]} 星 ${dimensionBand(entry.personal.dimensions[key]!)}`"
            >
              <span
                v-for="n in 5"
                :key="n"
                class="star-visual"
                :data-fill="starFill(entry.personal.dimensions[key], n)"
              >
                <Star class="star-icon star-empty" :size="16" />
                <span class="star-clip">
                  <Star class="star-icon star-filled" :size="16" />
                </span>
              </span>
            </div>
            <span class="dim-value" :class="{ muted: entry.personal.dimensions[key] === null }">
              {{ entry.personal.dimensions[key] === null ? '未评分' : entry.personal.dimensions[key]!.toFixed(1) }}
            </span>
            <span v-if="entry.personal.dimensions[key] !== null" class="dim-band">
              {{ dimensionBand(entry.personal.dimensions[key]!) }}
            </span>
          </div>
        </section>

        <section class="review-block">
          <h2 class="section-title">我的短评</h2>
          <p v-if="entry.personal.review.trim()" class="review-text">{{ entry.personal.review }}</p>
          <p v-else class="review-empty">
            还没有写下想法。
            <RouterLink :to="`/anime/${entry.subject.id}/edit`">去补一句 →</RouterLink>
          </p>
        </section>

        <section class="related-block">
          <h2 class="section-title">系列作品</h2>
          <p v-if="relationsPhase === 'loading'" class="related-state">正在读取关联条目…</p>
          <div v-else-if="relationsPhase === 'error'" class="related-state error" role="alert">
            <span>关联条目读取失败。</span>
            <button type="button" class="related-retry" @click="loadRelations">
              <RotateCcw :size="12" aria-hidden="true" />重试
            </button>
          </div>
          <p v-else-if="related.length === 0" class="related-state">没有关联条目。</p>
          <ul v-else class="related-list">
            <li v-for="item in related" :key="item.subject.id" class="related-row">
              <span class="related-thumb"><AnimeCover :subject="item.subject" /></span>
              <span class="related-text">
                <span class="related-title">{{ item.subject.nameCn || item.subject.name }}</span>
                <span class="related-meta">
                  <span class="related-relation">{{ item.relation }}</span>
                  {{ item.subject.year || '—' }} · {{ formatLabels[item.subject.format] ?? item.subject.format }}
                </span>
              </span>
              <RouterLink
                v-if="store.hasSubject(item.subject.id)"
                :to="`/anime/${item.subject.id}`"
                class="related-action"
              >
                已收录 · 查看
              </RouterLink>
              <button
                v-else
                type="button"
                class="related-action as-btn"
                @click="collectRelated(item.subject.id)"
              >
                收录
              </button>
            </li>
          </ul>
        </section>
      </section>
    </div>
  </div>
</template>

<style scoped>
.back-link {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.missing {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 90px 20px;
  text-align: center;
}

.missing-icon {
  color: var(--muted);
  width: 44px;
  height: 44px;
  padding: 14px;
  background: var(--fill);
  border-radius: 50%;
  box-sizing: content-box;
}

.missing-title {
  margin: 8px 0 0;
  font: 700 26px/1.25 var(--font-display);
  letter-spacing: -0.02em;
}

.missing-sub {
  margin: 0 0 16px;
  color: var(--muted);
  font-size: 14px;
}

.detail {
  display: grid;
  grid-template-columns: 216px minmax(0, 1fr);
  gap: 32px;
  max-width: 1120px;
  align-items: start;
}

.poster-col {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
  padding-top: 4px;
}

.poster {
  padding: 6px;
  background: var(--surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lift);
  view-transition-name: cover-morph;
}

.edit-btn {
  justify-content: center;
  width: 100%;
  font-size: 12px;
}

.fact-list {
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--border);
}

.fact {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid var(--border);
  font-size: 11px;
}

.fact dt {
  color: var(--muted);
  flex-shrink: 0;
}

.fact dd {
  margin: 0;
  min-width: 0;
  text-align: right;
  overflow-wrap: anywhere;
}

.tier-line {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 6px;
}

.muted {
  color: var(--muted);
}

.detail-main {
  min-width: 0;
  padding: 32px;
  background: var(--surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.detail-head {
  margin-bottom: 22px;
  padding-bottom: 22px;
  border-bottom: 1px solid var(--border);
}

.title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 30px;
  line-height: 1.25;
  font-weight: 700;
  letter-spacing: -0.025em;
  overflow-wrap: anywhere;
}

.original {
  margin: 7px 0 0;
  font-size: 12px;
  color: var(--muted);
  overflow-wrap: anywhere;
}

.meta {
  margin: 10px 0 0;
  font-size: 12px;
  color: var(--text-soft);
  overflow-wrap: anywhere;
}

.tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 14px;
}

.synopsis {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.95;
  color: var(--text-soft);
  overflow-wrap: anywhere;
}

.community-line {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 5px;
  font-size: 11px;
  color: var(--muted);
  margin-bottom: 22px;
  overflow-wrap: anywhere;
}

.compare-block {
  padding: 22px;
  border-radius: var(--radius-md);
  background: var(--surface-soft);
  margin-bottom: 26px;
}

.section-title {
  margin: 0 0 14px;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.01em;
  display: flex;
  align-items: center;
  gap: 9px;
}

.dims {
  margin-bottom: 24px;
}

.dim-row {
  display: grid;
  grid-template-columns: 42px minmax(90px, 1fr) 40px 40px;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}

.dim-label {
  font-size: 12px;
  color: var(--text-soft);
}

.dim-stars {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.star-visual {
  position: relative;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
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
}

.star-visual[data-fill='half'] .star-clip {
  width: 50%;
}

.star-visual[data-fill='full'] .star-clip {
  width: 100%;
}

.dim-value {
  text-align: right;
  font-size: 13px;
  font-weight: 600;
}

.dim-value.muted {
  font: 11px var(--font-body);
}

.dim-band {
  font-size: 11px;
  color: var(--muted);
  text-align: right;
}

.review-text {
  margin: 0;
  padding: 16px 18px;
  background: var(--surface-soft);
  border-radius: var(--radius-md);
  font-size: 13px;
  line-height: 1.8;
  color: var(--text-soft);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.review-empty {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
}

.watch-progress {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border-radius: var(--radius-md);
  background: var(--surface);
  box-shadow: var(--shadow-card);
}

.wp-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
}

.wp-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-soft);
}

.wp-value {
  font-size: 12px;
  font-weight: 600;
  color: var(--score-gold);
}

.wp-track {
  height: 5px;
  border-radius: var(--radius-pill);
  background: var(--fill);
  overflow: hidden;
}

.wp-track i {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: var(--score-gold);
  transition: width 320ms var(--ease-spring);
}

.wp-plus {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--brand);
  background: var(--fill);
  border-radius: var(--radius-sm);
  padding: 8px 12px;
  transition: background var(--motion-fast) var(--ease-snap),
    transform 100ms ease-out;
}

.wp-plus:hover:not(:disabled) {
  background: var(--fill-strong);
}

.wp-plus:active:not(:disabled) {
  transform: scale(0.97);
}

.wp-plus:disabled {
  color: var(--muted);
  cursor: default;
}

.remove-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  margin-top: 14px;
  font-size: 12px;
  color: var(--muted);
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  transition: color var(--motion-fast) var(--ease-snap),
    background var(--motion-fast) var(--ease-snap);
}

.remove-btn:hover:not(:disabled) {
  color: var(--danger);
  background: var(--danger-soft);
}

.remove-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.related-block {
  margin-top: 26px;
  padding-top: 22px;
  border-top: 1px solid var(--border);
}

.related-state {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
}

.related-state.error {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--danger);
}

.related-retry {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--brand);
  background: var(--fill);
  border-radius: var(--radius-sm);
  padding: 5px 12px;
  transition: background var(--motion-fast) var(--ease-snap);
}

.related-retry:hover {
  background: var(--fill-strong);
}

.related-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}

.related-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid var(--border);
}

.related-row:last-child {
  border-bottom: none;
}

.related-thumb {
  width: 40px;
  flex-shrink: 0;
}

.related-thumb :deep(.cover) {
  border-radius: var(--radius-xs);
}

.related-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
  flex: 1;
}

.related-title {
  font-size: 13.5px;
  font-weight: 600;
  letter-spacing: -0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.related-meta {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 11.5px;
  color: var(--muted);
}

.related-relation {
  padding: 1px 7px;
  border-radius: var(--radius-pill);
  background: var(--fill);
  color: var(--text-soft);
  font-size: 10.5px;
  font-weight: 500;
}

.related-action {
  flex-shrink: 0;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--brand);
  text-decoration: none;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  transition: background var(--motion-fast) var(--ease-snap);
}

.related-action:hover {
  background: var(--fill);
  text-decoration: none;
}

.related-action.as-btn {
  background: var(--fill);
}

.related-action.as-btn:hover {
  background: var(--fill-strong);
}

@media (max-width: 1050px) {
  .detail {
    grid-template-columns: 160px minmax(0, 1fr);
    gap: 20px;
  }

  .detail-main {
    padding: 20px;
  }

  .title {
    font-size: 27px;
  }

  .fact {
    flex-direction: column;
    gap: 3px;
  }

  .fact dd {
    text-align: left;
  }

  .compare-block {
    padding: 14px;
  }

  .dim-row {
    grid-template-columns: 30px minmax(84px, 1fr) 36px 32px;
    gap: 6px;
  }

  .dim-stars {
    gap: 2px;
  }
}

@media (max-width: 900px) {
  .detail {
    grid-template-columns: 140px minmax(0, 1fr);
    gap: 16px;
  }

  .detail-main {
    padding: 18px;
  }

  .edit-btn {
    padding: 0 6px;
    font-size: 11px;
  }
}
</style>
