import { defineStore } from 'pinia'
import { ref } from 'vue'
import { createDefaultFilters } from '../utils/library'

export const useLibraryViewStore = defineStore('library-view', () => {
  const filters = ref(createDefaultFilters())
  const layout = ref<'grid' | 'list'>('grid')

  function resetFilters() {
    filters.value = createDefaultFilters()
  }

  return { filters, layout, resetFilters }
})
