<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft, Ghost, Pencil, RotateCcw } from 'lucide-vue-next'
import AnimeCover from '../components/AnimeCover.vue'
import ScoreComparison from '../components/ScoreComparison.vue'
import { useLibraryStore } from '../stores/library'
import { useNotices } from '../stores/notices'
import { dimensionKeys } from '../types/anime'
import { dimensionLabels, formatLabels, formatScore, resolveTier, statusLabels, tierBadgeStyle, tierDescription } from '../utils/format'

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
            <div class="rail" role="img"
              :aria-label="entry.personal.dimensions[key] === null
                ? `${dimensionLabels[key]}未评分`
                : `${dimensionLabels[key]} ${entry.personal.dimensions[key]} 分`">
              <div
                v-if="entry.personal.dimensions[key] !== null"
                class="rail-fill"
                :style="{ width: `${(entry.personal.dimensions[key]! / 10) * 100}%` }"
              ></div>
            </div>
            <span class="dim-value" :class="{ muted: entry.personal.dimensions[key] === null }">
              {{ entry.personal.dimensions[key] === null ? '未评分' : entry.personal.dimensions[key]!.toFixed(1) }}
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
  gap: 8px;
  padding: 90px 20px;
  text-align: center;
}

.missing-icon {
  color: var(--muted);
}

.missing-title {
  margin: 8px 0 0;
  font-size: 22px;
}

.missing-sub {
  margin: 0 0 16px;
  color: var(--muted);
  font-size: 14px;
}

.detail {
  display: grid;
  grid-template-columns: 210px minmax(0, 1fr);
  gap: 38px;
  max-width: 980px;
}

.poster-col {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.poster {
  box-shadow: var(--shadow-card);
}

.edit-btn {
  justify-content: center;
}

.fact-list {
  margin: 4px 0 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.fact {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13.5px;
}

.fact dt {
  color: var(--muted);
  flex-shrink: 0;
}

.fact dd {
  margin: 0;
  text-align: right;
}

.tier-line {
  display: inline-flex;
  align-items: center;
  gap: 7px;
}

.muted {
  color: var(--muted);
}

.detail-head {
  margin-bottom: 16px;
}

.title {
  margin: 0;
  font-size: 30px;
  font-weight: 700;
}

.original {
  margin: 4px 0 0;
  font-size: 14px;
  color: var(--muted);
}

.meta {
  margin: 8px 0 0;
  font-size: 13.5px;
  color: var(--text-soft);
}

.tags {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
  margin-top: 12px;
}

.synopsis {
  margin: 0 0 18px;
  font-size: 14.5px;
  line-height: 1.85;
  color: var(--text-soft);
  max-width: 640px;
}

.community-line {
  font-size: 13px;
  color: var(--muted);
  margin-bottom: 20px;
}

.compare-block {
  padding: 18px 22px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  margin-bottom: 26px;
}

.section-title {
  margin: 0 0 12px;
  font-size: 15px;
  font-weight: 700;
}

.dims {
  margin-bottom: 26px;
  max-width: 560px;
}

.dim-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 5px 0;
}

.dim-label {
  width: 40px;
  font-size: 13.5px;
  color: var(--text-soft);
  flex-shrink: 0;
}

.rail {
  flex: 1;
  height: 7px;
  border-radius: 999px;
  background: var(--sage-tint);
  overflow: hidden;
}

.rail-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--accent);
}

.dim-value {
  width: 44px;
  text-align: right;
  font-size: 13.5px;
  font-weight: 600;
  flex-shrink: 0;
}

.dim-value.muted {
  font-weight: 400;
  color: var(--muted);
}

.review-block {
  max-width: 640px;
}

.review-text {
  margin: 0;
  font-size: 14.5px;
  line-height: 1.9;
  color: var(--text-soft);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.review-empty {
  margin: 0;
  font-size: 13.5px;
  color: var(--muted);
}

@media (max-width: 800px) {
  .detail {
    grid-template-columns: 1fr;
    gap: 26px;
  }

  .poster-col {
    max-width: 200px;
  }
}
</style>
