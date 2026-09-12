<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import SubjectSearch from '../components/SubjectSearch.vue'
import { useUnsavedGuard } from '../composables/useUnsavedGuard'
import { useNotices } from '../stores/notices'

const router = useRouter()
const { push } = useNotices()
const dirty = ref(false)

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

    <SubjectSearch @saved="onSaved" @dirty="dirty = $event" />
  </div>
</template>

<style scoped>
.page-title {
  margin: 0;
  font-size: 28px;
  font-weight: 700;
}

.page-sub {
  margin: 6px 0 26px;
  font-size: 14px;
  color: var(--muted);
}
</style>
