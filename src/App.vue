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
  right: 26px;
  bottom: 24px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  z-index: 60;
}

.notice {
  background: var(--text);
  color: #f2f6f3;
  font-size: 14px;
  padding: 11px 18px;
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-lift);
}

.boot-fail {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 40px;
  text-align: center;
}

.boot-title {
  margin: 0;
  font-size: 28px;
}

.boot-message,
.boot-hint {
  margin: 0;
  max-width: 36rem;
  color: var(--muted);
  line-height: 1.6;
}
</style>
