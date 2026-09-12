<script setup lang="ts">
import { isTauri } from '../runtime'
import AppSidebar from './AppSidebar.vue'

const desktop = isTauri()
</script>

<template>
  <div class="shell">
    <AppSidebar />
    <div class="main">
      <div v-if="!desktop" class="demo-banner" role="note">
        演示模式 · 修改仅在本次打开期间保留，刷新后恢复示例。资料及社区评分均为 Mock。
      </div>
      <main class="outlet">
        <slot />
      </main>
    </div>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  min-height: 100vh;
}

.main {
  flex: 1;
  min-width: 0;
  background: var(--workspace);
  display: flex;
  flex-direction: column;
}

.demo-banner {
  font-size: 11px;
  color: var(--muted);
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  padding: 7px var(--page-gutter);
  text-align: right;
  line-height: 1.7;
}

.outlet {
  flex: 1;
  min-width: 0;
}
</style>
