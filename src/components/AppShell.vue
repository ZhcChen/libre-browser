<script setup lang="ts">
import { computed } from "vue";
import SidebarNav from "./SidebarNav.vue";
import { resolveEffectiveTheme } from "../state/settings";
const isDark = computed(() => resolveEffectiveTheme() === "dark");
</script>

<template>
  <div class="relative flex h-auto min-h-screen w-full flex-col group/design-root design-root overflow-x-hidden" :class="isDark ? 'bg-[#111a22]' : 'bg-slate-50'">
    <div class="layout-container flex h-full grow flex-col">
      <div class="gap-1 px-6 flex flex-1 justify-start py-5 overflow-x-auto">
        <SidebarNav />
        <RouterView />
      </div>
    </div>
  </div>
</template>

<style scoped>
.design-root {
  --checkbox-tick-svg: url("data:image/svg+xml,%3csvg viewBox='0 0 16 16' fill='rgb(255,255,255)' xmlns='http://www.w3.org/2000/svg'%3e%3cpath d='M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z'/%3e%3c/svg%3e");
  font-family: Inter, "Noto Sans", sans-serif;
}
</style>
