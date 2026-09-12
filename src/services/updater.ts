import { ref } from 'vue'
import { isTauri } from '../runtime'

export interface PendingUpdate {
  version: string
  notes: string
  downloadAndInstall: (onProgress?: (percent: number | null) => void) => Promise<void>
}

export const pendingUpdate = ref<PendingUpdate | null>(null)
export const updateProgress = ref<number | null>(null)
export const updateInstalling = ref(false)
export const updateFailed = ref(false)

export async function checkForUpdates() {
  if (!isTauri()) return
  try {
    const { check } = await import('@tauri-apps/plugin-updater')
    const update = await check()
    if (!update) return
    pendingUpdate.value = {
      version: update.version,
      notes: update.body ?? '',
      downloadAndInstall: async onProgress => {
        let total = 0
        let received = 0
        await update.downloadAndInstall(event => {
          if (event.event === 'Started') {
            total = event.data.contentLength ?? 0
          } else if (event.event === 'Progress') {
            received += event.data.chunkLength
            onProgress?.(total > 0 ? Math.round((received / total) * 100) : null)
          } else {
            onProgress?.(100)
          }
        })
      }
    }
  } catch {
    // 无网络或未发布 Release 时静默忽略
  }
}

export function dismissUpdate() {
  if (updateInstalling.value) return
  pendingUpdate.value = null
  updateFailed.value = false
}

export async function installUpdate() {
  const update = pendingUpdate.value
  if (!update || updateInstalling.value) return
  updateInstalling.value = true
  updateFailed.value = false
  updateProgress.value = null
  try {
    await update.downloadAndInstall(percent => {
      updateProgress.value = percent
    })
    const { relaunch } = await import('@tauri-apps/plugin-process')
    await relaunch()
  } catch {
    updateInstalling.value = false
    updateFailed.value = true
    updateProgress.value = null
  }
}
