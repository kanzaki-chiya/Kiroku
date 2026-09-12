<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft, Ghost, Pencil, RotateCcw, Star } from 'lucide-vue-next'
import AnimeCover from '../components/AnimeCover.vue'
import ScoreComparison from '../components/ScoreComparison.vue'
import { useLibraryStore } from '../stores/library'
import { useNotices } from '../stores/notices'
import { dimensionKeys } from '../types/anime'
import { dimensionBand, dimensionLabels, formatLabels, formatScore, resolveTier, starFill, statusLabels, tierBadgeStyle, tierDescription } from '../utils/format'

const route = useRoute()
const store = useLibraryStore()
const { push } = useNotices()
const refreshing = shallowRef(false)

const subjectId = computed(() => Number(route.params.id))
const entry = computed(() =>
  Number.isFinite(subjectId.value) ? store.getEntry(subjectId.value) : null
)

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
</script>

<template>
  <div class="page">
    <nav class="breadcrumb" aria-label="面包屑">
      <RouterLink to="/library" class="back-link">
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
        <dl class="fact-list">
          <div class="fact">
            <dt>状态</dt>
            <dd>{{ statusLabels[entry.personal.status] }}</dd>
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
          <span>Bangumi 社区评分（Mock）：{{ formatScore(entry.subject.community.score) }}</span>
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
  color: var(--brand);
}

.missing-title {
  margin: 8px 0 0;
  font: 600 28px var(--font-display);
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
  border: 1px solid var(--border);
  box-shadow: var(--shadow-lift);
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
  padding: 28px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-card);
}

.detail-head {
  margin-bottom: 20px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
}

.title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 32px;
  line-height: 1.45;
  font-weight: 600;
  letter-spacing: 0.03em;
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
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-soft);
  margin-bottom: 24px;
}

.section-title {
  margin: 0 0 12px;
  font-size: 13px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 9px;
}

.section-title::before {
  content: '';
  width: 3px;
  height: 12px;
  background: var(--brand);
  border-radius: var(--radius-xs);
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
  font: 600 13px var(--font-number);
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
  padding: 15px 18px;
  border-left: 2px solid var(--brand);
  background: var(--surface-soft);
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  font-size: 13px;
  line-height: 1.95;
  color: var(--text-soft);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.review-empty {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
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
