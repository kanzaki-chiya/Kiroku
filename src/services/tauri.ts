import { invoke } from '@tauri-apps/api/core'
import { toBackendError } from './errors'

export async function invokeCmd<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw toBackendError(error)
  }
}
