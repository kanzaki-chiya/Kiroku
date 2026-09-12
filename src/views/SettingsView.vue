<script setup lang="ts">
import { shallowRef } from 'vue'
import TierManager from '../components/TierManager.vue'
import { useLibraryStore } from '../stores/library'
import { useNotices } from '../stores/notices'
import type { BackupDocument, ImportPreview } from '../types/anime'

const store = useLibraryStore()
const { push } = useNotices()
const busy = shallowRef(false)
const preview = shallowRef<ImportPreview | null>(null)
const pendingDocument = shallowRef<BackupDocument | null>(null)
const overwrite = shallowRef(false)

function downloadJson(name: string, data: unknown) {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = name
  link.click()
  URL.revokeObjectURL(url)
}

async function onExport() {
  busy.value = true
  try {
    const document = await store.exportBackup()
    downloadJson(`kiroku-backup-${document.exportedAt.slice(0, 10)}.json`, document)
    push('已导出备份')
  } catch (error) {
    push(error instanceof Error ? error.message : '导出失败')
  } finally {
    busy.value = false
  }
}

async function onSnapshot() {
  busy.value = true
  try {
    const path = await store.snapshotDatabase()
    push(`数据库快照已写入 ${path}`)
  } catch (error) {
    push(error instanceof Error ? error.message : '快照失败')
  } finally {
    busy.value = false
  }
}

async function onPickImport(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  busy.value = true
  try {
    const document = JSON.parse(await file.text()) as BackupDocument
    if (document.formatVersion !== 1 || !Array.isArray(document.entries)) {
      throw new Error('备份文件格式无效')
    }
    pendingDocument.value = document
    preview.value = await store.previewImport(document)
    overwrite.value = false
  } catch (error) {
    pendingDocument.value = null
    preview.value = null
    push(error instanceof Error ? error.message : '无法读取备份')
  } finally {
    busy.value = false
  }
}

async function onConfirmImport() {
  if (!pendingDocument.value) return
  if (overwrite.value && !window.confirm('将覆盖已有个人记录，确定继续？')) return
  busy.value = true
  try {
    const result = await store.importBackup(pendingDocument.value, overwrite.value)
    push(`导入完成：新增 ${result.added}，跳过/重复 ${result.duplicates}`)
    pendingDocument.value = null
    preview.value = null
  } catch (error) {
    push(error instanceof Error ? error.message : '导入失败，原有数据未改动')
  } finally {
    busy.value = false
  }
}

async function onClearCovers() {
  if (!window.confirm('只删除本地封面缓存，收藏和评分会保留。继续？')) return
  busy.value = true
  try {
    await store.clearCoverCache()
    push('已清理封面缓存')
  } catch (error) {
    push(error instanceof Error ? error.message : '清理失败')
  } finally {
    busy.value = false
  }
}

async function onDeleteData() {
  if (!window.confirm('将删除全部收藏、评分和封面，且不可恢复。确定？')) return
  if (!window.confirm('再次确认：个人数据会被清空。')) return
  busy.value = true
  try {
    await store.deletePersonalData()
    push('已删除个人数据')
  } catch (error) {
    push(error instanceof Error ? error.message : '删除失败')
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="page">
    <nav class="breadcrumb" aria-label="面包屑">
      <RouterLink to="/library">我的空间</RouterLink>
      <span class="sep">/</span>
      <span aria-current="page">备份与数据</span>
    </nav>

    <header class="page-head">
      <h1 class="page-title">备份与数据</h1>
      <p class="page-sub">导出导入个人收藏，管理分档，清理封面缓存。封面与收藏互不影响。</p>
    </header>

    <TierManager />

    <section class="panel">
      <h2 class="panel-title">JSON 备份</h2>
      <p class="panel-copy">包含个人记录、资料快照和分档设置，不含封面文件。</p>
      <div class="actions">
        <button type="button" class="btn btn-primary" :disabled="busy" @click="onExport">导出备份</button>
        <button type="button" class="btn btn-ghost" :disabled="busy || !store.desktop" @click="onSnapshot">
          导出数据库快照
        </button>
        <label class="btn btn-ghost file-btn">
          选择备份文件
          <input type="file" accept="application/json" class="sr-only" @change="onPickImport" />
        </label>
      </div>
      <div v-if="preview" class="preview">
        <p>新增 {{ preview.added }} · 已存在 {{ preview.duplicates }} · 内容冲突 {{ preview.conflicts }}</p>
        <label class="check">
          <input v-model="overwrite" type="checkbox" />
          <span>覆盖已有个人记录（默认跳过）</span>
        </label>
        <button type="button" class="btn btn-primary" :disabled="busy || !store.desktop" @click="onConfirmImport">
          确认导入
        </button>
        <p v-if="!store.desktop" class="hint">浏览器演示模式只预览，不写入。</p>
      </div>
    </section>

    <section class="panel">
      <h2 class="panel-title">封面缓存</h2>
      <p class="panel-copy">清理后列表会显示占位图，收藏和评分保留。</p>
      <button type="button" class="btn btn-ghost" :disabled="busy || !store.desktop" @click="onClearCovers">
        清理封面缓存
      </button>
    </section>

    <section class="panel danger">
      <h2 class="panel-title">删除个人数据</h2>
      <p class="panel-copy">清空收藏库。不会自动导出备份。</p>
      <button type="button" class="btn btn-danger" :disabled="busy || !store.desktop" @click="onDeleteData">
        删除全部收藏
      </button>
    </section>
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

.panel {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 22px 24px;
  margin-bottom: 16px;
}

.panel-title {
  margin: 0 0 8px;
  font-size: 18px;
}

.panel-copy,
.hint {
  margin: 0 0 16px;
  color: var(--muted);
  font-size: 14px;
}

.actions,
.preview {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.file-btn {
  cursor: pointer;
}

.check {
  display: inline-flex;
  gap: 8px;
  align-items: center;
  font-size: 14px;
}

.danger {
  border-color: color-mix(in srgb, #bd592e 35%, var(--border));
}

.btn-danger {
  background: #bd592e;
  color: #fff;
  border: none;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  border: 0;
}
</style>
