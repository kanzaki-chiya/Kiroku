export class BackendError extends Error {
  code: string

  constructor(code: string, message: string) {
    super(message)
    this.name = 'BackendError'
    this.code = code
  }
}

export function toBackendError(error: unknown): BackendError {
  if (error instanceof BackendError) return error
  if (error instanceof DOMException && error.name === 'AbortError') return error as unknown as BackendError
  if (typeof error === 'string') {
    try {
      return toBackendError(JSON.parse(error))
    } catch {
      return new BackendError('INTERNAL', error)
    }
  }
  if (error && typeof error === 'object') {
    const record = error as { code?: unknown; message?: unknown; error?: unknown }
    if (typeof record.code === 'string' && typeof record.message === 'string') {
      return new BackendError(record.code, record.message)
    }
    if (typeof record.message === 'string') {
      return new BackendError('INTERNAL', record.message)
    }
  }
  return new BackendError('INTERNAL', '操作失败')
}
