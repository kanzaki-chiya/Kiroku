<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import RatingForm from '../components/RatingForm.vue'
import { useUnsavedGuard } from '../composables/useUnsavedGuard'
import { useLibraryStore } from '../stores/library'
import { useNotices } from '../stores/notices'
import type { PersonalDraft } from '../types/anime'
import { cloneDraft } from '../utils/draft'

const route = useRoute()
const router = useRouter()
const store = useLibraryStore()
const { push } = useNotices()

const subjectId = computed(() => Number(route.params.id))
const entry = computed(() =>
  Number.isFinite(subjectId.value) ? store.getEntry(subjectId.value) : null
)

const dirty = ref(false)
useUnsavedGuard(dirty)

const initial = computed<PersonalDraft | null>(() =>
  entry.value ? cloneDraft(entry.value.personal) : null
)

async function onSave(draft: PersonalDraft) {
  if (!entry.value) return
  try {
    await store.update(entry.value.subject.id, draft)
  } catch (err) {
    push(err instanceof Error ? err.message : '保存失败')
    return
  }
  dirty.value = false
  push('已保存修改')
  router.push(`/anime/${entry.value.subject.id}`)
}

function onCancel() {
  if (dirty.value && !window.confirm('有未保存的修改，确定要离开吗？')) return
  dirty.value = false
  router.push(entry.value ? `/anime/${entry.value.subject.id}` : '/library')
}
</script>

<template>
  <div class="page">
    <nav class="breadcrumb" aria-label="面包屑">
      <RouterLink to="/library">番剧库</RouterLink>
      <span class="sep">/</span>
      <RouterLink v-if="entry" :to="`/anime/${entry.subject.id}`">{{ entry.subject.nameCn }}</RouterLink>
      <span v-if="entry" class="sep">/</span>
      <span aria-current="page">编辑</span>
    </nav>

    <template v-if="entry && initial">
      <header class="page-head">
        <h1 class="page-title">编辑「{{ entry.subject.nameCn }}」</h1>
      </header>
      <div class="form-wrap">
        <RatingForm
          :key="subjectId"
          :initial="initial"
          :tiers="store.tiers"
          :busy="store.saving"
          submit-label="保存修改"
          @save="onSave"
          @cancel="onCancel"
          @dirty-change="dirty = $event"
        />
      </div>
    </template>

    <div v-else class="missing">
      <h1 class="missing-title">没有找到这部番剧</h1>
      <p class="missing-sub">它还没有被收录，无法编辑。</p>
      <RouterLink to="/library" class="btn btn-primary">回到番剧库</RouterLink>
    </div>
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

.form-wrap {
  max-width: 800px;
}

.missing {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 90px 20px;
  text-align: center;
}

.missing-title {
  margin: 8px 0 0;
  font: 700 24px/1.25 var(--font-display);
  letter-spacing: -0.02em;
}

.missing-sub {
  margin: 0 0 16px;
  color: var(--muted);
  font-size: 13px;
}
</style>
