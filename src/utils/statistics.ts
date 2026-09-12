import type { LibraryEntry } from '../types/anime'

export interface StatisticsBin {
  label: string
  min: number
  max: number
  count: number
}

export interface StatisticsDifference {
  entry: LibraryEntry
  delta: number
}

export interface LibraryStatistics {
  total: number
  ratedCount: number
  communityCount: number
  pairedCount: number
  personalMean: number | null
  communityMean: number | null
  meanDifference: number | null
  highest: LibraryEntry | null
  differences: StatisticsDifference[]
  bins: StatisticsBin[]
}

const mean = (values: number[]) =>
  values.length ? values.reduce((sum, value) => sum + value, 0) / values.length : null

export function calculateStatistics(entries: LibraryEntry[]): LibraryStatistics {
  const rated = entries.filter(e => e.personal.score !== null)
  const community = entries.filter(e => e.subject.community.score !== null)
  const paired = entries.filter(
    e => e.personal.score !== null && e.subject.community.score !== null
  )
  const differences = paired
    .map(e => ({ entry: e, delta: e.personal.score! - e.subject.community.score! }))
    .sort(
      (a, b) => Math.abs(b.delta) - Math.abs(a.delta) || a.entry.subject.id - b.entry.subject.id
    )
  const bins = [
    { label: '0–<6', min: 0, max: 6 },
    { label: '6–<7', min: 6, max: 7 },
    { label: '7–<8', min: 7, max: 8 },
    { label: '8–<9', min: 8, max: 9 },
    { label: '9–10', min: 9, max: 10.1 }
  ].map(bin => ({
    ...bin,
    count: rated.filter(e => e.personal.score! >= bin.min && e.personal.score! < bin.max).length
  }))
  const highest =
    [...rated].sort(
      (a, b) => b.personal.score! - a.personal.score! || a.subject.id - b.subject.id
    )[0] ?? null
  return {
    total: entries.length,
    ratedCount: rated.length,
    communityCount: community.length,
    pairedCount: paired.length,
    personalMean: mean(rated.map(e => e.personal.score!)),
    communityMean: mean(community.map(e => e.subject.community.score!)),
    meanDifference: mean(differences.map(d => d.delta)),
    highest,
    differences,
    bins
  }
}
