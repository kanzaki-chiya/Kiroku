<script setup lang="ts">
import { onMounted, shallowRef } from 'vue'
import TierManager from '../components/TierManager.vue'
import { themeMode, type ThemeMode } from '../services/theme'
import {
  syncDeleteCloudLibrary,
  syncListConflicts,
  syncLogin,
  syncLogout,
  syncNow,
  syncReconcile,
  syncRedeemCode,
  syncResolveConflict,
  syncSetEnabled,
  syncSignup,
  syncStatus,
  type SyncConflict,
  type SyncStatus
} from '../services/sync'
import { useLibraryStore } from '../stores/library'
import { useNotices } from '../stores/notices'
import type { BackupDocument, ImportPreview } from '../types/anime'

const store = useLibraryStore()
const { push } = useNotices()

const themeOptions: { value: ThemeMode; label: string }[] = [
  { value: 'system', label: '跟随系统' },
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' }
]
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
    if ((document.formatVersion !== 1 && document.formatVersion !== 2) || !Array.isArray(document.entries)) {
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

// ---------- 云同步 ----------

const sync = shallowRef<SyncStatus | null>(null)
const syncBusy = shallowRef(false)
const authTab = shallowRef<'login' | 'signup'>('login')
const loginEmail = shallowRef('')
const loginPassword = shallowRef('')
const signupPassword2 = shallowRef('')
const redeemCode = shallowRef('')
const conflicts = shallowRef<SyncConflict[]>([])

async function refreshSync() {
  if (!store.desktop) return
  try {
    sync.value = await syncStatus()
    conflicts.value = sync.value.conflictCount > 0 ? await syncListConflicts() : []
  } catch {
    sync.value = null
  }
}

async function withSyncBusy(task: () => Promise<SyncStatus | void>, ok?: string) {
  syncBusy.value = true
  try {
    const next = await task()
    if (next) sync.value = next
    await refreshSync()
    if (ok) push(ok)
  } catch (error) {
    push(error instanceof Error ? error.message : '操作失败')
  } finally {
    syncBusy.value = false
  }
}

function onLogin() {
  if (!loginEmail.value || !loginPassword.value) {
    push('请输入邮箱和密码')
    return
  }
  void withSyncBusy(() => syncLogin(loginEmail.value, loginPassword.value), '已登录')
}

function onSignup() {
  if (!loginEmail.value || !loginPassword.value) {
    push('请输入邮箱和密码')
    return
  }
  if (loginPassword.value !== signupPassword2.value) {
    push('两次输入的密码不一致')
    return
  }
  syncBusy.value = true
  syncSignup(loginEmail.value, loginPassword.value)
    .then(async (result) => {
      if (result.status === 'signed_in') {
        push('注册成功，已自动登录')
      } else {
        push('验证邮件已发送，请完成邮箱验证后登录')
        authTab.value = 'login'
      }
      loginPassword.value = ''
      signupPassword2.value = ''
      await refreshSync()
    })
    .catch((error) => push(error instanceof Error ? error.message : '注册失败'))
    .finally(() => {
      syncBusy.value = false
    })
}

function onAuthSubmit() {
  if (authTab.value === 'login') onLogin()
  else onSignup()
}

function onLogout() {
  void withSyncBusy(() => syncLogout(), '已退出登录')
}

function onToggleSync() {
  if (!sync.value) return
  void withSyncBusy(() => syncSetEnabled(!sync.value!.syncEnabled))
}

function onSyncNow() {
  void withSyncBusy(() => syncNow(), '同步完成')
}

function onRedeem() {
  const code = redeemCode.value.trim()
  if (!code) return
  syncBusy.value = true
  syncRedeemCode(code)
    .then(async (result) => {
      if (result.status === 'redeemed') push('兑换成功，会员已生效')
      else if (result.status === 'replayed') push('该请求已处理过，未重复兑换')
      else if (result.status === 'already_redeemed') push('该兑换码已被使用')
      else push(`兑换失败：${result.status}`)
      redeemCode.value = ''
      await refreshSync()
    })
    .catch((error) => push(error instanceof Error ? error.message : '兑换失败'))
    .finally(() => {
      syncBusy.value = false
    })
}

function onResolve(conflict: SyncConflict, keep: 'local' | 'remote') {
  void withSyncBusy(
    () => syncResolveConflict(conflict.entityType, conflict.entityKey, keep),
    keep === 'local' ? '已保留本地版本' : '已采用云端版本'
  )
}

function onReconcile(mode: 'rebuild' | 'overwrite') {
  const tip =
    mode === 'rebuild'
      ? '将把本地全部数据重新上传到云端。继续？'
      : '将清空本地收藏并以云端数据为准（会先自动备份一份数据库）。继续？'
  if (!window.confirm(tip)) return
  void withSyncBusy(() => syncReconcile(mode), '对账完成')
}

function onDeleteCloud() {
  if (!window.confirm('将删除云端全部收藏数据（本地数据保留）。此操作不可撤销，确定？')) return
  void withSyncBusy(() => syncDeleteCloudLibrary(), '云端数据已删除')
}

onMounted(refreshSync)
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

    <section class="panel">
      <h2 class="panel-title">外观</h2>
      <p class="panel-copy">选择应用主题。跟随系统时随 Windows 明暗模式自动切换。</p>
      <div class="segmented" role="group" aria-label="外观主题">
        <button
          v-for="opt in themeOptions"
          :key="opt.value"
          type="button"
          class="seg-btn"
          :class="{ 'is-active': themeMode === opt.value }"
          :aria-pressed="themeMode === opt.value"
          @click="themeMode = opt.value"
        >
          {{ opt.label }}
        </button>
      </div>
    </section>

    <TierManager />

    <section class="panel">
      <h2 class="panel-title">JSON 备份</h2>
      <p class="panel-copy">包含个人记录、资料快照和分档设置，不含封面文件。备份不向前兼容旧版应用。</p>
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

    <section v-if="store.desktop" class="panel">
      <h2 class="panel-title">云同步</h2>
      <p class="panel-copy">可选功能：登录并开启后，收藏、评分与分档会在设备间同步。不登录时所有功能照常本地使用。</p>

      <template v-if="!sync?.loggedIn">
        <div class="segmented auth-tabs" role="group" aria-label="账号操作">
          <button
            type="button"
            class="seg-btn"
            :class="{ 'is-active': authTab === 'login' }"
            :aria-pressed="authTab === 'login'"
            @click="authTab = 'login'"
          >
            登录
          </button>
          <button
            type="button"
            class="seg-btn"
            :class="{ 'is-active': authTab === 'signup' }"
            :aria-pressed="authTab === 'signup'"
            @click="authTab = 'signup'"
          >
            注册
          </button>
        </div>
        <div class="sync-login">
          <input v-model="loginEmail" type="email" class="input" placeholder="邮箱" autocomplete="email" />
          <input
            v-model="loginPassword"
            type="password"
            class="input"
            placeholder="密码（至少 6 位）"
            :autocomplete="authTab === 'login' ? 'current-password' : 'new-password'"
            @keyup.enter="onAuthSubmit"
          />
          <input
            v-if="authTab === 'signup'"
            v-model="signupPassword2"
            type="password"
            class="input"
            placeholder="再次输入密码"
            autocomplete="new-password"
            @keyup.enter="onAuthSubmit"
          />
          <button type="button" class="btn btn-primary" :disabled="syncBusy" @click="onAuthSubmit">
            {{ authTab === 'login' ? '登录' : '注册' }}
          </button>
        </div>
        <p v-if="authTab === 'signup'" class="hint">
          注册后如收到验证邮件，完成验证再登录；登录不代表已开启同步。
        </p>
      </template>

      <template v-else>
        <div class="sync-status">
          <p class="sync-line">已登录 {{ sync.email }}</p>
          <p v-if="sync.memberActive === true" class="sync-line">
            会员有效<span v-if="sync.expiresAt"> · 至 {{ sync.expiresAt.slice(0, 10) }}</span>
          </p>
          <p v-else-if="sync.memberActive === false" class="sync-line warn">
            {{ sync.inRetention ? '会员已到期（云端数据保留期内可拉取，续期后恢复上传）' : '会员未激活' }}
          </p>
          <p class="sync-line">
            待上传 {{ sync.pendingOps }} · 冲突 {{ sync.conflictCount }}
            <span v-if="sync.nextRetryAt"> · 下次重试 {{ new Date(sync.nextRetryAt).toLocaleTimeString() }}</span>
            <span v-if="sync.lastSyncAt"> · 上次同步 {{ new Date(sync.lastSyncAt).toLocaleString() }}</span>
          </p>
          <p v-if="sync.lastError" class="sync-line warn">{{ sync.lastError }}</p>
        </div>

        <div v-if="sync.reconcileRequired" class="reconcile">
          <p class="sync-line warn">本地与云端状态不一致，需要选择处理方式：</p>
          <div class="actions">
            <button type="button" class="btn btn-ghost" :disabled="syncBusy" @click="onReconcile('rebuild')">
              以本地为准重建云端
            </button>
            <button type="button" class="btn btn-ghost" :disabled="syncBusy" @click="onReconcile('overwrite')">
              以云端为准覆盖本地（先备份）
            </button>
          </div>
        </div>

        <div v-if="conflicts.length" class="conflicts">
          <p class="sync-line warn">以下条目本地与云端都有修改，请选择保留哪一边：</p>
          <div v-for="c in conflicts" :key="`${c.entityType}:${c.entityKey}`" class="conflict-row">
            <span class="conflict-name">{{ c.entityType === 'record' ? `番剧 #${c.entityKey}` : c.entityKey }}</span>
            <button type="button" class="btn btn-ghost" :disabled="syncBusy" @click="onResolve(c, 'local')">
              保留本地
            </button>
            <button type="button" class="btn btn-ghost" :disabled="syncBusy" @click="onResolve(c, 'remote')">
              采用云端
            </button>
          </div>
        </div>

        <div class="actions">
          <button type="button" class="btn btn-primary" :disabled="syncBusy || sync.reconcileRequired" @click="onToggleSync">
            {{ sync.syncEnabled ? '关闭同步' : '开启同步' }}
          </button>
          <button
            type="button"
            class="btn btn-ghost"
            :disabled="syncBusy || !sync.syncEnabled || sync.reconcileRequired"
            @click="onSyncNow"
          >
            立即同步
          </button>
          <button type="button" class="btn btn-ghost" :disabled="syncBusy" @click="onLogout">退出登录</button>
        </div>

        <div class="redeem">
          <input v-model="redeemCode" class="input" placeholder="会员兑换码" @keyup.enter="onRedeem" />
          <button type="button" class="btn btn-ghost" :disabled="syncBusy || !redeemCode.trim()" @click="onRedeem">
            兑换
          </button>
        </div>

        <div class="actions">
          <button type="button" class="btn btn-ghost danger-link" :disabled="syncBusy" @click="onDeleteCloud">
            删除云端数据
          </button>
        </div>
      </template>
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

.panel {
  background: var(--surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  padding: 26px;
  margin-bottom: 20px;
}

.panel-title {
  margin: 0 0 8px;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.panel-copy,
.hint {
  margin: 0 0 16px;
  color: var(--muted);
  font-size: 13px;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.preview {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  width: 100%;
  padding: 16px;
  background: var(--surface-soft);
  border-radius: var(--radius-md);
  margin-top: 12px;
}

.file-btn {
  cursor: pointer;
}

.segmented {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  background: var(--fill);
  border-radius: var(--radius-md);
}

.seg-btn {
  padding: 6px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-soft);
  border-radius: 10px;
  transition: background var(--motion-fast) var(--ease-snap),
    color var(--motion-fast) var(--ease-snap),
    box-shadow var(--motion-fast) var(--ease-snap),
    transform 100ms ease-out;
}

.seg-btn:active {
  transform: scale(0.97);
}

.seg-btn:hover {
  color: var(--text);
}

.seg-btn.is-active {
  color: var(--text);
  font-weight: 600;
  background: var(--surface);
  box-shadow: var(--shadow-thumb);
}

.check {
  display: inline-flex;
  gap: 8px;
  align-items: center;
  font-size: 14px;
}

.auth-tabs {
  margin-bottom: 14px;
}

.sync-login,
.redeem {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
  margin-bottom: 14px;
}

.input {
  flex: 1;
  min-width: 160px;
  padding: 8px 12px;
  font-size: 14px;
  color: var(--text);
  background: var(--surface-soft);
  border: 1px solid var(--fill);
  border-radius: var(--radius-md);
}

.input:focus {
  outline: none;
  border-color: var(--text-soft);
}

.sync-status {
  margin-bottom: 14px;
}

.sync-line {
  margin: 2px 0;
  font-size: 13px;
  color: var(--muted);
}

.sync-line.warn {
  color: var(--danger);
}

.reconcile,
.conflicts {
  margin-bottom: 14px;
  padding: 14px;
  background: var(--surface-soft);
  border-radius: var(--radius-md);
}

.conflict-row {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 6px 0;
}

.conflict-name {
  flex: 1;
  font-size: 13px;
}

.danger-link {
  color: var(--danger);
}

.danger {
  background: var(--danger-soft);
  box-shadow: none;
}

.btn-danger {
  background: var(--danger);
  color: var(--on-brand);
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

@media (max-width: 1050px) {
  .panel {
    padding: 18px;
  }
}
</style>
