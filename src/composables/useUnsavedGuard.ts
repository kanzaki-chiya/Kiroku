import { onBeforeUnmount, onMounted, type Ref } from 'vue'
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router'

export function useUnsavedGuard(dirty: Ref<boolean>) {
  const confirmLeave = () =>
    !dirty.value || window.confirm('有未保存的修改，确定要离开吗？')

  const handler = (event: BeforeUnloadEvent) => {
    if (!dirty.value) return
    event.preventDefault()
    event.returnValue = ''
  }

  onMounted(() => window.addEventListener('beforeunload', handler))
  onBeforeUnmount(() => window.removeEventListener('beforeunload', handler))

  onBeforeRouteLeave(confirmLeave)
  onBeforeRouteUpdate(() => {
    if (!confirmLeave()) return false
    dirty.value = false
    return true
  })
}
