import { allSubjects } from '../data/subjects'
import { isTauri } from '../runtime'
import { BackendError } from '../services/errors'
import { invokeCmd } from '../services/tauri'
import type { BangumiSubject } from '../types/anime'

export interface BangumiApi {
  searchSubjects(query: string, signal?: AbortSignal): Promise<BangumiSubject[]>
  getSubject(id: number, signal?: AbortSignal): Promise<BangumiSubject | null>
  getSuggestions(signal?: AbortSignal): Promise<BangumiSubject[]>
}

const SEARCH_DELAY = 350
const DETAIL_DELAY = 180
const SUGGEST_DELAY = 200

function delay(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(new DOMException('Aborted', 'AbortError'))
      return
    }
    const timer = setTimeout(() => {
      signal?.removeEventListener('abort', onAbort)
      resolve()
    }, ms)
    const onAbort = () => {
      clearTimeout(timer)
      reject(new DOMException('Aborted', 'AbortError'))
    }
    signal?.addEventListener('abort', onAbort)
  })
}

function cloneSubject(subject: BangumiSubject): BangumiSubject {
  return {
    ...subject,
    tags: [...subject.tags],
    aliases: subject.aliases ? [...subject.aliases] : undefined,
    community: { ...subject.community }
  }
}

const mockBangumiApi: BangumiApi = {
  async searchSubjects(query, signal) {
    await delay(SEARCH_DELAY, signal)
    const normalized = query.trim().toLowerCase()
    if (!normalized) return []
    return allSubjects
      .filter(subject => {
        const haystack = [
          subject.nameCn,
          subject.name,
          String(subject.year),
          subject.studio,
          ...(subject.aliases ?? []),
          ...subject.tags
        ]
          .join('\n')
          .toLowerCase()
        return haystack.includes(normalized)
      })
      .map(cloneSubject)
  },

  async getSubject(id, signal) {
    await delay(DETAIL_DELAY, signal)
    const found = allSubjects.find(subject => subject.id === id)
    return found ? cloneSubject(found) : null
  },

  async getSuggestions(signal) {
    await delay(SUGGEST_DELAY, signal)
    return allSubjects.map(cloneSubject)
  }
}

const tauriBangumiApi: BangumiApi = {
  async searchSubjects(query, signal) {
    if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')
    const results = await invokeCmd<BangumiSubject[]>('search_subjects', { query })
    if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')
    return results
  },
  async getSubject(id, signal) {
    if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')
    try {
      const subject = await invokeCmd<BangumiSubject>('get_subject', { bangumiSubjectId: id })
      if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')
      return subject
    } catch (error) {
      if (error instanceof BackendError && error.code === 'NOT_FOUND') return null
      throw error
    }
  },
  async getSuggestions(signal) {
    if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')
    return invokeCmd<BangumiSubject[]>('list_recent_searches')
  }
}

export const bangumiApi: BangumiApi = isTauri() ? tauriBangumiApi : mockBangumiApi
