import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import RatingForm from '../src/components/RatingForm.vue'
import SubjectSearch from '../src/components/SubjectSearch.vue'
import { useLibraryStore } from '../src/stores/library'
import { bangumiApi } from '../src/api/bangumi'
import { allSubjects } from '../src/data/subjects'
import type { BangumiSubject, PersonalDraft } from '../src/types/anime'
import { emptyDraft } from '../src/utils/draft'
import { builtinTiers } from '../src/utils/format'

function mountForm(initial: PersonalDraft) {
  return mount(RatingForm, {
    props: { initial, submitLabel: '保存', tiers: builtinTiers }
  })
}

function scoreInput(wrapper: ReturnType<typeof mountForm>) {
  return wrapper.find('input[type="number"]')
}

describe('RatingForm', () => {
  it('emits save with edited score, tier, dimension and review', async () => {
    const initial = emptyDraft()
    const wrapper = mountForm(initial)

    await wrapper.get('[role="switch"]').trigger('click')
    await scoreInput(wrapper).setValue('8.5')
    await wrapper.get('[data-tier="A"]').trigger('click')
    await wrapper.get('[data-star-value="4.5"]').trigger('click')
    await wrapper.find('textarea').setValue('慢热但回味很长。')

    await wrapper.find('form').trigger('submit')

    const events = wrapper.emitted('save')
    expect(events).toHaveLength(1)
    const draft = events![0][0] as PersonalDraft
    expect(draft.score).toBe(8.5)
    expect(draft.tier).toBe('A')
    expect(draft.status).toBe('completed')
    expect(draft.dimensions.story).toBe(4.5)
    expect(draft.dimensions.music).toBeNull()
    expect(draft.review).toBe('慢热但回味很长。')
  })

  it('emits cancel without mutating the initial prop', async () => {
    const initial: PersonalDraft = {
      score: 7.5,
      tier: 'B',
      status: 'watching',
      dimensions: { story: 4, characters: null, direction: null, animation: null, music: null },
      review: '原始短评'
    }
    const wrapper = mountForm(initial)

    await scoreInput(wrapper).setValue('3')
    await wrapper.find('textarea').setValue('改成别的')

    const cancel = wrapper.findAll('button').find(b => b.text() === '取消')
    await cancel!.trigger('click')

    expect(wrapper.emitted('cancel')).toHaveLength(1)
    expect(wrapper.emitted('save')).toBeUndefined()
    expect(initial.score).toBe(7.5)
    expect(initial.tier).toBe('B')
    expect(initial.review).toBe('原始短评')
  })

  it('rejects a score above 10', async () => {
    const initial = emptyDraft()
    initial.score = 8
    const wrapper = mountForm(initial)

    await scoreInput(wrapper).setValue('12')
    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('save')).toBeUndefined()
    expect(wrapper.text()).toContain('0–10')
  })

  it('rejects a blank score instead of silently storing zero', async () => {
    const initial = emptyDraft()
    initial.score = 8
    const wrapper = mountForm(initial)

    await scoreInput(wrapper).setValue('')
    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('save')).toBeUndefined()
    expect(wrapper.text()).toContain('不能为空')
  })

  it('accepts an explicit zero score', async () => {
    const initial = emptyDraft()
    initial.score = 8
    const wrapper = mountForm(initial)

    await scoreInput(wrapper).setValue('0')
    await wrapper.find('form').trigger('submit')

    const events = wrapper.emitted('save')
    expect(events).toHaveLength(1)
    expect((events![0][0] as PersonalDraft).score).toBe(0)
  })
})

describe('SubjectSearch import flow', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('lets a searched uncollected subject be rated and added to the store', async () => {
    const store = useLibraryStore()
    const before = store.count
    const wrapper = mount(SubjectSearch, {
      global: { stubs: { RouterLink: { template: '<a><slot /></a>' } } }
    })

    await wrapper.find('input[type="search"]').setValue('间谍')
    await vi.advanceTimersByTimeAsync(700)
    await flushPromises()

    const resultButton = wrapper
      .findAll('button')
      .find(b => b.text().includes('间谍过家家'))
    expect(resultButton).toBeTruthy()
    await resultButton!.trigger('click')
    await vi.advanceTimersByTimeAsync(200)
    await flushPromises()

    expect(wrapper.find('form').exists()).toBe(true)
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(store.count).toBe(before + 1)
    expect(store.getEntry(50265)?.subject.nameCn).toBe('间谍过家家')
    expect(wrapper.emitted('saved')).toEqual([[50265]])

    wrapper.unmount()
    await vi.advanceTimersByTimeAsync(2000)
  })

  it('drops a stale response when the query changes before it resolves', async () => {
    const deferreds: { resolve: (value: BangumiSubject[]) => void }[] = []
    const spy = vi.spyOn(bangumiApi, 'searchSubjects').mockImplementation(() => {
      let resolve!: (value: BangumiSubject[]) => void
      const pending = new Promise<BangumiSubject[]>(r => {
        resolve = r
      })
      deferreds.push({ resolve })
      return pending
    })

    const wrapper = mount(SubjectSearch, {
      global: { stubs: { RouterLink: { template: '<a><slot /></a>' } } }
    })

    const input = wrapper.find('input[type="search"]')
    await input.setValue('芙莉莲')
    await vi.advanceTimersByTimeAsync(250)
    expect(deferreds.length).toBe(1)

    await input.setValue('间谍')
    deferreds[0].resolve([allSubjects.find(s => s.id === 52991)!])
    await flushPromises()
    expect(wrapper.findAll('.result').length).toBe(0)
    expect(wrapper.text()).not.toContain('葬送的芙莉莲')

    await vi.advanceTimersByTimeAsync(250)
    expect(deferreds.length).toBe(2)
    deferreds[1].resolve([allSubjects.find(s => s.id === 50265)!])
    await flushPromises()
    expect(wrapper.text()).toContain('间谍过家家')
    expect(wrapper.text()).not.toContain('葬送的芙莉莲')

    spy.mockRestore()
    wrapper.unmount()
    await vi.advanceTimersByTimeAsync(2000)
  })
})
