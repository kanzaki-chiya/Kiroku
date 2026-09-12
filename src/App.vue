<script setup lang="ts">
import AppShell from './components/AppShell.vue'
import UpdateBanner from './components/UpdateBanner.vue'
import { useLibraryStore } from './stores/library'
import { useNotices } from './stores/notices'

const { notices } = useNotices()
const library = useLibraryStore()
</script>

<template>
  <div v-if="library.bootError" class="boot-fail" role="alert">
    <h1 class="boot-title">无法启动 Kiroku</h1>
    <p class="boot-message">{{ library.bootError }}</p>
    <p class="boot-hint">数据文件未被覆盖。检查数据目录权限或磁盘空间后重新打开。</p>
  </div>
  <template v-else>
    <AppShell>
      <RouterView />
    </AppShell>
    <UpdateBanner />
    <div class="notices" role="status" aria-live="polite">
      <div v-for="notice in notices" :key="notice.id" class="notice">{{ notice.text }}</div>
    </div>
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
  animation: notice-in 300ms var(--ease-spring);
}

@keyframes notice-in {
  from {
    opacity: 0;
    transform: translateY(10px) scale(0.96);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
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
