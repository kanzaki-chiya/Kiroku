import type { LibraryEntry, LibraryFilters } from '../types/anime'

export function createDefaultFilters(): LibraryFilters {
  return {
    query: '',
    tier: 'all',
    status: 'all',
    year: 'all',
    sort: 'updated',
    direction: 'desc'
  }
}

type SortValue = string | number | null

function compareValues(a: SortValue, b: SortValue, direction: 1 | -1): number {
  const aNull = a === null
  const bNull = b === null
  if (aNull && bNull) return 0
  if (aNull) return 1
  if (bNull) return -1
  if (a < b) return -1 * direction
  if (a > b) return 1 * direction
  return 0
}

export function filterLibrary(entries: LibraryEntry[], filters: LibraryFilters): LibraryEntry[] {
  const query = filters.query.trim().toLowerCase()

  const filtered = entries.filter(entry => {
    if (query) {
      const haystack = [
        entry.subject.nameCn,
        entry.subject.name,
        ...(entry.subject.aliases ?? []),
        ...entry.subject.tags
      ]
        .join('\n')
        .toLowerCase()
      if (!haystack.includes(query)) return false
    }
    if (filters.tier === 'unassigned') {
      if (entry.personal.tier !== null) return false
    } else if (filters.tier !== 'all' && entry.personal.tier !== filters.tier) {
      return false
    }
    if (filters.status !== 'all' && entry.personal.status !== filters.status) return false
    if (filters.year !== 'all' && entry.subject.year !== filters.year) return false
    return true
  })

  const direction = filters.direction === 'asc' ? 1 : -1

  return [...filtered].sort((a, b) => {
    let result = 0
    switch (filters.sort) {
      case 'updated':
        result = compareValues(a.personal.updatedAt, b.personal.updatedAt, direction)
        break
      case 'personal':
        result = compareValues(a.personal.score, b.personal.score, direction)
        break
      case 'community':
        result = compareValues(a.subject.community.score, b.subject.community.score, direction)
        break
      case 'title':
        result =
          a.subject.nameCn.localeCompare(b.subject.nameCn, 'zh-CN') * direction
        break
    }
    return result || a.subject.id - b.subject.id
  })
}
