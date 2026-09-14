<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import SubjectSearch from '../components/SubjectSearch.vue'
import { useUnsavedGuard } from '../composables/useUnsavedGuard'
import { useNotices } from '../stores/notices'
import type { WatchStatus } from '../types/anime'

const route = useRoute()
const router = useRouter()
const { push } = useNotices()
const dirty = ref(false)

const prefillId = computed(() => {
  const raw = Number(route.query.subject)
  return Number.isInteger(raw) && raw > 0 ? raw : undefined
})

const prefillStatus = computed<WatchStatus | undefined>(() => {
  const raw = route.query.status
  return raw === 'planned' || raw === 'watching' || raw === 'completed' ? raw : undefined
})

useUnsavedGuard(dirty)

function onSaved(subjectId: number) {
  dirty.value = false
  push('已收录到我的番剧库')
  router.push(`/anime/${subjectId}`)
}
</script>

<template>
  <div class="page">
    <nav class="breadcrumb" aria-label="面包屑">
      <RouterLink to="/library">番剧库</RouterLink>
      <span class="sep">/</span>
      <span aria-current="page">收录番剧</span>
    </nav>

    <header class="page-head">
      <h1 class="page-title">收录一部番剧</h1>
      <p class="page-sub">搜索 Bangumi 作品资料，再写下自己的评分。个人记录只保存在本地。</p>
    </header>

    <SubjectSearch
      :prefill-id="prefillId"
      :prefill-status="prefillStatus"
      @saved="onSaved"
      @dirty="dirty = $event"
    />
  </div>
</template>

<style scoped>
.page-title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 34px;
  line-height: 1.15;
  font-weight: 700;
  letter-spacing: -0.025em;
}

.page-sub {
  margin: 8px 0 26px;
  font-size: 13px;
  color: var(--muted);
}
</style>
