import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import AnimeCover from '../src/components/AnimeCover.vue'
import FeaturedPanel from '../src/components/FeaturedPanel.vue'
import type { BangumiSubject, LibraryEntry } from '../src/types/anime'
import { emptyDimensions } from '../src/utils/draft'

function makeSubject(id: number, nameCn: string, coverUrl = `/covers/${id}.jpg`): BangumiSubject {
  return {
    id,
    name: `Name ${id}`,
    nameCn,
    summary: '简介',
    coverUrl,
    year: 2024,
    format: 'TV',
    episodes: 12,
    studio: 'Studio',
    tags: [],
    community: { score: 8.0, votes: 100, rank: 10 }
  }
}

function makeEntry(subject: BangumiSubject, review: string): LibraryEntry {
  return {
    subject,
    personal: {
      score: 8.5,
      tier: 'A',
      status: 'completed',
      progress: 12,
      dimensions: emptyDimensions(),
      review,
      subjectId: subject.id,
      createdAt: '2025-01-01T00:00:00.000Z',
      updatedAt: '2025-01-01T00:00:00.000Z',
      version: 1
    }
  }
}

const routerStubs = { RouterLink: { template: '<a><slot /></a>' } }

describe('prop-driven reactivity', () => {
  it('FeaturedPanel recomputes the excerpt when the featured entry changes', async () => {
    const wrapper = mount(FeaturedPanel, {
      props: { entry: makeEntry(makeSubject(1, '作品甲'), '第一段感想') },
      global: { stubs: routerStubs }
    })
    expect(wrapper.text()).toContain('第一段感想')

    await wrapper.setProps({ entry: makeEntry(makeSubject(2, '作品乙'), '第二段感想') })
    expect(wrapper.text()).toContain('第二段感想')
    expect(wrapper.text()).toContain('作品乙')
  })

  it('AnimeCover recovers from a failed image when the subject changes', async () => {
    const wrapper = mount(AnimeCover, {
      props: { subject: makeSubject(1, '旧作品') }
    })
    await wrapper.find('img').trigger('error')
    expect(wrapper.find('img').exists()).toBe(false)
    expect(wrapper.find('.cover-fallback').exists()).toBe(true)

    await wrapper.setProps({ subject: makeSubject(2, '新作品', '/covers/new.jpg') })
    const img = wrapper.find('img')
    expect(img.exists()).toBe(true)
    expect(img.attributes('src')).toBe('/covers/new.jpg')
  })
})
