import { dimensionKeys, type PersonalDraft, type WatchStatus } from '../types/anime'

const statuses: WatchStatus[] = ['completed', 'watching', 'planned']

function isValidScore(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 && value <= 10
}

export function validateDraft(draft: PersonalDraft, knownTiers: string[]): PersonalDraft {
  if (draft.score !== null && !isValidScore(draft.score)) {
    throw new Error('个人评分必须是 0–10 之间的数字')
  }
  if (draft.tier !== null && !knownTiers.includes(draft.tier)) {
    throw new Error('无效的分档')
  }
  if (!statuses.includes(draft.status)) {
    throw new Error('无效的观看状态')
  }
  for (const key of dimensionKeys) {
    const value = draft.dimensions[key]
    if (value !== null && !isValidScore(value)) {
      throw new Error('维度评分必须是 0–10 之间的数字')
    }
  }
  if (typeof draft.review !== 'string' || draft.review.length > 5000) {
    throw new Error('短评不能超过 5000 字')
  }
  return draft
}

export function cloneDraft(draft: PersonalDraft): PersonalDraft {
  return {
    score: draft.score,
    tier: draft.tier,
    status: draft.status,
    dimensions: { ...draft.dimensions },
    review: draft.review
  }
}

export function emptyDimensions(): Record<(typeof dimensionKeys)[number], number | null> {
  return { story: null, characters: null, direction: null, animation: null, music: null }
}

export function emptyDraft(status: WatchStatus = 'completed'): PersonalDraft {
  return { score: null, tier: null, status, dimensions: emptyDimensions(), review: '' }
}

export function personalFieldsConflict(left: PersonalDraft, right: PersonalDraft): boolean {
  if (
    left.score !== right.score ||
    left.review !== right.review ||
    left.tier !== right.tier ||
    left.status !== right.status
  ) {
    return true
  }
  return dimensionKeys.some(key => left.dimensions[key] !== right.dimensions[key])
}
