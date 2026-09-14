import { describe, expect, it } from 'vitest'
import { calculateStatistics } from '../src/utils/statistics'
import { dimensionKeys, type BangumiSubject, type LibraryEntry, type PersonalDraft } from '../src/types/anime'
import { emptyDimensions } from '../src/utils/draft'

let nextId = 1

function makeEntry(
  personal: number | null,
  community: number | null,
  dimensions: Partial<PersonalDraft['dimensions']> = {}
): LibraryEntry {
  const id = nextId++
  const subject: BangumiSubject = {
    id,
    name: `Subject ${id}`,
    nameCn: `作品${id}`,
    summary: '',
    coverUrl: '',
    year: 2020,
    format: 'TV',
    episodes: 12,
    studio: 'Studio',
    tags: [],
    community: { score: community, votes: 100, rank: null }
  }
  const draft: PersonalDraft = {
    score: personal,
    tier: null,
    status: 'completed',
    progress: null,
    dimensions: { ...emptyDimensions(), ...dimensions },
    review: ''
  }
  return {
    subject,
    personal: {
      ...draft,
      subjectId: id,
      createdAt: '2025-01-01T00:00:00.000Z',
      updatedAt: '2025-01-01T00:00:00.000Z',
      version: 1
    }
  }
}

describe('calculateStatistics', () => {
  it('computes means, pairing and divergence on a small fixture', () => {
    const entries = [
      makeEntry(8, 7, { story: 4, characters: 0, animation: 5 }),
      makeEntry(6, 8, { characters: 4, animation: 4 }),
      makeEntry(null, 9, { story: 3, animation: 4.5, music: 0.5 })
    ]
    entries[1].personal.status = 'watching'
    entries[2].personal.status = 'planned'
    const stats = calculateStatistics(entries)

    expect(stats.total).toBe(3)
    expect(stats.ratedCount).toBe(2)
    expect(stats.communityCount).toBe(3)
    expect(stats.pairedCount).toBe(2)
    expect(stats.personalMean).toBe(7)
    expect(stats.communityMean).toBe(8)
    expect(stats.meanDifference).toBe(-0.5)
    expect(stats.highest?.personal.score).toBe(8)
    expect(stats.highest?.subject.id).toBe(entries[0].subject.id)
    expect(stats.differences.map(d => d.delta)).toEqual([-2, 1])
    expect(stats.differences[0].entry.subject.id).toBe(entries[1].subject.id)
    expect(stats.bins.map(b => b.count)).toEqual([0, 1, 0, 1, 0])
    expect(stats.dimensions).toEqual([
      { key: 'story', mean: 3.5, count: 2 },
      { key: 'characters', mean: 2, count: 2 },
      { key: 'direction', mean: null, count: 0 },
      { key: 'animation', mean: 4.5, count: 3 },
      { key: 'music', mean: 0.5, count: 1 }
    ])
  })

  it('returns null means and null highest on an empty library', () => {
    const stats = calculateStatistics([])
    expect(stats.total).toBe(0)
    expect(stats.personalMean).toBeNull()
    expect(stats.communityMean).toBeNull()
    expect(stats.meanDifference).toBeNull()
    expect(stats.highest).toBeNull()
    expect(stats.differences).toEqual([])
    expect(stats.bins.every(b => b.count === 0)).toBe(true)
    expect(stats.dimensions).toEqual(dimensionKeys.map(key => ({ key, mean: null, count: 0 })))
  })

  it('keeps all dimensions empty for entries with only total scores', () => {
    expect(calculateStatistics([makeEntry(8, 7)]).dimensions).toEqual(
      dimensionKeys.map(key => ({ key, mean: null, count: 0 }))
    )
  })

  it('keeps zero, half-stars and full stars without requiring a total score', () => {
    const stats = calculateStatistics([
      makeEntry(null, null, { story: 0, characters: 0.5, animation: 5 }),
      makeEntry(null, null, { characters: 1 })
    ])
    expect(stats.dimensions).toEqual([
      { key: 'story', mean: 0, count: 1 },
      { key: 'characters', mean: 0.75, count: 2 },
      { key: 'direction', mean: null, count: 0 },
      { key: 'animation', mean: 5, count: 1 },
      { key: 'music', mean: null, count: 0 }
    ])
    expect(stats.ratedCount).toBe(0)
  })

  it('places boundary scores into the correct bins', () => {
    const entries = [0, 6, 7, 8, 9, 10].map(score => makeEntry(score, null))
    const stats = calculateStatistics(entries)
    expect(stats.bins.map(b => b.count)).toEqual([1, 1, 1, 1, 2])
  })

  it('excludes unpaired entries from meanDifference and treats zero as scored', () => {
    const entries = [makeEntry(0, null), makeEntry(null, 9)]
    const stats = calculateStatistics(entries)
    expect(stats.ratedCount).toBe(1)
    expect(stats.personalMean).toBe(0)
    expect(stats.pairedCount).toBe(0)
    expect(stats.meanDifference).toBeNull()
    expect(stats.highest?.personal.score).toBe(0)
  })
})
