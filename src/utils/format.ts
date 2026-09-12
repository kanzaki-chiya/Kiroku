import type { DimensionKey, TierDefinition, WatchStatus } from '../types/anime'

export const builtinTiers: TierDefinition[] = [
  { id: 1, name: 'S', description: '私心珍藏', color: '#bd592e', sortOrder: 0, builtin: true },
  { id: 2, name: 'A', description: '非常推荐', color: '#335d4e', sortOrder: 1, builtin: true },
  { id: 3, name: 'B', description: '值得一看', color: '#5b7c6e', sortOrder: 2, builtin: true },
  { id: 4, name: 'C', description: '略有保留', color: '#8a7a4d', sortOrder: 3, builtin: true },
  { id: 5, name: 'D', description: '不太对味', color: '#8b6b63', sortOrder: 4, builtin: true }
]

export const tierMeta: Record<string, { label: string; description: string }> = Object.fromEntries(
  builtinTiers.map(tier => [tier.name, { label: tier.name, description: tier.description }])
)

export const statusLabels: Record<WatchStatus, string> = {
  completed: '已看完',
  watching: '在看',
  planned: '想看'
}

export const dimensionLabels: Record<DimensionKey, string> = {
  story: '剧情',
  characters: '角色',
  direction: '演出',
  animation: '作画',
  music: '音乐'
}

export const formatLabels: Record<string, string> = {
  TV: 'TV 动画',
  Movie: '剧场版',
  OVA: 'OVA'
}

export function formatScore(score: number | null): string {
  return score === null ? '—' : score.toFixed(1)
}

export function formatDelta(delta: number): string {
  const rounded = Math.round(delta * 10) / 10
  return `${rounded > 0 ? '+' : ''}${rounded.toFixed(1)}`
}

export function resolveTier(name: string | null, tiers: TierDefinition[]): TierDefinition | undefined {
  if (!name) return undefined
  return tiers.find(tier => tier.name === name)
}

export function tierDescription(name: string | null, tiers: TierDefinition[]): string {
  const found = resolveTier(name, tiers)
  if (found) return found.description
  if (!name) return ''
  return tierMeta[name]?.description ?? ''
}

export function tierBadgeStyle(color: string | undefined): { backgroundColor: string } | undefined {
  return color ? { backgroundColor: color } : undefined
}
