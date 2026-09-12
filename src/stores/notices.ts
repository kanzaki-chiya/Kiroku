import { ref } from 'vue'

export interface Notice {
  id: number
  text: string
}

const notices = ref<Notice[]>([])
let nextId = 1

export function useNotices() {
  function push(text: string) {
    const id = nextId++
    notices.value = [...notices.value, { id, text }]
    setTimeout(() => {
      notices.value = notices.value.filter(n => n.id !== id)
    }, 3600)
  }
  return { notices, push }
}
