import { invokeCmd } from './tauri'

export interface SyncStatus {
  loggedIn: boolean
  email: string | null
  accountId: string | null
  syncEnabled: boolean
  memberActive: boolean | null
  expiresAt: string | null
  inRetention: boolean | null
  pendingOps: number
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
