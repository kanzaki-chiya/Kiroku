import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { convertFileSrc } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { join } from '@tauri-apps/api/path'
import { allSubjects } from '../data/subjects'
import { seedRecords } from '../data/seed'
import { isTauri } from '../runtime'
import { invokeCmd } from '../services/tauri'
import { toBackendError } from '../services/errors'
import type {
  BackupDocument,
  BangumiSubject,
  ImportPreview,
  LibraryEntry,
  PersonalDraft,
  PersonalRecord,
  TierDefinition
} from '../types/anime'
import { cloneDraft, personalFieldsConflict, validateDraft } from '../utils/draft'
import { builtinTiers } from '../utils/format'
import { calculateStatistics, type LibraryStatistics } from '../utils/statistics'

function cloneSubject(subject: BangumiSubject): BangumiSubject {
  return {
    ...subject,
    tags: [...subject.tags],
    aliases: subject.aliases ? [...subject.aliases] : undefined,
    community: { ...subject.community },
    coverLocalPath: subject.coverLocalPath
  }
}

interface ListLibraryResponse {
  items: LibraryEntry[]
  total: number
}

interface CoverReadyEvent {
  bangumiSubjectId: number
  coverLocalPath: string
}

const HYDRATE_PAGE_SIZE = 200

export const useLibraryStore = defineStore('library', () => {
  const subjects = ref(new Map<number, BangumiSubject>())
  const records = ref(new Map<number, PersonalRecord>())
  const tiers = ref<TierDefinition[]>([])
  const bootError = ref<string | null>(null)
  const saving = ref(false)
  const dataDir = ref('')
  const desktop = isTauri()

  const entries = computed<LibraryEntry[]>(() => {
    const list: LibraryEntry[] = []
    for (const [subjectId, personal] of records.value) {
      const subject = subjects.value.get(subjectId)
      if (subject) list.push({ subject, personal })
    }
    return list
  })

  const count = computed(() => records.value.size)
  const knownTierNames = computed(() => tiers.value.map(tier => tier.name))

  function getEntry(subjectId: number): LibraryEntry | null {
    const subject = subjects.value.get(subjectId)
    const personal = records.value.get(subjectId)
    if (!subject || !personal) return null
    return { subject, personal }
  }

  function hasSubject(subjectId: number): boolean {
    return records.value.has(subjectId)
  }

  function applyEntry(entry: LibraryEntry) {
    subjects.value.set(entry.subject.id, cloneSubject(entry.subject))
    records.value.set(entry.subject.id, { ...entry.personal, dimensions: { ...entry.personal.dimensions } })
  }

  function replaceAll(list: LibraryEntry[]) {
    const nextSubjects = new Map<number, BangumiSubject>()
    const nextRecords = new Map<number, PersonalRecord>()
    for (const entry of list) {
      nextSubjects.set(entry.subject.id, cloneSubject(entry.subject))
      nextRecords.set(entry.subject.id, {
        ...entry.personal,
        dimensions: { ...entry.personal.dimensions }
      })
    }
    subjects.value = nextSubjects
    records.value = nextRecords
  }

  function renameTierOnRecords(from: string, to: string) {
    if (from === to) return
    for (const [id, record] of records.value) {
      if (record.tier === from) {
        records.value.set(id, { ...record, tier: to })
      }
    }
  }

  async function resolveCoverUrl(relativePath: string | null | undefined): Promise<string> {
    if (!relativePath || !dataDir.value) return ''
    const absolute = await join(dataDir.value, relativePath)
    return convertFileSrc(absolute)
  }

  async function decorateEntry(entry: LibraryEntry): Promise<LibraryEntry> {
    if (!desktop) return entry
    const coverUrl = await resolveCoverUrl(entry.subject.coverLocalPath)
    return {
      ...entry,
      subject: { ...entry.subject, coverUrl }
    }
  }

  async function add(subject: BangumiSubject, draft: PersonalDraft): Promise<PersonalRecord> {
    validateDraft(draft, knownTierNames.value)
    saving.value = true
    try {
      if (desktop) {
        const entry = await decorateEntry(
          await invokeCmd<LibraryEntry>('add_library_entry', {
            payload: { subject, draft }
          })
        )
        applyEntry(entry)
        return entry.personal
      }
      if (records.value.has(subject.id)) {
        throw new Error('这部作品已经在你的番剧库中')
      }
      if (!subjects.value.has(subject.id)) {
        subjects.value.set(subject.id, cloneSubject(subject))
      }
      const now = new Date().toISOString()
      const record: PersonalRecord = {
        ...cloneDraft(draft),
        subjectId: subject.id,
        createdAt: now,
        updatedAt: now,
        version: 1
      }
      records.value.set(subject.id, record)
      return record
    } catch (error) {
      throw toBackendError(error)
    } finally {
      saving.value = false
    }
  }

  async function update(subjectId: number, draft: PersonalDraft): Promise<PersonalRecord> {
    validateDraft(draft, knownTierNames.value)
    const existing = records.value.get(subjectId)
    if (!existing) {
      throw new Error('这部作品还没有收录')
    }
    saving.value = true
    try {
      if (desktop) {
        const entry = await decorateEntry(
          await invokeCmd<LibraryEntry>('update_personal_record', {
            payload: {
              bangumiSubjectId: subjectId,
              draft,
              version: existing.version
            }
          })
        )
        applyEntry(entry)
        return entry.personal
      }
      const next: PersonalRecord = {
        ...existing,
        ...cloneDraft(draft),
        updatedAt: new Date().toISOString(),
        version: existing.version + 1
      }
      records.value.set(subjectId, next)
      return next
    } catch (error) {
      throw toBackendError(error)
    } finally {
      saving.value = false
    }
  }

  function seed() {
    const subjectById = new Map(allSubjects.map(s => [s.id, s]))
    for (const seed of seedRecords) {
      const subject = subjectById.get(seed.subjectId)
      if (!subject) continue
      subjects.value.set(subject.id, cloneSubject(subject))
      records.value.set(subject.id, {
        ...cloneDraft(seed.draft),
        subjectId: seed.subjectId,
        createdAt: seed.createdAt,
        updatedAt: seed.updatedAt,
        version: 1
      })
    }
    tiers.value = builtinTiers.map(tier => ({ ...tier }))
  }

  async function fetchLibraryPage(offset: number): Promise<ListLibraryResponse> {
    return invokeCmd<ListLibraryResponse>('list_library', {
      query: {
        query: '',
        tier: 'all',
        status: 'all',
        sort: 'updated',
        direction: 'desc',
        offset,
        limit: HYDRATE_PAGE_SIZE
      }
    })
  }

  async function hydrate() {
    bootError.value = null
    if (!desktop) return
    try {
      const boot = await invokeCmd<{ dataDir: string }>('bootstrap')
      dataDir.value = boot.dataDir
      const first = await fetchLibraryPage(0)
      const items = [...first.items]
      while (items.length < first.total) {
        const page = await fetchLibraryPage(items.length)
        if (page.items.length === 0) break
        items.push(...page.items)
        if (page.items.length < HYDRATE_PAGE_SIZE) break
      }
      const decorated: LibraryEntry[] = []
      for (const entry of items) {
        decorated.push(await decorateEntry(entry))
      }
      replaceAll(decorated)
      tiers.value = await invokeCmd<TierDefinition[]>('list_tiers')
    } catch (error) {
      const next = toBackendError(error)
      bootError.value = next.message
      throw next
    }
  }

  async function refreshSubject(subjectId: number) {
    if (!desktop) return getEntry(subjectId)
    const entry = await decorateEntry(
      await invokeCmd<LibraryEntry>('refresh_subject', { bangumiSubjectId: subjectId })
    )
    applyEntry(entry)
    return entry
  }

  async function saveTier(payload: {
    id?: number
    name: string
    description: string
    color: string
  }): Promise<TierDefinition> {
    if (desktop) {
      const saved = await invokeCmd<TierDefinition>('save_tier', { payload })
      tiers.value = await invokeCmd<TierDefinition[]>('list_tiers')
      return saved
    }
    const name = payload.name.trim()
    if (!name || name.length > 20) throw new Error('分档名称不能为空且不超过 20 字')
    if (!payload.color.trim()) throw new Error('分档颜色不能为空')
    if (payload.id !== undefined) {
      const index = tiers.value.findIndex(tier => tier.id === payload.id)
      if (index < 0) throw new Error('分档不存在')
      const previous = tiers.value[index]
      const next = { ...previous, name, description: payload.description, color: payload.color }
      tiers.value = tiers.value.map(tier => (tier.id === payload.id ? next : tier))
      renameTierOnRecords(previous.name, name)
      return next
    }
    const id = Math.max(0, ...tiers.value.map(tier => tier.id)) + 1
    const created: TierDefinition = {
      id,
      name,
      description: payload.description,
      color: payload.color,
      sortOrder: tiers.value.length,
      builtin: false
    }
    tiers.value = [...tiers.value, created]
    return created
  }

  async function reorderTiers(ids: number[]): Promise<TierDefinition[]> {
    if (desktop) {
      const next = await invokeCmd<TierDefinition[]>('reorder_tiers', { ids })
      tiers.value = next
      return next
    }
    if (ids.length !== tiers.value.length) throw new Error('分档排序不完整')
    const byId = new Map(tiers.value.map(tier => [tier.id, tier]))
    const next = ids.map((id, sortOrder) => {
      const tier = byId.get(id)
      if (!tier) throw new Error('分档不存在')
      return { ...tier, sortOrder }
    })
    tiers.value = next
    return next
  }

  async function deleteTier(id: number) {
    if (desktop) {
      await invokeCmd('delete_tier', { id })
      tiers.value = await invokeCmd<TierDefinition[]>('list_tiers')
      return
    }
    const target = tiers.value.find(tier => tier.id === id)
    if (!target) throw new Error('分档不存在')
    if (target.builtin) throw toBackendError({ code: 'CONFLICT', message: '内置分档不能删除' })
    const referenced = [...records.value.values()].some(record => record.tier === target.name)
    if (referenced) {
      throw toBackendError({ code: 'CONFLICT', message: '该分档仍被收藏引用，请先迁移或取消关联' })
    }
    tiers.value = tiers.value.filter(tier => tier.id !== id)
  }

  async function getStatistics(): Promise<LibraryStatistics> {
    if (desktop) return invokeCmd<LibraryStatistics>('get_statistics')
    return calculateStatistics(entries.value)
  }

  async function exportBackup() {
    if (!desktop) {
      return {
        formatVersion: 1,
        exportedAt: new Date().toISOString(),
        tiers: tiers.value,
        entries: entries.value.map(entry => ({
          bangumiSubjectId: entry.subject.id,
          subject: entry.subject,
          personal: {
            ...entry.personal,
            dimensions: { ...entry.personal.dimensions }
          }
        }))
      } satisfies BackupDocument
    }
    return invokeCmd<BackupDocument>('export_backup')
  }

  async function previewImport(document: BackupDocument) {
    if (!desktop) {
      let added = 0
      let duplicates = 0
      let conflicts = 0
      for (const item of document.entries) {
        const existing = records.value.get(item.bangumiSubjectId)
        if (!existing) {
          added += 1
          continue
        }
        duplicates += 1
        if (personalFieldsConflict(existing, item.personal)) conflicts += 1
      }
      return { added, duplicates, conflicts } satisfies ImportPreview
    }
    return invokeCmd<ImportPreview>('preview_import', { document })
  }

  async function importBackup(document: BackupDocument, overwrite: boolean) {
    if (!desktop) {
      throw new Error('浏览器演示模式不写入导入结果')
    }
    const preview = await invokeCmd<ImportPreview>('import_backup', {
      payload: { document, overwrite }
    })
    await hydrate()
    return preview
  }

  async function snapshotDatabase() {
    if (!desktop) throw new Error('浏览器演示模式不支持数据库快照')
    return invokeCmd<string>('snapshot_database')
  }

  async function clearCoverCache() {
    if (!desktop) return
    await invokeCmd('clear_cover_cache')
    for (const [id, subject] of subjects.value) {
      subjects.value.set(id, { ...subject, coverUrl: '', coverLocalPath: null })
    }
  }

  async function deletePersonalData() {
    if (!desktop) return
    await invokeCmd('delete_personal_data')
    subjects.value = new Map()
    records.value = new Map()
  }

  let stopCoverListen: UnlistenFn | null = null

  async function listenCovers() {
    if (!desktop) return
    stopCoverListen = await listen<CoverReadyEvent>('cover-ready', async event => {
      const current = subjects.value.get(event.payload.bangumiSubjectId)
      if (!current) return
      const coverUrl = await resolveCoverUrl(event.payload.coverLocalPath)
      subjects.value.set(event.payload.bangumiSubjectId, {
        ...current,
        coverUrl,
        coverLocalPath: event.payload.coverLocalPath
      })
    })
  }

  function dispose() {
    stopCoverListen?.()
    stopCoverListen = null
  }

  if (!desktop) seed()

  return {
    subjects,
    records,
    entries,
    count,
    tiers,
    bootError,
    saving,
    desktop,
    getEntry,
    hasSubject,
    add,
    update,
    hydrate,
    refreshSubject,
    saveTier,
    reorderTiers,
    deleteTier,
    getStatistics,
    exportBackup,
    previewImport,
    importBackup,
    snapshotDatabase,
    clearCoverCache,
    deletePersonalData,
    listenCovers,
    dispose
  }
})
