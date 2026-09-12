import { describe, expect, it } from 'vitest'
import { bangumiApi } from '../src/api/bangumi'

describe('mock bangumi api', () => {
  it('finds subjects by Chinese name', async () => {
    const results = await bangumiApi.searchSubjects('孤独摇滚')
    expect(results.map(s => s.id)).toContain(47917)
  })

  it('finds subjects by original name and tags', async () => {
    const byName = await bangumiApi.searchSubjects('steins')
    expect(byName.map(s => s.id)).toEqual([9253])

    const byTag = await bangumiApi.searchSubjects('美食')
    expect(byTag.map(s => s.id)).toEqual([52701])
  })

  it('finds subjects by Japanese original name', async () => {
    const results = await bangumiApi.searchSubjects('ぼっち')
    expect(results.map(s => s.id)).toEqual([47917])
  })

  it('finds subjects by romanized alias', async () => {
    const results = await bangumiApi.searchSubjects('bocchi')
    expect(results.map(s => s.id)).toEqual([47917])
  })

  it('returns cloned suggestions that cannot corrupt the catalog', async () => {
    const first = await bangumiApi.getSuggestions()
    expect(first.length).toBe(18)
    first[0].nameCn = '被篡改'
    first[0].tags.push('脏数据')
    const second = await bangumiApi.getSuggestions()
    expect(second[0].nameCn).not.toBe('被篡改')
    expect(second[0].tags).not.toContain('脏数据')
  })

  it('returns an empty array for unknown and blank queries', async () => {
    expect(await bangumiApi.searchSubjects('不存在的作品xyz')).toEqual([])
    expect(await bangumiApi.searchSubjects('   ')).toEqual([])
  })

  it('returns deep copies so results cannot corrupt the catalog', async () => {
    const first = await bangumiApi.getSubject(52991)
    first!.nameCn = '被篡改'
    first!.tags.push('脏数据')
    const second = await bangumiApi.getSubject(52991)
    expect(second!.nameCn).toBe('葬送的芙莉莲')
    expect(second!.tags).not.toContain('脏数据')
  })

  it('returns null for an unknown id', async () => {
    expect(await bangumiApi.getSubject(999999)).toBeNull()
  })

  it('rejects with AbortError when the signal is aborted', async () => {
    const controller = new AbortController()
    const pending = bangumiApi.searchSubjects('芙莉莲', controller.signal)
    controller.abort()
    await expect(pending).rejects.toMatchObject({ name: 'AbortError' })

    const preAborted = new AbortController()
    preAborted.abort()
    await expect(
      bangumiApi.searchSubjects('芙莉莲', preAborted.signal)
    ).rejects.toMatchObject({ name: 'AbortError' })
  })
})
