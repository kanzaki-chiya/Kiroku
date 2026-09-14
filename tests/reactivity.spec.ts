import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import AnimeCover from '../src/components/AnimeCover.vue'
import FeaturedPanel from '../src/components/FeaturedPanel.vue'
import TasteProfile from '../src/components/TasteProfile.vue'
import { dimensionKeys, type BangumiSubject, type LibraryEntry } from '../src/types/anime'
import type { StatisticsDimension } from '../src/utils/statistics'
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

function profileDimensions(
  values: Partial<Record<StatisticsDimension['key'], number>> = {}
): StatisticsDimension[] {
  return dimensionKeys.map(key => ({
    key,
    mean: values[key] ?? null,
    count: values[key] === undefined ? 0 : 2
  }))
}

describe('taste profile', () => {
  it('renders a five-axis radar with means, counts and normalized vertices', () => {
    const wrapper = mount(TasteProfile, {
      props: { dimensions: profileDimensions({ story: 0, animation: 5, music: 2.5 }), ready: true }
    })
    const radar = wrapper.find('svg.radar')
    expect(radar.exists()).toBe(true)
    expect(radar.attributes('role')).toBe('img')
    expect(wrapper.findAll('.radar-ring')).toHaveLength(5)
    expect(wrapper.findAll('.radar-axis')).toHaveLength(5)
    const labels = wrapper.findAll('.radar-label')
    expect(labels).toHaveLength(5)
    expect(wrapper.find('.dimension-track').exists()).toBe(false)
    expect(labels[0].text()).toContain('0.0 星')
    expect(labels[0].text()).toContain('已评 2 部')
    expect(labels[1].text()).toContain('未评')
    expect(labels[1].text()).toContain('已评 0 部')
    expect(wrapper.find('desc').text()).toContain('剧情：0.0 星，已评 2 部')
    expect(wrapper.find('desc').text()).toContain('角色：未评，已评 0 部')
    const vertices = wrapper
      .find('.radar-area')
      .attributes('points')!
      .split(' ')
      .map(pair => pair.split(',').map(Number))
    expect(vertices).toHaveLength(5)
    expect(vertices[0]).toEqual([230, 180])
    expect(vertices[1]).toEqual([230, 180])
    expect(vertices[2]).toEqual([230, 180])
    expect(vertices[3][0]).toBeCloseTo(164.168, 2)
    expect(vertices[3][1]).toBeCloseTo(270.61, 2)
    expect(vertices[4][0]).toBeCloseTo(176.741, 2)
    expect(vertices[4][1]).toBeCloseTo(162.695, 2)
    const points = wrapper.findAll('.radar-point')
    expect(points).toHaveLength(3)
    const storyPoint = wrapper.find('.radar-point[data-dimension="story"]')
    expect(storyPoint.attributes('cx')).toBe('230')
    expect(storyPoint.attributes('cy')).toBe('180')
    expect(wrapper.find('.radar-point[data-dimension="characters"]').exists()).toBe(false)
    expect(wrapper.find('.profile-summary').text()).toBe('均分最高：作画 5.0 星 · 均分最低：剧情 0.0 星')
  })

  it('shows the empty state and reacts to prop changes', async () => {
    const wrapper = mount(TasteProfile, {
      props: { dimensions: profileDimensions(), ready: false }
    })
    expect(wrapper.find('.profile-empty').exists()).toBe(true)
    expect(wrapper.find('.profile-empty').text()).toContain('在作品详情中')
    for (const value of wrapper.findAll('.dimension-value')) {
      expect(value.text()).toBe('未评')
    }
    for (const count of wrapper.findAll('.dimension-count')) {
      expect(count.text()).toBe('已评 0 部')
    }
    expect(wrapper.find('.radar-area').exists()).toBe(false)
    expect(wrapper.findAll('.radar-point')).toHaveLength(0)
    expect(wrapper.findAll('.radar-ring')).toHaveLength(5)
    expect(wrapper.findAll('.radar-label')).toHaveLength(5)

    await wrapper.setProps({ dimensions: profileDimensions({ music: 4 }), ready: true })
    expect(wrapper.find('.profile-empty').exists()).toBe(false)
    expect(wrapper.find('.profile-summary').text()).toBe('目前仅评了音乐 · 均分 4.0 星')
    const musicPoint = wrapper.find('.radar-point[data-dimension="music"]')
    expect(Number(musicPoint.attributes('cx'))).toBeCloseTo(144.785, 2)
    expect(Number(musicPoint.attributes('cy'))).toBeCloseTo(152.312, 2)

    await wrapper.setProps({ dimensions: profileDimensions() })
    expect(wrapper.find('.radar-area').exists()).toBe(false)
    expect(wrapper.find('.profile-empty').exists()).toBe(true)
  })

  it('groups tied dimensions in the summary and handles equal means', async () => {
    const wrapper = mount(TasteProfile, {
      props: {
        dimensions: profileDimensions({ story: 4, characters: 4, animation: 2, music: 2 }),
        ready: true
      }
    })
    expect(wrapper.find('.profile-summary').text()).toBe(
      '均分最高：剧情、角色 4.0 星 · 均分最低：作画、音乐 2.0 星'
    )

    await wrapper.setProps({ dimensions: profileDimensions({ story: 3, music: 3 }) })
    expect(wrapper.find('.profile-summary').text()).toBe('已评维度均分相同 · 3.0 星')
  })

  it('animates the radar values from the center when ready', async () => {
    const wrapper = mount(TasteProfile, {
      props: { dimensions: profileDimensions({ animation: 5 }), ready: false }
    })
    const area = wrapper.find('.radar-area')
    expect(area.exists()).toBe(true)
    const points = area.attributes('points')
    expect(wrapper.find('.radar-values').classes()).not.toContain('is-ready')

    await wrapper.setProps({ ready: true })
    expect(wrapper.find('.radar-values').classes()).toContain('is-ready')
    expect(wrapper.find('.radar-area').attributes('points')).toBe(points)
  })
})
