import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, type Pinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import App from '../src/App.vue'
import { routes } from '../src/router'
import { useLibraryStore } from '../src/stores/library'

const wrappers: VueWrapper[] = []

async function mountApp(path: string): Promise<{
  wrapper: VueWrapper
  router: Router
  pinia: Pinia
}> {
  const pinia = createPinia()
  const router = createRouter({ history: createMemoryHistory(), routes })
  router.push(path)
  await router.isReady()
  const wrapper = mount(App, { global: { plugins: [pinia, router] } })
  await flushPromises()
  wrappers.push(wrapper)
  return { wrapper, router, pinia }
}

afterEach(() => {
  wrappers.splice(0).forEach(w => w.unmount())
  vi.restoreAllMocks()
})

describe('edit route navigation', () => {
  it('confirms before switching to another edit route and keeps records isolated', async () => {
    const { wrapper, router, pinia } = await mountApp('/anime/52991/edit')
    const store = useLibraryStore(pinia)
    const confirmMock = vi.fn(() => false)
    window.confirm = confirmMock

    const scoreInput = wrapper.find('input[type="number"]')
    expect((scoreInput.element as HTMLInputElement).value).toBe('9.6')
    await scoreInput.setValue('7')

    await router.push('/anime/47917/edit')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/anime/52991/edit')

    confirmMock.mockReturnValue(true)
    await router.push('/anime/47917/edit')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/anime/47917/edit')

    const freshInput = wrapper.find('input[type="number"]')
    expect((freshInput.element as HTMLInputElement).value).toBe('8.8')

    confirmMock.mockClear()
    await freshInput.setValue('9.9')
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(router.currentRoute.value.path).toBe('/anime/47917')
    expect(store.getEntry(47917)!.personal.score).toBe(9.9)
    expect(store.getEntry(47917)!.subject.nameCn).toBe('孤独摇滚！')
    expect(store.getEntry(52991)!.personal.score).toBe(9.6)
    expect(store.getEntry(52991)!.subject.nameCn).toBe('葬送的芙莉莲')
  })
})

describe('library view state', () => {
  it('retains filters and layout across a detail detour and keeps tier query in sync', async () => {
    const { wrapper, router } = await mountApp('/library')

    await wrapper.find('input[type="search"]').setValue('孤独')
    await wrapper.find('button[aria-label="列表视图"]').trigger('click')
    expect(wrapper.findAll('a.card').length).toBe(1)

    await router.push('/anime/47917')
    await flushPromises()
    await router.push('/library')
    await flushPromises()

    expect((wrapper.find('input[type="search"]').element as HTMLInputElement).value).toBe('孤独')
    expect(wrapper.find('button[aria-label="列表视图"]').attributes('aria-pressed')).toBe('true')
    expect(wrapper.findAll('a.card').length).toBe(1)

    await router.push({ path: '/library', query: { tier: 'S' } })
    await flushPromises()

    await wrapper.find('.filter-toggle').trigger('click')
    const tierSelect = wrapper.findAll('.filter-row select')[0]
    expect((tierSelect.element as HTMLSelectElement).value).toBe('S')

    const clearBtn = wrapper
      .findAll('button')
      .find(b => b.text().includes('清空筛选'))
    await clearBtn!.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.query.tier).toBeUndefined()
    expect((tierSelect.element as HTMLSelectElement).value).toBe('all')
  })
})
