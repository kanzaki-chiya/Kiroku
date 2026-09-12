<script setup lang="ts">
import {
  dismissUpdate,
  installUpdate,
  pendingUpdate,
  updateFailed,
  updateInstalling,
  updateProgress
} from '../services/updater'
</script>

<template>
  <Transition name="update-pop">
    <div v-if="pendingUpdate" class="update-banner" role="alertdialog" aria-label="应用更新">
      <div class="update-text">
        <p class="update-title">发现新版本 v{{ pendingUpdate.version }}</p>
        <p class="update-sub">
          <template v-if="updateInstalling">
            <template v-if="updateProgress !== null">正在下载… {{ updateProgress }}%</template>
            <template v-else>正在下载…</template>
          </template>
          <template v-else-if="updateFailed">更新失败，请稍后再试</template>
          <template v-else>{{ pendingUpdate.notes || '包含最新功能与修复' }}</template>
        </p>
      </div>
      <div class="update-actions">
        <button type="button" class="update-btn primary" :disabled="updateInstalling" @click="installUpdate">
          {{ updateInstalling ? '更新中' : updateFailed ? '重试' : '立即更新' }}
        </button>
        <button v-if="!updateInstalling" type="button" class="update-btn" @click="dismissUpdate">稍后</button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.update-banner {
  position: fixed;
  right: 24px;
  bottom: 24px;
  z-index: 70;
  display: flex;
  align-items: center;
  gap: 16px;
  max-width: min(440px, calc(100vw - 48px));
  padding: 14px 16px;
  border-radius: var(--radius-md);
  background: var(--surface);
  border: 1px solid var(--line);
  box-shadow: var(--shadow-lift);
}

.update-text {
  min-width: 0;
}

.update-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.update-sub {
  margin: 3px 0 0;
  font-size: 12px;
  color: var(--muted);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.update-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.update-btn {
  padding: 6px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-soft);
  background: var(--fill);
  border-radius: 980px;
  transition: transform 100ms ease-out, opacity var(--motion-fast) ease;
}

.update-btn:active {
  transform: scale(0.96);
}

.update-btn:disabled {
  opacity: 0.55;
  cursor: default;
}

.update-btn.primary {
  background: var(--brand);
  color: #fff;
}

.update-pop-enter-active,
.update-pop-leave-active {
  transition: opacity 300ms var(--ease-spring), transform 300ms var(--ease-spring);
}

.update-pop-enter-from,
.update-pop-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.96);
}

@media (prefers-reduced-motion: reduce) {
  .update-pop-enter-active,
  .update-pop-leave-active,
  .update-btn {
    transition: none;
  }
}
</style>
