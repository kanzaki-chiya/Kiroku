import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLibraryStore } from '../src/stores/library'
import { allSubjects } from '../src/data/subjects'
import { emptyDraft, scaleV1Dimension } from '../src/utils/draft'

const KON_ID = 5680

function konSubject() {
  const subject = allSubjects.find(s => s.id === KON_ID)
  if (!subject) throw new Error('missing fixture subject')
  return subject
}

describe('library store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('seeds the demo collection', () => {
    const store = useLibraryStore()
    expect(store.count).toBe(12)
    expect(store.getEntry(52991)?.subject.nameCn).toBe('葬送的芙莉莲')
  })

  it('add clones the draft instead of holding the caller object', async () => {
    const store = useLibraryStore()
    const draft = emptyDraft()
    draft.score = 7.5
    draft.dimensions.story = 4
    draft.review = '很不错的社团日常'

    await store.add(konSubject(), draft)

    draft.score = 1
    draft.dimensions.story = 1
    draft.review = '被改掉'

    const saved = store.getEntry(KON_ID)!.personal
    expect(saved.score).toBe(7.5)
    expect(saved.dimensions.story).toBe(4)
    expect(saved.review).toBe('很不错的社团日常')
  })

  it('rejects adding a duplicate subject', async () => {
    const store = useLibraryStore()
    await store.add(konSubject(), emptyDraft())
    await expect(store.add(konSubject(), emptyDraft())).rejects.toThrow()
    expect(store.count).toBe(13)
  })

  it('rejects out-of-range and NaN scores', async () => {
    const store = useLibraryStore()
    const tooHigh = emptyDraft()
    tooHigh.score = 10.5
    await expect(store.add(konSubject(), tooHigh)).rejects.toThrow()

    const nan = emptyDraft()
    nan.score = Number.NaN
    await expect(store.add(konSubject(), nan)).rejects.toThrow()

    const badDim = emptyDraft()
    badDim.dimensions.music = -1
    await expect(store.add(konSubject(), badDim)).rejects.toThrow()
    expect(store.count).toBe(12)
  })

  it('update edits personal fields without touching subject metadata', async () => {
    const store = useLibraryStore()
    const before = store.getEntry(52991)!
    const subjectRef = before.subject
    const createdAt = before.personal.createdAt

    vi.useFakeTimers()
    vi.setSystemTime(new Date('2025-07-01T12:00:00.000Z'))
    try {
      const draft = emptyDraft('watching')
      draft.score = 9.8
      draft.tier = 'S'
      draft.dimensions.direction = 5
      draft.review = '二刷之后更确定了。'
      await store.update(52991, draft)
    } finally {
      vi.useRealTimers()
    }

    const after = store.getEntry(52991)!
    expect(after.subject).toBe(subjectRef)
    expect(after.subject.nameCn).toBe('葬送的芙莉莲')
    expect(after.personal.score).toBe(9.8)
    expect(after.personal.tier).toBe('S')
    expect(after.personal.status).toBe('watching')
    expect(after.personal.dimensions.direction).toBe(5)
    expect(after.personal.review).toBe('二刷之后更确定了。')
    expect(after.personal.createdAt).toBe(createdAt)
    expect(after.personal.updatedAt).toBe('2025-07-01T12:00:00.000Z')
  })

  it('update rejects invalid scores and unknown ids', async () => {
    const store = useLibraryStore()
    const bad = emptyDraft()
    bad.score = 11
    await expect(store.update(52991, bad)).rejects.toThrow()
    await expect(store.update(KON_ID, emptyDraft())).rejects.toThrow()
  })

  it('treats dimension-only backup diffs as conflicts', async () => {
    const store = useLibraryStore()
    const backup = await store.exportBackup()
    const target = backup.entries.find(entry => entry.bangumiSubjectId === 52991)
    if (!target) throw new Error('missing fixture entry')
    target.personal.dimensions = { ...target.personal.dimensions, music: 4 }
    const preview = await store.previewImport(backup)
    expect(preview.added).toBe(0)
    expect(preview.duplicates).toBe(12)
    expect(preview.conflicts).toBe(1)
  })
})

describe('scaleV1Dimension', () => {
  it('halves v1 scores and clamps sub-half-star results to 0.5', () => {
    expect(scaleV1Dimension(8.7)).toBe(4.5)
    expect(scaleV1Dimension(9)).toBe(4.5)
    expect(scaleV1Dimension(0.4)).toBe(0.5)
    expect(scaleV1Dimension(0)).toBe(0.5)
    expect(scaleV1Dimension(null)).toBeNull()
  })
})
