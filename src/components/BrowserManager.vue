<script setup lang="ts">
import { reactive, ref, computed, onMounted, onBeforeUnmount } from "vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
 
 type BrowserProfile = {
   id: string;
   name: string;
   project: string;
   lastOpenedAt?: string;
   opened: boolean;
  fingerprint?: string;
  proxy?: string;
 };
 
const search = ref("");
const selected = reactive(new Set<string>());
const state = reactive({
   profiles: [] as BrowserProfile[],
 });
 
 const STORAGE_KEY = "libre_browser_profiles";
 
 function load() {
   try {
     const raw = localStorage.getItem(STORAGE_KEY);
     if (raw) state.profiles = JSON.parse(raw);
     else {
       state.profiles = [1, 2, 3, 4, 5].map((i) => ({
         id: `CHE-${i}`,
         name: `CHE-${i}`,
         project: "默认项目",
         opened: false,
       }));
       save();
     }
   } catch {
     state.profiles = [];
   }
 }
 
 function save() {
   localStorage.setItem(STORAGE_KEY, JSON.stringify(state.profiles));
 }
 
 function labelOf(p: BrowserProfile) {
   return `browser-${p.id}`;
 }
 
 async function isWindowOpen(label: string) {
   try {
     // @ts-ignore
     const win = typeof WebviewWindow.getByLabel === "function"
       ? // @ts-ignore
         WebviewWindow.getByLabel(label)
       : null;
     return !!win;
   } catch {
     return false;
   }
 }
 
 function edit(p: BrowserProfile) {
   const name = window.prompt("编辑窗口名称", p.name);
   if (name && name.trim()) {
     p.name = name.trim();
     save();
   }
 }
 
 function removeOne(p: BrowserProfile) {
   if (!confirm(`确认删除 ${p.name}？`)) return;
   state.profiles = state.profiles.filter((x) => x.id !== p.id);
   save();
 }
 
 function toggleOpen(p: BrowserProfile) {
   p.opened = !p.opened;
   if (p.opened) p.lastOpenedAt = new Date().toISOString();
   save();
 }

function createProfile() {
  const id = `CHE-${Date.now().toString().slice(-5)}`;
  state.profiles.push({ id, name: id, project: "默认项目", opened: false });
  save();
}
 
 let timer: number | undefined;
 onMounted(() => {
   load();
   timer = window.setInterval(async () => {
     for (const p of state.profiles) {
       const current = await isWindowOpen(labelOf(p));
       if (current !== p.opened) p.opened = current;
     }
   }, 1000);
 });
 
 onBeforeUnmount(() => {
   if (timer) window.clearInterval(timer);
 });
 
 const filtered = computed(() => {
   const q = search.value.trim().toLowerCase();
   if (!q) return state.profiles;
   return state.profiles.filter(
     (p) => p.id.toLowerCase().includes(q) || p.name.toLowerCase().includes(q)
   );
 });

function toggleRow(id: string) {
  if (selected.has(id)) selected.delete(id);
  else selected.add(id);
}

function bulkDelete() {
  if (selected.size === 0) return;
  if (!confirm(`确认删除选中的 ${selected.size} 个窗口？`)) return;
  state.profiles = state.profiles.filter((p) => !selected.has(p.id));
  selected.clear();
  save();
}

function bulkOpen() {
  for (const p of state.profiles) if (selected.has(p.id)) { p.opened = true; p.lastOpenedAt = new Date().toISOString(); }
  save();
}

function bulkClose() {
  for (const p of state.profiles) if (selected.has(p.id)) p.opened = false;
  save();
}
 </script>
 
<template>
  <div
    class="relative flex h-auto min-h-screen w-full flex-col bg-[#111a22] dark group/design-root design-root overflow-x-hidden"
  >
    <div class="layout-container flex h-full grow flex-col">
      <div class="gap-1 px-6 flex flex-1 justify-center py-5">
        <!-- 左侧栏 -->
        <div class="layout-content-container flex flex-col w-80">
          <div class="flex h-full min-h-[700px] flex-col justify-between bg-[#111a22] p-4">
            <div class="flex flex-col gap-4">
              <h1 class="text-white text-base font-medium leading-normal">浏览器管理器</h1>
              <div class="flex flex-col gap-2">
                <div class="flex items-center gap-3 px-3 py-2 rounded-lg bg-[#233648]">
                  <div class="text-white">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm78.37,64H170.94a142.39,142.39,0,0,0-20.26-45A88.37,88.37,0,0,1,206.37,88ZM128,40.11c12,13,21,29.55,26.37,47.89H101.63C107,69.66,116,53.13,128,40.11ZM96,128a145.29,145.29,0,0,1,2-24h60a145.72,145.72,0,0,1,0,48H98A145.29,145.29,0,0,1,96,128Zm5.63,40h52.74C149,186.34,140,202.87,128,215.89,116,202.87,107,186.34,101.63,168Zm49.05,45a142.39,142.39,0,0,0,20.26-45h35.43A88.37,88.37,0,0,1,150.68,213Zm23.53-61a161.79,161.79,0,0,0,0-48h38.46a88.15,88.15,0,0,1,0,48Z"/></svg>
                  </div>
                  <p class="text-white text-sm font-medium leading-normal">浏览器列表</p>
                </div>
                <div class="flex items-center gap-3 px-3 py-2">
                  <div class="text-white">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M213.92,210.62a8,8,0,1,1-11.84,10.76l-52-57.15a60,60,0,0,0-57.41,7.24,8,8,0,1,1-9.42-12.93A75.43,75.43,0,0,1,128,144c1.28,0,2.55,0,3.82.1L104.9,114.49A108,108,0,0,0,61,135.31,8,8,0,0,1,49.73,134,8,8,0,0,1,51,122.77a124.27,124.27,0,0,1,41.71-21.66L69.37,75.4a155.43,155.43,0,0,0-40.29,24A8,8,0,0,1,18.92,87,171.87,171.87,0,0,1,58,62.86L42.08,45.38A8,8,0,1,1,53.92,34.62ZM128,192a12,12,0,1,0,12,12A12,12,0,0,0,128,192ZM237.08,87A172.3,172.3,0,0,0,106,49.4a8,8,0,1,0,2,15.87A158.33,158.33,0,0,1,128,64a156.25,156.25,0,0,1,98.92,35.37A8,8,0,0,0,237.08,87ZM195,135.31a8,8,0,0,0,11.24-1.3,8,8,0,0,0-1.3-11.24,124.25,124.25,0,0,0-51.73-24.2A8,8,0,1,0,150,114.24,108.12,108.12,0,0,1,195,135.31Z"/></svg>
                  </div>
                  <p class="text-white text-sm font-medium leading-normal">代理设置</p>
                </div>
                <div class="flex items-center gap-3 px-3 py-2">
                  <div class="text-white">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M152,96H104a8,8,0,0,0-8,8v48a8,8,0,0,0,8,8h48a8,8,0,0,0,8-8V104A8,8,0,0,0,152,96Zm-8,48H112V112h32Zm88,0H216V112h16a8,8,0,0,0,0-16H216V56a16,16,0,0,0-16-16H160V24a8,8,0,0,0-16,0V40H112V24a8,8,0,0,0-16,0V40H56A16,16,0,0,0,40,56V96H24a8,8,0,0,0,0,16H40v32H24a8,8,0,0,0,0,16H40v40a16,16,0,0,0,16,16H96v16a8,8,0,0,0,16,0V216h32v16a8,8,0,0,0,16,0V216h40a16,16,0,0,0,16-16V160h16a8,8,0,0,0,0-16Zm-32,56H56V56H200v95.87s0,.09,0,.13,0,.09,0,.13V200Z"/></svg>
                  </div>
                  <p class="text-white text-sm font-medium leading-normal">内核管理</p>
                </div>
              </div>
            </div>
            <div class="flex flex-col gap-1">
              <div class="flex items-center gap-3 px-3 py-2">
                <div class="text-white">
                  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M128,80a48,48,0,1,0,48,48A48.05,48.05,0,0,0,128,80Zm0,80a32,32,0,1,1,32-32A32,32,0,0,1,128,160Zm88-29.84q.06-2.16,0-4.32l14.92-18.64a8,8,0,0,0,1.48-7.06,107.21,107.21,0,0,0-10.88-26.25,8,8,0,0,0-6-3.93l-23.72-2.64q-1.48-1.56-3-3L186,40.54a8,8,0,0,0-3.94-6,107.71,107.71,0,0,0-26.25-10.87,8,8,0,0,0-7.06,1.49L130.16,40Q128,40,125.84,40L107.2,25.11a8,8,0,0,0-7.06-1.48A107.6,107.6,0,0,0,73.89,34.51a8,8,0,0,0-3.93,6L67.32,64.27q-1.56,1.49-3,3L40.54,70a8,8,0,0,0-6,3.94,107.71,107.71,0,0,0-10.87,26.25,8,8,0,0,0,1.49,7.06L40,125.84Q40,128,40,130.16L25.11,148.8a8,8,0,0,0-1.48,7.06,107.21,107.21,0,0,0,10.88,26.25,8,8,0,0,0,6,3.93l23.72,2.64q1.49,1.56,3,3L70,215.46a8,8,0,0,0,3.94,6,107.71,107.71,0,0,0,26.25,10.87,8,8,0,0,0,7.06-1.49L125.84,216q2.16.06,4.32,0l18.64,14.92a8,8,0,0,0,7.06,1.48,107.21,107.21,0,0,0,26.25-10.88,8,8,0,0,0,3.93-6l2.64-23.72q1.56-1.48,3-3L215.46,186a8,8,0,0,0,6-3.94,107.71,107.71,0,0,0,10.87-26.25,8,8,0,0,0-1.49-7.06Zm-16.1-6.5a73.93,73.93,0,0,1,0,8.68,8,8,0,0,0,1.74,5.48l14.19,17.73a91.57,91.57,0,0,1-6.23,15L187,173.11a8,8,0,0,0-5.1,2.64,74.11,74.11,0,0,1-6.14,6.14,8,8,0,0,0-2.64,5.1l-2.51,22.58a91.32,91.32,0,0,1-15,6.23l-17.74-14.19a8,8,0,0,0-5-1.75h-.48a73.93,73.93,0,0,1-8.68,0,8,8,0,0,0-5.48,1.74L100.45,215.8a91.57,91.57,0,0,1-15-6.23L82.89,187a8,8,0,0,0-2.64-5.1,74.11,74.11,0,0,1-6.14-6.14,8,8,0,0,0-5.1-2.64L46.43,170.6a91.32,91.32,0,0,1-6.23-15l14.19-17.74a8,8,0,0,0,1.74-5.48,73.93,73.93,0,0,1,0-8.68,8,8,0,0,0-1.74-5.48L40.2,100.45a91.57,91.57,0,0,1,6.23-15L69,82.89a8,8,0,0,0,5.1-2.64,74.11,74.11,0,0,1,6.14-6.14A8,8,0,0,0,82.89,69L85.4,46.43a91.32,91.32,0,0,1,15-6.23l17.74,14.19a8,8,0,0,0,5.48,1.74,73.93,73.93,0,0,1,8.68,0,8,8,0,0,0,5.48-1.74L155.55,40.2a91.57,91.57,0,0,1,15,6.23L173.11,69a8,8,0,0,0,2.64,5.1,74.11,74.11,0,0,1,6.14,6.14,8,8,0,0,0,5.1,2.64l22.58,2.51a91.32,91.32,0,0,1,6.23,15l-14.19,17.74A8,8,0,0,0,199.87,123.66Z"/></svg>
                </div>
                <p class="text-white text-sm font-medium leading-normal">设置</p>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧内容区 -->
        <div class="layout-content-container flex flex-col max-w-[960px] flex-1">
          <div class="flex flex-wrap justify-between gap-3 p-4">
            <p class="text-white tracking-light text-[32px] font-bold leading-tight min-w-72">浏览器列表</p>
            <button class="flex min-w-[84px] max-w-[480px] cursor-pointer items-center justify-center overflow-hidden rounded-lg h-8 px-4 bg-[#233648] text-white text-sm font-medium leading-normal" @click="createProfile"><span class="truncate">新建浏览器</span></button>
          </div>
          <div class="px-4 py-3">
            <label class="flex flex-col min-w-40 h-12 w-full">
              <div class="flex w-full flex-1 items-stretch rounded-lg h-full">
                <div class="text-[#92adc9] flex border-none bg-[#233648] items-center justify-center pl-4 rounded-l-lg border-r-0">
                  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M229.66,218.34l-50.07-50.06a88.11,88.11,0,1,0-11.31,11.31l50.06,50.07a8,8,0,0,0,11.32-11.32ZM40,112a72,72,0,1,1,72,72A72.08,72.08,0,0,1,40,112Z"/></svg>
                </div>
                <input v-model="search" placeholder="搜索浏览器" class="form-input flex w-full min-w-0 flex-1 resize-none overflow-hidden rounded-lg text-white focus:outline-0 focus:ring-0 border-none bg-[#233648] focus:border-none h-full placeholder:text-[#92adc9] px-4 rounded-l-none border-l-0 pl-2 text-base font-normal leading-normal" />
              </div>
            </label>
          </div>
          <div class="flex justify-stretch">
            <div class="flex flex-1 gap-3 flex-wrap px-4 py-3 justify-start">
              <button class="flex min-w-[84px] max-w-[480px] cursor-pointer items-center justify-center overflow-hidden rounded-lg h-10 px-4 bg-[#233648] text-white text-sm font-bold leading-normal tracking-[0.015em]" @click="bulkOpen"><span class="truncate">批量启动</span></button>
              <button class="flex min-w-[84px] max-w-[480px] cursor-pointer items-center justify-center overflow-hidden rounded-lg h-10 px-4 bg-[#233648] text-white text-sm font-bold leading-normal tracking-[0.015em]" @click="bulkClose"><span class="truncate">批量关闭</span></button>
              <button class="flex min-w-[84px] max-w-[480px] cursor-pointer items-center justify-center overflow-hidden rounded-lg h-10 px-4 bg-[#233648] text-white text-sm font-bold leading-normal tracking-[0.015em]" @click="bulkDelete"><span class="truncate">批量删除</span></button>
            </div>
          </div>
          <div class="px-4 py-3 @container">
            <div class="flex overflow-hidden rounded-lg border border-[#324d67] bg-[#111a22]">
              <table class="flex-1">
                <thead>
                  <tr class="bg-[#192633]">
                    <th class="px-4 py-3 text-left text-white w-[120px] text-sm font-medium leading-normal">选择</th>
                    <th class="px-4 py-3 text-left text-white w-[400px] text-sm font-medium leading-normal">名称</th>
                    <th class="px-4 py-3 text-left text-white w-[400px] text-sm font-medium leading-normal">指纹</th>
                    <th class="px-4 py-3 text-left text-white w-[400px] text-sm font-medium leading-normal">代理</th>
                    <th class="px-4 py-3 text-left text-white w-60 text-[#92adc9] text-sm font-medium leading-normal">操作</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="p in filtered" :key="p.id" class="border-t border-t-[#324d67]">
                    <td class="h-[72px] px-4 py-2 w-[120px] text-center text-sm font-normal leading-normal">
                      <input type="checkbox" class="h-5 w-5 rounded border-[#324d67] border-2 bg-transparent text-[#1172d4] checked:bg-[#1172d4] checked:border-[#1172d4] focus:ring-0 focus:ring-offset-0 focus:border-[#324d67] focus:outline-none" :checked="selected.has(p.id)" @change="toggleRow(p.id)" />
                    </td>
                    <td class="h-[72px] px-4 py-2 w-[400px] text-white text-sm font-normal leading-normal">{{ p.name }}</td>
                    <td class="h-[72px] px-4 py-2 w-[400px] text-[#92adc9] text-sm font-normal leading-normal">{{ p.fingerprint || '未配置' }}</td>
                    <td class="h-[72px] px-4 py-2 w-[400px] text-[#92adc9] text-sm font-normal leading-normal">{{ p.proxy || '未配置' }}</td>
                    <td class="h-[72px] px-4 py-2 w-60 text-[#92adc9] text-sm font-bold leading-normal tracking-[0.015em]">
                      <button class="text-white mr-3" @click="toggleOpen(p)">{{ p.opened ? '关闭' : '启动' }}</button>
                      <button class="text-white mr-3" @click="edit(p)">编辑</button>
                      <button class="text-white" @click="removeOne(p)">删除</button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
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
