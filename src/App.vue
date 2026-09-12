<script setup lang="ts">
import AppShell from './components/AppShell.vue'
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
  background: var(--sidebar-bg);
  color: var(--sidebar-text);
  font-size: 13px;
  padding: 14px 20px;
  border: 1px solid var(--sidebar-line);
  border-left: 3px solid var(--sidebar-muted);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-lift);
  overflow-wrap: anywhere;
  animation: notice-in 180ms ease-out;
}

@keyframes notice-in {
  from {
    opacity: 0;
    transform: translateY(6px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
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
  width: 34px;
  height: 42px;
  border: 2px solid var(--brand);
  border-radius: var(--radius-xs);
  box-shadow: 6px -6px 0 var(--brand-soft);
  margin-bottom: 18px;
}

.boot-title {
  margin: 0;
  font: 600 30px/1.5 var(--font-display);
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
