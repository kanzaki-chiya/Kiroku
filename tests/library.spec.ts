import { describe, expect, it } from 'vitest'
import { createDefaultFilters, filterLibrary } from '../src/utils/library'
import type { BangumiSubject, LibraryEntry, LibraryFilters, Tier, WatchStatus } from '../src/types/anime'
import { emptyDimensions } from '../src/utils/draft'

function makeEntry(options: {
  id: number
  nameCn: string
  name?: string
  tags?: string[]
  year?: number
  score?: number | null
  community?: number | null
  tier?: Tier | null
  status?: WatchStatus
  updatedAt?: string
}): LibraryEntry {
  const subject: BangumiSubject = {
    id: options.id,
    name: options.name ?? `Name ${options.id}`,
    nameCn: options.nameCn,
    summary: '',
    coverUrl: '',
    year: options.year ?? 2020,
    format: 'TV',
    episodes: 12,
    studio: 'Studio',
    tags: options.tags ?? [],
    community: { score: options.community ?? null, votes: 0, rank: null }
  }
  return {
    subject,
    personal: {
      score: options.score ?? null,
      tier: options.tier ?? null,
      status: options.status ?? 'completed',
      dimensions: emptyDimensions(),
      review: '',
      subjectId: options.id,
      createdAt: '2025-01-01T00:00:00.000Z',
      updatedAt: options.updatedAt ?? '2025-01-01T00:00:00.000Z',
      version: 1
    }
  }
}

function filters(partial: Partial<LibraryFilters>): LibraryFilters {
  return { ...createDefaultFilters(), ...partial }
}

describe('filterLibrary', () => {
  const entries = [
    makeEntry({ id: 1, nameCn: '葬送的芙莉莲', name: 'Sousou no Frieren', tags: ['奇幻'], year: 2023, score: 9.6, tier: 'S', status: 'completed', updatedAt: '2025-06-14T00:00:00.000Z' }),
    makeEntry({ id: 2, nameCn: '孤独摇滚！', name: 'Bocchi the Rock!', tags: ['音乐'], year: 2022, score: 8.8, tier: 'A', status: 'completed', updatedAt: '2025-03-22T00:00:00.000Z' }),
    makeEntry({ id: 3, nameCn: '迷宫饭', name: 'Dungeon Meshi', tags: ['美食'], year: 2024, score: null, tier: null, status: 'watching', updatedAt: '2025-06-01T00:00:00.000Z' })
  ]

  it('matches a trimmed query against Chinese names', () => {
    const result = filterLibrary(entries, filters({ query: '  芙莉莲 ' }))
    expect(result.map(e => e.subject.id)).toEqual([1])
  })

  it('matches a query against original names case-insensitively', () => {
    const result = filterLibrary(entries, filters({ query: 'bocchi' }))
    expect(result.map(e => e.subject.id)).toEqual([2])
  })

  it('matches a query against tags', () => {
    const result = filterLibrary(entries, filters({ query: '美食' }))
    expect(result.map(e => e.subject.id)).toEqual([3])
  })

  it('intersects tier, status and year filters', () => {
    const base = filters({ tier: 'A', status: 'completed', year: 2022 })
    expect(filterLibrary(entries, base).map(e => e.subject.id)).toEqual([2])
    expect(filterLibrary(entries, filters({ tier: 'A', status: 'watching', year: 2022 }))).toEqual([])
    expect(filterLibrary(entries, filters({ tier: 'A', status: 'completed', year: 2023 }))).toEqual([])
  })

  it('supports the unassigned tier filter', () => {
    const result = filterLibrary(entries, filters({ tier: 'unassigned' }))
    expect(result.map(e => e.subject.id)).toEqual([3])
  })

  it('keeps null scores last in both directions', () => {
    const asc = filterLibrary(entries, filters({ sort: 'personal', direction: 'asc' }))
    const desc = filterLibrary(entries, filters({ sort: 'personal', direction: 'desc' }))
    expect(asc.map(e => e.subject.id)).toEqual([2, 1, 3])
    expect(desc.map(e => e.subject.id)).toEqual([1, 2, 3])
  })

  it('sorts by updated datetime descending', () => {
    const result = filterLibrary(entries, filters({ sort: 'updated', direction: 'desc' }))
    expect(result.map(e => e.subject.id)).toEqual([1, 3, 2])
  })

  it('does not mutate the input array', () => {
    const input = [...entries]
    filterLibrary(input, filters({ sort: 'personal', direction: 'desc' }))
    expect(input.map(e => e.subject.id)).toEqual([1, 2, 3])
  })
})
