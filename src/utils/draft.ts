import { dimensionKeys, type PersonalDraft, type WatchStatus } from '../types/anime'

const statuses: WatchStatus[] = ['completed', 'watching', 'planned']

function isValidScore(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 && value <= 10
}

export function isValidDimension(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0.5 && value <= 5 && Number.isInteger(value * 2)
}

export function scaleV1Dimension(value: number | null): number | null {
  if (value === null) return null
  return Math.max(0.5, Math.round((value / 2) * 2) / 2)
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
  if (draft.progress !== null && (!Number.isInteger(draft.progress) || draft.progress < 0)) {
    throw new Error('观看进度需要是不小于 0 的整数')
  }
  for (const key of dimensionKeys) {
    const value = draft.dimensions[key]
    if (value !== null && !isValidDimension(value)) {
      throw new Error('维度评分必须是 0.5–5 之间的半星步进')
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
    progress: draft.progress,
    dimensions: { ...draft.dimensions },
    review: draft.review
  }
}

export function emptyDimensions(): Record<(typeof dimensionKeys)[number], number | null> {
  return { story: null, characters: null, direction: null, animation: null, music: null }
}

export function emptyDraft(status: WatchStatus = 'completed'): PersonalDraft {
  return { score: null, tier: null, status, progress: null, dimensions: emptyDimensions(), review: '' }
}

export function personalFieldsConflict(left: PersonalDraft, right: PersonalDraft): boolean {
  if (
    left.score !== right.score ||
    left.review !== right.review ||
    left.tier !== right.tier ||
    left.status !== right.status ||
    (left.progress ?? null) !== (right.progress ?? null)
  ) {
    return true
  }
  return dimensionKeys.some(key => left.dimensions[key] !== right.dimensions[key])
}
