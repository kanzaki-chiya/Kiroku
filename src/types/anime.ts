export type Tier = string
export type WatchStatus = 'completed' | 'watching' | 'planned'
export const dimensionKeys = ['story', 'characters', 'direction', 'animation', 'music'] as const
export type DimensionKey = typeof dimensionKeys[number]
export type CommunityStatus = 'ok' | 'missing' | 'not_fetched' | 'stale'

export interface BangumiSubject {
  id: number
  name: string
  nameCn: string
  aliases?: string[]
  summary: string
  coverUrl: string
  coverLocalPath?: string | null
  year: number
  format: 'TV' | 'Movie' | 'OVA'
  episodes: number
  studio: string
  tags: string[]
  community: {
    score: number | null
    votes: number
    rank: number | null
    fetchedAt?: string | null
    status?: CommunityStatus
  }
}

export interface PersonalDraft {
  score: number | null
  tier: Tier | null
  status: WatchStatus
  /** 0–5 星，0.5 步进；null 表示未评 */
  dimensions: Record<DimensionKey, number | null>
  review: string
}

export interface PersonalRecord extends PersonalDraft {
  subjectId: number
  createdAt: string
  updatedAt: string
  version: number
}

export interface LibraryEntry {
  localId?: number
  subject: BangumiSubject
  personal: PersonalRecord
}

export interface LibraryFilters {
  query: string
  tier: Tier | 'all' | 'unassigned'
  status: WatchStatus | 'all'
  year: number | 'all'
  sort: 'updated' | 'personal' | 'community' | 'title'
  direction: 'asc' | 'desc'
}

export interface TierDefinition {
  id: number
  name: string
  description: string
  color: string
  sortOrder: number
  builtin: boolean
}

export interface ImportPreview {
  added: number
  duplicates: number
  conflicts: number
}

export interface BackupDocument {
  formatVersion: number
  exportedAt: string
  tiers: TierDefinition[]
  entries: Array<{
    bangumiSubjectId: number
    subject: BangumiSubject
    personal: PersonalRecord
  }>
}
