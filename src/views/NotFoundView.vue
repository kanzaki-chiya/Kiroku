<script setup lang="ts">
import { Compass } from 'lucide-vue-next'
</script>

<template>
  <div class="page notfound">
    <Compass :size="40" class="icon" aria-hidden="true" />
    <h1 class="title">这一页不在记录里</h1>
    <p class="sub">链接可能有误，或者内容已被移动。</p>
    <RouterLink to="/library" class="btn btn-primary">回到番剧库</RouterLink>
  </div>
</template>

<style scoped>
.notfound {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-height: 70vh;
  text-align: center;
}

.icon {
  color: var(--muted);
  width: 44px;
  height: 44px;
  padding: 16px;
  background: var(--fill);
  border-radius: 50%;
  box-sizing: content-box;
}

.title {
  margin: 0;
  font-family: var(--font-display);
  font-size: 30px;
  line-height: 1.2;
  font-weight: 700;
  letter-spacing: -0.025em;
}

.sub {
  margin: 0 0 18px;
  font-size: 13px;
  color: var(--muted);
}
</style>
