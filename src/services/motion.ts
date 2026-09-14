import { nextTick, ref } from 'vue'
import type { Router } from 'vue-router'

/** 正在参与封面共享转场的卡片 subjectId；非转场期间为 null */
export const morphCardId = ref<number | null>(null)

/** View Transition 进行中时为 true，用来跳过 Vue 的路由过渡（避免双重动画） */
export const vtBusy = ref(false)

interface ViewTransitionLike {
  ready: Promise<void>
  finished: Promise<void>
  skipTransition: () => void
}

type DocumentWithVT = Document & {
  startViewTransition?: (callback: () => void | Promise<void>) => ViewTransitionLike
}

export function canMorph(): boolean {
  return (
    typeof document !== 'undefined' &&
    'startViewTransition' in document &&
    !window.matchMedia('(prefers-reduced-motion: reduce)').matches
  )
}

/**
 * 在 View Transition 中执行路由跳转。调用方负责：
 * - 旧态：跳转前给源元素加 .vt-source（同步，确保旧帧捕获到）
 * - 新态：通过 morphCardId 让目标元素拿到 .vt-source（回调内 nextTick 后捕获新帧）
 * onDone 在动画结束（或被跳过）后清理状态。
 */
export function runMorphNav(router: Router, to: string, onDone: () => void): void {
  vtBusy.value = true
  const vt = (document as DocumentWithVT).startViewTransition!(async () => {
    await router.push(to)
    await nextTick()
  })
  vt.finished.then(
    () => {
      vtBusy.value = false
      onDone()
    },
    () => {
      vtBusy.value = false
      onDone()
    }
  )
  vt.ready.catch(() => {})
}
