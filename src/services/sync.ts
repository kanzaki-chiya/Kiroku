import { invokeCmd } from './tauri'
import { isTauri } from '../runtime'

export interface SyncStatus {
  loggedIn: boolean
  email: string | null
  accountId: string | null
  syncEnabled: boolean
  memberActive: boolean | null
  expiresAt: string | null
  inRetention: boolean | null
  pendingOps: number
  nextRetryAt: string | null
  conflictCount: number
  epoch: number
  cursor: number
  reconcileRequired: boolean
  lastSyncAt: string | null
  lastError: string | null
}

export interface SyncConflict {
  entityType: string
  entityKey: string
  localPayload: unknown
  remotePayload: unknown
  remoteServerVersion: number | null
  detectedAt: string
}

export interface RedeemResult {
  status: string
  expires_at?: string
}

export function syncLogin(email: string, password: string) {
  return invokeCmd<SyncStatus>('sync_login', { payload: { email, password } })
}

export interface SignupResult {
  status: 'signed_in' | 'confirm_email' | string
}

export function syncSignup(email: string, password: string) {
  return invokeCmd<SignupResult>('sync_signup', { payload: { email, password } })
}

export function syncLogout() {
  return invokeCmd<SyncStatus>('sync_logout')
}

export function syncStatus() {
  return invokeCmd<SyncStatus>('sync_status')
}

export function syncSetEnabled(enabled: boolean) {
  return invokeCmd<SyncStatus>('sync_set_enabled', { enabled })
}

export function syncNow() {
  return invokeCmd<SyncStatus>('sync_now')
}

/** 轻量唤起同步 worker（网络恢复等事件触发），不等待结果。 */
export function syncKick() {
  return invokeCmd<void>('sync_kick')
}

/** 网络恢复时唤起一轮同步（SYNC_DESIGN §9 事件触发点）。仅桌面端挂载。 */
export function initSyncTriggers() {
  if (!isTauri()) return
  window.addEventListener('online', () => {
    void syncKick().catch(() => {})
  })
}

export function syncRedeemCode(code: string) {
  return invokeCmd<RedeemResult>('sync_redeem_code', { code })
}

export function syncListConflicts() {
  return invokeCmd<SyncConflict[]>('sync_list_conflicts')
}

export function syncResolveConflict(entityType: string, entityKey: string, keep: 'local' | 'remote') {
  return invokeCmd<SyncStatus>('sync_resolve_conflict', {
    payload: { entityType, entityKey, keep }
  })
}

export function syncReconcile(mode: 'rebuild' | 'overwrite') {
  return invokeCmd<SyncStatus>('sync_reconcile', { mode })
}

export function syncDeleteCloudLibrary() {
  return invokeCmd<SyncStatus>('sync_delete_cloud_library', { confirm: 'DELETE' })
}
