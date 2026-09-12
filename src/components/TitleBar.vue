<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Copy, Minus, Square, X } from 'lucide-vue-next'

const appWindow = getCurrentWindow()
const maximized = ref(false)

let unlisten: (() => void) | null = null

onMounted(async () => {
  maximized.value = await appWindow.isMaximized()
  unlisten = await appWindow.onResized(async () => {
    maximized.value = await appWindow.isMaximized()
  })
})

onBeforeUnmount(() => {
  unlisten?.()
})

function minimize() {
  void appWindow.minimize()
}

function toggleMaximize() {
  void appWindow.toggleMaximize()
}

function close() {
  void appWindow.close()
}
</script>

<template>
  <div class="titlebar">
    <div class="drag" data-tauri-drag-region @dblclick="toggleMaximize" />
    <div class="controls">
      <button type="button" class="win-btn" aria-label="最小化" @click="minimize">
        <Minus :size="15" aria-hidden="true" />
      </button>
      <button
        type="button"
        class="win-btn"
        :aria-label="maximized ? '还原' : '最大化'"
        @click="toggleMaximize"
      >
        <Copy v-if="maximized" :size="12" aria-hidden="true" />
        <Square v-else :size="13" aria-hidden="true" />
      </button>
      <button type="button" class="win-btn close" aria-label="关闭" @click="close">
        <X :size="16" aria-hidden="true" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  position: sticky;
  top: 0;
  z-index: 50;
  display: flex;
  align-items: stretch;
  height: 38px;
  flex-shrink: 0;
  background: rgba(245, 245, 247, 0.78);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  backdrop-filter: blur(20px) saturate(180%);
  user-select: none;
  -webkit-user-select: none;
}

.drag {
  flex: 1;
  min-width: 0;
}

.controls {
  display: flex;
  align-items: stretch;
  flex-shrink: 0;
}

.win-btn {
  width: 46px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-soft);
  transition: background 120ms ease-out, color 120ms ease-out;
}

.win-btn:hover {
  background: var(--fill-strong);
  color: var(--text);
}

.win-btn:active {
  background: rgba(0, 0, 0, 0.12);
}

.win-btn.close:hover {
  background: #e81123;
  color: #fff;
}
</style>
