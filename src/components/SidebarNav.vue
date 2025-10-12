<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from "vue";
import { useRoute } from "vue-router";
import { resolveEffectiveTheme } from "../state/settings";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
const route = useRoute();

const usedCount = ref(0); // 已创建窗口数
const maxQuota = ref(100); // 设置中的最大可创建数量
const percent = ref(0);

const isDark = computed(() => resolveEffectiveTheme() === "dark");

function computeUsage() {
  try {
    const raw = localStorage.getItem("libre_browser_profiles");
    const arr = raw ? (JSON.parse(raw) as Array<Record<string, unknown>>) : [];
    usedCount.value = arr.length;

    const sraw = localStorage.getItem("libre_settings");
    if (sraw) {
      const s = JSON.parse(sraw) as Partial<{ maxBrowsers: number }>;
      if (typeof s.maxBrowsers === "number" && s.maxBrowsers > 0) maxQuota.value = s.maxBrowsers;
    }

    const denom = maxQuota.value > 0 ? maxQuota.value : 100;
    percent.value = Math.max(0, Math.min(100, Math.round((usedCount.value / denom) * 100)));
  } catch {
    usedCount.value = 0;
    maxQuota.value = 100;
    percent.value = 0;
  }
}

let timer: number | undefined;
onMounted(() => {
  computeUsage();
  timer = window.setInterval(computeUsage, 1000);
});
onBeforeUnmount(() => { if (timer) window.clearInterval(timer); });

async function openLogs() {
  try {
    const p = await invoke<string>("logs_dir");
    await openPath(p);
  } catch (e) {
    console.error("open logs failed", e);
  }
}
</script>

<template>
  <div class="layout-content-container flex flex-col w-[240px] shrink-0">
    <div class="flex h-full min-h-[700px] flex-col justify-between p-4" :class="isDark ? 'bg-[#111a22]' : 'bg-slate-50'">
      <div class="flex flex-col gap-4">
        <h1 class="text-base font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">浏览器管理器</h1>
        <div class="flex flex-col gap-2">
          <RouterLink
            to="/browsers"
            class="flex items-center gap-3 px-3 py-2 rounded-lg"
            :class="[route.path.startsWith('/browsers') ? (isDark ? 'bg-[#233648]' : 'bg-[#e7edf3]') : '', isDark ? 'text-white' : 'text-[#0d141b]']"
          >
            <div :class="isDark ? 'text-white' : 'text-[#0d141b]'">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm78.37,64H170.94a142.39,142.39,0,0,0-20.26-45A88.37,88.37,0,0,1,206.37,88ZM128,40.11c12,13,21,29.55,26.37,47.89H101.63C107,69.66,116,53.13,128,40.11ZM96,128a145.29,145.29,0,0,1,2-24h60a145.72,145.72,0,0,1,0,48H98A145.29,145.29,0,0,1,96,128Zm5.63,40h52.74C149,186.34,140,202.87,128,215.89,116,202.87,107,186.34,101.63,168Zm49.05,45a142.39,142.39,0,0,0,20.26-45h35.43A88.37,88.37,0,0,1,150.68,213Zm23.53-61a161.79,161.79,0,0,0,0-48h38.46a88.15,88.15,0,0,1,0,48Z"/></svg>
            </div>
            <p class="text-sm font-medium leading-normal">浏览器列表</p>
          </RouterLink>

          <RouterLink to="/proxies" class="flex items-center gap-3 px-3 py-2 rounded-lg" :class="[route.path.startsWith('/proxies') ? (isDark ? 'bg-[#233648]' : 'bg-[#e7edf3]') : '', isDark ? 'text-white' : 'text-[#0d141b]']">
            <div :class="isDark ? 'text-white' : 'text-[#0d141b]'">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M213.92,210.62a8,8,0,1,1-11.84,10.76l-52-57.15a60,60,0,0,0-57.41,7.24,8,8,0,1,1-9.42-12.93A75.43,75.43,0,0,1,128,144c1.28,0,2.55,0,3.82.1L104.9,114.49A108,108,0,0,0,61,135.31,8,8,0,0,1,49.73,134,8,8,0,0,1,51,122.77a124.27,124.27,0,0,1,41.71-21.66L69.37,75.4a155.43,155.43,0,0,0-40.29,24A8,8,0,0,1,18.92,87,171.87,171.87,0,0,1,58,62.86L42.08,45.38A8,8,0,1,1,53.92,34.62ZM128,192a12,12,0,1,0,12,12A12,12,0,0,0,128,192ZM237.08,87A172.3,172.3,0,0,0,106,49.4a8,8,0,1,0,2,15.87A158.33,158.33,0,0,1,128,64a156.25,156.25,0,0,1,98.92,35.37A8,8,0,0,0,237.08,87ZM195,135.31a8,8,0,0,0,11.24-1.3,8,8,0,0,0-1.3-11.24,124.25,124.25,0,0,0-51.73-24.2A8,8,0,1,0,150,114.24,108.12,108.12,0,0,1,195,135.31Z"/></svg>
            </div>
            <p class="text-sm font-medium leading-normal">代理设置</p>
          </RouterLink>

          <RouterLink to="/engines" class="flex items-center gap-3 px-3 py-2 rounded-lg" :class="[route.path.startsWith('/engines') ? (isDark ? 'bg-[#233648]' : 'bg-[#e7edf3]') : '', isDark ? 'text-white' : 'text-[#0d141b]']">
            <div :class="isDark ? 'text-white' : 'text-[#0d141b]'">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M152,96H104a8,8,0,0,0-8,8v48a8,8,0,0,0,8,8h48a8,8,0,0,0,8-8V104A8,8,0,0,0,152,96Zm-8,48H112V112h32Zm88,0H216V112h16a8,8,0,0,0,0-16H216V56a16,16,0,0,0-16-16H160V24a8,8,0,0,0-16,0V40H112V24a8,8,0,0,0-16,0V40H56A16,16,0,0,0,40,56V96H24a8,8,0,0,0,0,16H40v32H24a8,8,0,0,0,0,16H40v40a16,16,0,0,0,16,16H96v16a8,8,0,0,0,16,0V216h32v16a8,8,0,0,0,16,0V216h40a16,16,0,0,0,16-16V160h16a8,8,0,0,0,0-16Zm-32,56H56V56H200v95.87s0,.09,0,.13,0,.09,0,.13V200Z"/></svg>
            </div>
            <p class="text-sm font-medium leading-normal">内核管理</p>
          </RouterLink>

          <RouterLink to="/settings" class="flex items-center gap-3 px-3 py-2 rounded-lg" :class="[route.path.startsWith('/settings') ? (isDark ? 'bg-[#233648]' : 'bg-[#e7edf3]') : '', isDark ? 'text-white' : 'text-[#0d141b]']">
            <div :class="isDark ? 'text-white' : 'text-[#0d141b]'">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M128,80a48,48,0,1,0,48,48A48.05,48.05,0,0,0,128,80Zm0,80a32,32,0,1,1,32-32A32,32,0,0,1,128,160Zm88-29.84q.06-2.16,0-4.32l14.92-18.64a8,8,0,0,0,1.48-7.06,107.21,107.21,0,0,0-10.88-26.25,8,8,0,0,0-6-3.93l-23.72-2.64q-1.48-1.56-3-3L186,40.54a8,8,0,0,0-3.94-6,107.71,107.71,0,0,0-26.25-10.87,8,8,0,0,0-7.06,1.49L130.16,40Q128,40,125.84,40L107.2,25.11a8,8,0,0,0-7.06-1.48A107.6,107.6,0,0,0,73.89,34.51a8,8,0,0,0-3.93,6L67.32,64.27q-1.56,1.49-3,3L40.54,70a8,8,0,0,0-6,3.94,107.71,107.71,0,0,0-10.87,26.25,8,8,0,0,0,1.49,7.06L40,125.84Q40,128,40,130.16L25.11,148.8a8,8,0,0,0-1.48,7.06,107.21,107.21,0,0,0,10.88,26.25,8,8,0,0,0,6,3.93l23.72,2.64q1.49,1.56,3,3L70,215.46a8,8,0,0,0,3.94,6,107.71,107.71,0,0,0,26.25,10.87,8,8,0,0,0,7.06-1.49L125.84,216q2.16.06,4.32,0l18.64,14.92a8,8,0,0,0,7.06,1.48,107.21,107.21,0,0,0,26.25-10.88,8,8,0,0,0,3.93-6l2.64-23.72q1.56-1.48,3-3L215.46,186a8,8,0,0,0,6-3.94,107.71,107.71,0,0,0,10.87-26.25,8,8,0,0,0-1.49-7.06Zm-16.1-6.5a73.93,73.93,0,0,1,0,8.68,8,8,0,0,0,1.74,5.48l14.19,17.73a91.57,91.57,0,0,1-6.23,15L187,173.11a8,8,0,0,0-5.1,2.64,74.11,74.11,0,0,1-6.14,6.14,8,8,0,0,0-2.64,5.1l-2.51,22.58a91.32,91.32,0,0,1-15,6.23l-17.74-14.19a8,8,0,0,0-5-1.75h-.48a73.93,73.93,0,0,1-8.68,0,8,8,0,0,0-5.48,1.74L100.45,215.8a91.57,91.57,0,0,1-15-6.23L82.89,187a8,8,0,0,0-2.64-5.1,74.11,74.11,0,0,1-6.14-6.14,8,8,0,0,0-5.1-2.64L46.43,170.6a91.32,91.32,0,0,1-6.23-15l14.19-17.74a8,8,0,0,0,1.74-5.48,73.93,73.93,0,0,1,0-8.68,8,8,0,0,0-1.74-5.48L40.2,100.45a91.57,91.57,0,0,1,6.23-15L69,82.89a8,8,0,0,0,5.1-2.64,74.11,74.11,0,0,1,6.14-6.14A8,8,0,0,0,82.89,69L85.4,46.43a91.32,91.32,0,0,1,15-6.23l17.74,14.19a8,8,0,0,0,5.48,1.74,73.93,73.93,0,0,1,8.68,0,8,8,0,0,0,5.48-1.74L155.55,40.2a91.57,91.57,0,0,1,15,6.23L173.11,69a8,8,0,0,0,2.64,5.1,74.11,74.11,0,0,1,6.14,6.14,8,8,0,0,0,2.64,5.1l22.58,2.51a91.32,91.32,0,0,1,6.23,15l-14.19,17.74A8,8,0,0,0,199.87,123.66Z"/></svg>
            </div>
            <p class="text-sm font-medium leading-normal">设置</p>
          </RouterLink>

          <button @click="openLogs" class="flex items-center gap-3 px-3 py-2 rounded-lg text-left" :class="[isDark ? 'text-white hover:bg-[#233648]' : 'text-[#0d141b] hover:bg-[#e7edf3]']">
            <div :class="isDark ? 'text-white' : 'text-[#0d141b]'">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M224,56V200a16,16,0,0,1-16,16H48a16,16,0,0,1-16-16V56A16,16,0,0,1,48,40H80a8,8,0,0,1,0,16H48V200H208V56H176a8,8,0,0,1,0-16h32A16,16,0,0,1,224,56ZM168,120a8,8,0,0,0-8-8H96a8,8,0,0,0,0,16h64A8,8,0,0,0,168,120Zm0,40a8,8,0,0,0-8-8H96a8,8,0,0,0,0,16h64A8,8,0,0,0,168,160ZM88,88a12,12,0,1,0-12-12A12,12,0,0,0,88,88Z"/></svg>
            </div>
            <p class="text-sm font-medium leading-normal">查看日志</p>
          </button>
        </div>
      </div>
      <div class="mt-auto text-xs">
        <div class="h-2 w-full rounded" :class="isDark ? 'bg-[#233648]' : 'bg-[#e7edf3]'">
          <div class="h-2 bg-[#60a5fa] rounded" :style="{ width: percent + '%' }"></div>
        </div>
        <div class="flex items-center justify-between mt-2">
          <div :class="isDark ? 'text-white' : 'text-[#0d141b]'">浏览器用量</div>
          <div :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ usedCount }} / {{ maxQuota }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
</style>
