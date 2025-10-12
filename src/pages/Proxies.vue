<script setup lang="ts">
import { computed } from "vue";
import { resolveEffectiveTheme } from "../state/settings";
const isDark = computed(() => resolveEffectiveTheme() === "dark");
</script>
<template>
  <div class="layout-content-container flex flex-col flex-1 min-w-[600px] shrink-0 p-4" :class="isDark ? 'text-white' : 'text-[#0d141b]'">
    <h2 class="text-2xl font-bold" :class="isDark ? 'text-white' : 'text-[#0d141b]'">代理设置</h2>
    <p class="mt-2" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">此页面为占位，后续实现代理管理功能。</p>
  </div>
</template>
<style scoped></style>
