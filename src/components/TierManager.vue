<script setup lang="ts">
import { ref, shallowRef, watch } from 'vue'
import { ArrowDown, ArrowUp, Plus, Trash2 } from 'lucide-vue-next'
import { BackendError } from '../services/errors'
import { useLibraryStore } from '../stores/library'
import { useNotices } from '../stores/notices'
import type { TierDefinition } from '../types/anime'
import { tierBadgeStyle } from '../utils/format'

const store = useLibraryStore()
const { push } = useNotices()
const busy = shallowRef(false)
const rows = ref<TierDefinition[]>([])
const draftName = shallowRef('')
const draftDescription = shallowRef('')
const draftColor = shallowRef('#6464a6')

watch(
  () => store.tiers,
  value => {
    rows.value = value.map(tier => ({ ...tier }))
  },
  { immediate: true, deep: true }
)

async function run(action: () => Promise<void>) {
  if (busy.value) return
  busy.value = true
  try {
    await action()
  } catch (error) {
    push(error instanceof Error ? error.message : '操作失败')
  } finally {
    busy.value = false
  }
}

function referenced(id: number) {
  const saved = store.tiers.find(tier => tier.id === id)
  if (!saved) return false
  return store.entries.some(entry => entry.personal.tier === saved.name)
}

async function onSaveExisting(row: TierDefinition) {
  await run(async () => {
    await store.saveTier({
      id: row.id,
      name: row.name,
      description: row.description,
      color: row.color
    })
    push(store.desktop ? '已保存分档' : '已更新分档（演示模式不持久化）')
  })
}

async function onCreate() {
  await run(async () => {
    await store.saveTier({
      name: draftName.value,
      description: draftDescription.value,
      color: draftColor.value
    })
    draftName.value = ''
    draftDescription.value = ''
    draftColor.value = '#6464a6'
    push(store.desktop ? '已新增分档' : '已新增分档（演示模式不持久化）')
  })
}

async function onMove(index: number, delta: number) {
  const next = index + delta
  if (next < 0 || next >= store.tiers.length) return
  const ids = store.tiers.map(tier => tier.id)
  const [moved] = ids.splice(index, 1)
  ids.splice(next, 0, moved)
  await run(async () => {
    await store.reorderTiers(ids)
  })
}

async function onDelete(id: number) {
  await run(async () => {
    try {
      await store.deleteTier(id)
      push('已删除分档')
    } catch (error) {
      if (error instanceof BackendError && error.code === 'CONFLICT') {
        push(error.message)
        return
      }
      throw error
    }
  })
}
</script>

<template>
  <section class="panel">
    <h2 class="panel-title">分档管理</h2>
    <p class="panel-copy">
      内置分档和仍被收藏引用的分档不能删除。
      <span v-if="!store.desktop">浏览器演示只改本次内存，刷新后恢复。</span>
    </p>

    <ul class="tier-list">
      <li v-for="(row, index) in rows" :key="row.id" class="tier-row">
        <span class="tier-badge" :style="tierBadgeStyle(row.color)">{{ row.name }}</span>
        <input v-model="row.name" class="field" aria-label="分档名称" />
        <input v-model="row.description" class="field grow" aria-label="分档说明" />
        <input v-model="row.color" class="color" type="color" aria-label="分档颜色" />
        <button type="button" class="btn btn-ghost" :disabled="busy" @click="onSaveExisting(row)">
          保存
        </button>
        <button type="button" class="icon-btn" :disabled="busy || index === 0" aria-label="上移" @click="onMove(index, -1)">
          <ArrowUp :size="14" />
        </button>
        <button
          type="button"
          class="icon-btn"
          :disabled="busy || index === rows.length - 1"
          aria-label="下移"
          @click="onMove(index, 1)"
        >
          <ArrowDown :size="14" />
        </button>
        <button
          type="button"
          class="icon-btn danger"
          :disabled="busy || row.builtin || referenced(row.id)"
          :title="row.builtin ? '内置分档不能删除' : referenced(row.id) ? '仍被收藏引用' : '删除'"
          aria-label="删除"
          @click="onDelete(row.id)"
        >
          <Trash2 :size="14" />
        </button>
      </li>
    </ul>

    <form class="create-row" @submit.prevent="onCreate">
      <Plus :size="14" aria-hidden="true" />
      <input v-model="draftName" class="field" placeholder="新分档名称" required />
      <input v-model="draftDescription" class="field grow" placeholder="说明" />
      <input v-model="draftColor" class="color" type="color" aria-label="新分档颜色" />
      <button type="submit" class="btn btn-primary" :disabled="busy">新增</button>
    </form>
  </section>
</template>

<style scoped>
.panel {
  background: var(--surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: 24px;
  margin-bottom: 20px;
}

.panel-title {
  margin: 0 0 8px;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.panel-copy {
  margin: 0 0 16px;
  color: var(--muted);
  font-size: 13px;
}

.tier-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}

.tier-row {
  display: grid;
  grid-template-columns: 28px 76px minmax(100px, 1fr) 36px auto 32px 32px 32px;
  gap: 8px;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid var(--border);
  min-width: 0;
}

.create-row {
  display: grid;
  grid-template-columns: 28px 76px minmax(100px, 1fr) 36px auto;
  gap: 8px;
  align-items: center;
  margin-top: 14px;
  min-width: 0;
}

.field {
  width: 100%;
  min-width: 0;
  height: 34px;
  padding: 6px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--fill);
  transition: background var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap);
}

.field:focus {
  background: var(--surface);
  outline: none;
  box-shadow: var(--focus-ring), inset 0 0 0 1px var(--brand);
}

.grow {
  min-width: 0;
}

.color {
  width: 36px;
  height: 32px;
  padding: 0;
  border: none;
  border-radius: var(--radius-xs);
  background: transparent;
}

.icon-btn {
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  background: var(--fill);
  transition: background var(--motion-fast) var(--ease-snap),
    transform 100ms ease-out;
}

.icon-btn:hover:not(:disabled) {
  background: var(--fill-strong);
}

.icon-btn:active:not(:disabled) {
  transform: scale(0.94);
}

.icon-btn:disabled {
  opacity: 0.4;
}

.danger {
  color: var(--danger);
}

@media (max-width: 1050px) {
  .panel {
    padding: 18px;
  }

  .tier-row {
    grid-template-columns: 26px 64px minmax(72px, 1fr) 32px auto 28px 28px 28px;
    gap: 5px;
  }

  .create-row {
    grid-template-columns: 26px 64px minmax(72px, 1fr) 32px auto;
    gap: 5px;
  }

  .icon-btn {
    width: 28px;
    height: 32px;
  }

  .color {
    width: 32px;
    height: 32px;
  }

  .field {
    padding: 4px 6px;
  }
}
</style>
