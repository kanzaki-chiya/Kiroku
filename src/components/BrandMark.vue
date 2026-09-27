<script setup lang="ts">
import { useId } from 'vue'

// 同页可能渲染多个实例，mask id 需唯一
const uid = useId()
const backMask = `brand-back-${uid}`
const frontMask = `brand-front-${uid}`

const STAR =
  'M23 15.9L25.12 20.39L30.04 21.01L26.42 24.41L27.35 29.29L23 26.9L18.65 29.29L19.58 24.41L15.96 21.01L20.88 20.39Z'
</script>

<template>
  <svg
    viewBox="0 0 32 32"
    fill="none"
    stroke="currentColor"
    stroke-width="1.5"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <defs>
      <mask :id="backMask" maskUnits="userSpaceOnUse" x="0" y="0" width="32" height="32">
        <rect width="32" height="32" fill="#fff" stroke="none" />
        <rect x="4" y="8" width="15" height="20" rx="2.5" fill="#000" stroke="#000" stroke-width="4" />
        <path :d="STAR" fill="#000" stroke="#000" stroke-width="4.5" />
      </mask>
      <mask :id="frontMask" maskUnits="userSpaceOnUse" x="0" y="0" width="32" height="32">
        <rect width="32" height="32" fill="#fff" stroke="none" />
        <path :d="STAR" fill="#000" stroke="#000" stroke-width="4.5" />
      </mask>
    </defs>
    <!-- 身后的收藏卡 -->
    <rect x="9" y="4" width="15" height="20" rx="2.5" :mask="`url(#${backMask})`" />
    <!-- 正面封面卡：太阳 + 山丘 + 标题行 -->
    <g :mask="`url(#${frontMask})`">
      <rect x="4" y="8" width="15" height="20" rx="2.5" />
      <circle cx="8.6" cy="12.6" r="1.5" />
      <path d="M4 20.5C6.5 18 9 18 11.5 20.5S16.5 23 19 20.5" />
      <path d="M7.5 24.6H11" />
    </g>
    <path :d="STAR" fill="currentColor" stroke="none" />
  </svg>
</template>
