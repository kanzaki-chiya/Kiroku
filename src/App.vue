<script setup lang="ts">
import { ref } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'
import AppShell from './components/AppShell.vue'
import UpdateBanner from './components/UpdateBanner.vue'
import { vtBusy } from './services/motion'
import { useLibraryStore } from './stores/library'
import { useNotices } from './stores/notices'

const { notices } = useNotices()
const library = useLibraryStore()
const route = useRoute()
const router = useRouter()
const routeTransition = ref('route-fade')

router.beforeEach((to, from) => {
  if (vtBusy.value) {
    routeTransition.value = 'route-none'
    return
  }
  const toDepth = (to.meta.depth as number | undefined) ?? 0
  const fromDepth = (from.meta.depth as number | undefined) ?? 0
  routeTransition.value =
    toDepth > fromDepth ? 'route-forward' : toDepth < fromDepth ? 'route-back' : 'route-fade'
})
</script>

<template>
  <div v-if="library.bootError" class="boot-fail" role="alert">
    <h1 class="boot-title">无法启动 Kiroku</h1>
    <p class="boot-message">{{ library.bootError }}</p>
    <p class="boot-hint">数据文件未被覆盖。检查数据目录权限或磁盘空间后重新打开。</p>
  </div>
  <template v-else>
    <AppShell>
      <RouterView v-slot="{ Component }">
        <Transition :name="routeTransition" mode="out-in" appear>
          <component :is="Component" :key="route.fullPath" />
        </Transition>
      </RouterView>
    </AppShell>
    <UpdateBanner />
    <TransitionGroup name="notice" tag="div" class="notices" role="status" aria-live="polite">
      <div v-for="notice in notices" :key="notice.id" class="notice">{{ notice.text }}</div>
    </TransitionGroup>
  </template>
</template>

<style scoped>
.notices {
  position: fixed;
  right: 24px;
  bottom: 24px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  z-index: 60;
  max-width: min(440px, calc(100vw - 48px));
  pointer-events: none;
}

.notice {
  background: rgba(28, 28, 30, 0.88);
  -webkit-backdrop-filter: blur(18px) saturate(160%);
  backdrop-filter: blur(18px) saturate(160%);
  color: #f5f5f7;
  font-size: 13px;
  padding: 12px 18px;
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lift);
  overflow-wrap: anywhere;
}

.notice-move,
.notice-enter-active,
.notice-leave-active {
  transition: opacity 260ms var(--ease-spring), transform 260ms var(--ease-spring);
}

.notice-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.96);
}

.notice-leave-to {
  opacity: 0;
  transform: translateY(6px) scale(0.97);
}

.notice-leave-active {
  position: absolute;
  right: 0;
  width: 100%;
}

.boot-fail {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 40px;
  text-align: center;
  background: var(--workspace);
}

.boot-fail::before {
  content: '';
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  background: var(--brand-soft);
  box-shadow: var(--shadow-card);
  margin-bottom: 18px;
}

.boot-title {
  margin: 0;
  font: 700 30px/1.2 var(--font-display);
  letter-spacing: -0.02em;
}

.boot-message, .boot-hint {
  margin: 0;
  max-width: 36rem;
  color: var(--muted);
  font-size: 13px;
  line-height: 1.8;
  overflow-wrap: anywhere;
}

.boot-message {
  color: var(--danger);
}
</style>
