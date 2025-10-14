<script setup lang="ts">
import { reactive, ref, computed, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { resolveEffectiveTheme } from "../state/settings";
import BrowserCreateModal from "../components/BrowserCreateModal.vue";
import AppTable from "../components/AppTable.vue";
import Modal from "../components/Modal.vue";

type BrowserStatus = "closed" | "closing" | "open" | "opening";

type BrowserProfile = {
  id: string;
  name: string;
  project: string;
  lastOpenedAt?: string;
  opened: boolean;
  fingerprint?: string;
  proxy?: string;
  status?: BrowserStatus;
  engineVersion?: string;
  pid?: number;
};

const search = ref("");
const selected = reactive(new Set<string>());
const state = reactive({ profiles: [] as BrowserProfile[] });
const STORAGE_KEY = "libre_browser_profiles";
// 最短过渡展示时长，保证“开启中/关闭中”至少可见一帧
const TRANSIENT_MIN_MS = 150;
// 记录每个 profile 进入过渡态的时间戳
const transitionAt = new Map<string, number>();

function load() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const arr = JSON.parse(raw) as BrowserProfile[];
      state.profiles = arr.map((p) => ({
        ...p,
        status: p.status || (p.opened ? "open" : "closed"),
      }));
    } else {
      state.profiles = [];
      save();
    }
  } catch {
    state.profiles = [];
  }
}
function save() { localStorage.setItem(STORAGE_KEY, JSON.stringify(state.profiles)); }
// 延迟保存，避免在点击瞬间阻塞渲染
let saveTimer: number | undefined;
function saveLater(delay = 0) {
  if (saveTimer) window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => { saveTimer = undefined; save(); }, delay);
}
// 保留历史结构，实际存在检测使用 p.id
async function isWindowOpen(label: string) {
  try { return await invoke<boolean>("browser_exists", { label }); } catch { return false; }
}
function edit(p: BrowserProfile) { const name = window.prompt("编辑窗口名称", p.name); if (name && name.trim()) { p.name = name.trim(); save(); } }
const confirmVisible = ref(false);
const confirmMsg = ref("");
let confirmAction: null | (() => Promise<void> | void) = null;
async function onConfirm() { const fn = confirmAction; confirmVisible.value = false; confirmAction = null; if (fn) await fn(); }
async function removeOne(p: BrowserProfile) {
  confirmMsg.value = `确认删除 ${p.name}？`;
  confirmAction = async () => {
    try { if (p.status === "open" || p.status === "opening") { try { await invoke("browser_close", { label: p.id }); } catch {} } }
    finally { state.profiles = state.profiles.filter((x) => x.id !== p.id); selected.delete(p.id); save(); }
  };
  confirmVisible.value = true;
}
async function toggleOpen(p: BrowserProfile) {
  if (p.status === "opening" || p.status === "closing") return;
  try {
    if (p.status === "open") {
      // 立即设置为关闭中状态
      p.status = "closing";
      transitionAt.set(p.id, Date.now());
      p.opened = false;
      saveLater(0);
      // 启动高频轮询
      startFastPolling(p.id);
      // 异步执行关闭命令，不阻塞UI更新
      invoke("browser_close", { label: p.id }).catch(async (err: any) => {
        // 关闭失败，恢复为开启状态
        p.status = "open";
        p.opened = true;
        save();
        stopFastPolling(p.id);
        transitionAt.delete(p.id);
        try { await invoke("log_info", { message: `[BrowserList] close failed: ${err?.message || String(err)}` }); } catch {}
      });
      // 不立即设置为 closed，让高频轮询检测到关闭后自动更新
    } else {
      p.status = "opening";
      transitionAt.set(p.id, Date.now());
      saveLater(0);
      // 启动高频轮询
      startFastPolling(p.id);
      await invoke("log_info", { message: `[BrowserList] try open label=${p.id} version=${p.engineVersion || 'N/A'}` });
      const pid = await invoke<number | null>("browser_open", { label: p.id, url: null, version: p.engineVersion || null });
      if (pid && typeof pid === 'number') { (p as any).pid = pid; }
      // 不直接置为 open，交由轮询在达到最短展示时长后切换
    }
  } catch (e:any) {
    await invoke("log_info", { message: `[BrowserList] open/close failed: ${e?.message || String(e)}` });
    try { const tail = await invoke<string>("read_logs_tail", { lines: 120 }); console.error(tail); } catch {}
    p.status = p.opened ? "open" : "closed";
    stopFastPolling(p.id);
    transitionAt.delete(p.id);
  }
}
function statusLabel(s?: BrowserStatus) {
  switch (s) {
    case "open": return "开启";
    case "opening": return "开启中";
    case "closing": return "关闭中";
    default: return "关闭";
  }
}
function statusClass(s?: BrowserStatus) {
  switch (s) {
    case "open": return "text-green-400";
    case "opening": return "text-yellow-400";
    case "closing": return "text-yellow-400";
    default: return "text-gray-400";
  }
}
const showCreate = ref(false);
function createProfile() { showCreate.value = true; }

let timer: number | undefined;
// 高频轮询管理：每个浏览器可以有自己的独立高频轮询
const fastPollingTimers = reactive(new Map<string, { timer: number; startTime: number }>());

// 启动单个浏览器的高频轮询（10ms 间隔，最长 10 秒）
function startFastPolling(profileId: string) {
  // 如果已存在，先清除
  stopFastPolling(profileId);
  
  const startTime = Date.now();
  const timer = window.setInterval(async () => {
    // 检查是否超过 10 秒
    if (Date.now() - startTime > 10000) {
      stopFastPolling(profileId);
      return;
    }
    
    // 检查这个 profile 的状态
    const p = state.profiles.find(x => x.id === profileId);
    if (!p) {
      stopFastPolling(profileId);
      return;
    }
    
    // 检测状态
    let cur = false;
    try { 
      const running = await invoke<number | null>("browser_running", { label: p.id }); 
      if (typeof running === 'number' && running > 0) { 
        (p as any).pid = running; 
        cur = true; 
      } 
    } catch {}
    
    if (!cur) { 
      try { cur = await isWindowOpen(p.id); } catch {} 
    }
    
    const started = transitionAt.get(profileId) || 0;
    const allowSwitch = !started || (Date.now() - started >= TRANSIENT_MIN_MS);
    // 更新状态
    if (p.status === "opening" && cur && allowSwitch) { 
      p.status = "open"; 
      p.opened = true; 
      p.lastOpenedAt = p.lastOpenedAt || new Date().toISOString();
      save();
      transitionAt.delete(profileId);
      stopFastPolling(profileId); // 状态稳定了，停止高频轮询
    } else if (p.status === "closing" && !cur && allowSwitch) { 
      p.status = "closed"; 
      p.opened = false;
      save();
      transitionAt.delete(profileId);
      stopFastPolling(profileId); // 状态稳定了，停止高频轮询
    }
  }, 10);
  
  fastPollingTimers.set(profileId, { timer, startTime });
}

// 停止单个浏览器的高频轮询
function stopFastPolling(profileId: string) {
  const entry = fastPollingTimers.get(profileId);
  if (entry) {
    window.clearInterval(entry.timer);
    fastPollingTimers.delete(profileId);
  }
}

// 全局低频轮询（1 秒间隔）
async function refreshStatusesOnce() {
  for (const p of state.profiles) {
    let cur = false;
    try { const running = await invoke<number | null>("browser_running", { label: p.id }); if (typeof running === 'number' && running > 0) { (p as any).pid = running; cur = true; } } catch {}
    if (!cur) { try { cur = await isWindowOpen(p.id); } catch {} }
    const started = transitionAt.get(p.id) || 0;
    const allowSwitch = !started || (Date.now() - started >= TRANSIENT_MIN_MS);
    if (p.status === "opening" && cur && allowSwitch) { p.status = "open"; p.opened = true; p.lastOpenedAt = p.lastOpenedAt || new Date().toISOString(); transitionAt.delete(p.id); }
    else if (p.status === "closing" && !cur && allowSwitch) { p.status = "closed"; p.opened = false; transitionAt.delete(p.id); }
    else if (p.status !== "opening" && p.status !== "closing") { p.status = cur ? "open" : "closed"; p.opened = cur; }
  }
  save();
}
onMounted(() => { load(); refreshStatusesOnce(); timer = window.setInterval(async () => { await refreshStatusesOnce(); }, 1000); });
onBeforeUnmount(() => { 
  if (timer) window.clearInterval(timer); 
  // 清除所有高频轮询
  for (const [id, _] of fastPollingTimers) {
    stopFastPolling(id);
  }
});

const filtered = computed(() => { const q = search.value.trim().toLowerCase(); if (!q) return state.profiles; return state.profiles.filter((p) => p.id.toLowerCase().includes(q) || p.name.toLowerCase().includes(q)); });
const allSelected = computed(() => filtered.value.length > 0 && filtered.value.every((p) => selected.has(p.id)));
function toggleRow(id: string) { if (selected.has(id)) selected.delete(id); else selected.add(id); }
function toggleAll() { if (allSelected.value) { for (const p of filtered.value) selected.delete(p.id); } else { for (const p of filtered.value) selected.add(p.id); } }
async function bulkDelete() {
  if (selected.size === 0) return;
  confirmMsg.value = `确认删除选中的 ${selected.size} 个窗口？`;
  confirmAction = async () => {
    const ids = Array.from(selected.values());
    for (const id of ids) {
      const p = state.profiles.find(x => x.id === id);
      if (!p) continue;
      if (p.status === "open" || p.status === "opening") { try { await invoke("browser_close", { label: p.id }); } catch {} }
    }
    state.profiles = state.profiles.filter((p) => !selected.has(p.id));
    selected.clear();
    save();
  };
  confirmVisible.value = true;
}
async function bulkOpen() {
  for (const p of state.profiles) {
    if (!selected.has(p.id)) continue;
    if (p.status === "open" || p.status === "opening") continue;
    try {
      p.status = "opening"; transitionAt.set(p.id, Date.now()); saveLater(0);
      // 启动高频轮询
      startFastPolling(p.id);
      await invoke("log_info", { message: `[BrowserList] bulk open label=${p.id} version=${p.engineVersion || 'N/A'}` });
      const pid = await invoke<number | null>("browser_open", { label: p.id, url: null, version: p.engineVersion || null });
      if (pid && typeof pid === 'number') { (p as any).pid = pid; }
      // 交由轮询切换到 open
    } catch {}
  }
}
async function bulkClose() {
  // 收集所有需要关闭的配置
  const targets = state.profiles.filter(p => 
    selected.has(p.id) && p.status !== "closed" && p.status !== "closing"
  );
  if (targets.length === 0) return;
  
  // 立即设置所有目标为关闭中状态，并启动高频轮询
  for (const p of targets) {
    p.status = "closing";
    transitionAt.set(p.id, Date.now());
    p.opened = false;
    // 为每个目标启动独立的高频轮询
    startFastPolling(p.id);
  }
  saveLater(0);
  
  // 异步执行所有关闭命令，不阻塞UI
  for (const p of targets) {
    invoke("browser_close", { label: p.id }).catch(async (err: any) => {
      // 关闭失败，恢复为开启状态
      p.status = "open";
      p.opened = true;
      save();
      stopFastPolling(p.id);
      transitionAt.delete(p.id);
      try { await invoke("log_info", { message: `[BrowserList] bulk close failed: ${err?.message || String(err)}` }); } catch {}
    });
  }
  // 不立即设置为 closed，让高频轮询检测到关闭后自动更新
}

const isDark = computed(() => resolveEffectiveTheme() === "dark");

async function onCreate(payload: any) {
  const id = `CHE-${Date.now().toString().slice(-6)}`;
  try { await invoke("browser_close", { label: id }); } catch {}
  let engineVersion: string | undefined; try { const v = localStorage.getItem("libre_default_engine"); if (v) engineVersion = v; } catch {}
  state.profiles.push({ id, name: payload.name || id, project: payload.project || "默认项目", opened: false, fingerprint: JSON.stringify(payload.fingerprint), proxy: payload.proxy || "", status: "closed", engineVersion });
  save();
}
</script>

<template>
  <div class="layout-content-container flex flex-col flex-1 min-w-[600px] shrink-0">
    <div class="flex flex-wrap justify-between gap-3 p-4">
      <p class="tracking-light text-[32px] font-bold leading-tight min-w-72" :class="isDark ? 'text-white' : 'text-[#0d141b]'">浏览器列表</p>
      <button :class="['flex min-w-[84px] max-w-[480px] cursor-pointer items-center justify-center overflow-hidden rounded-lg h-8 px-4 text-sm font-medium leading-normal', isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]']" @click="createProfile"><span class="truncate">新建浏览器</span></button>
    </div>
    <div class="px-4 py-3">
      <label class="flex flex-col min-w-40 h-12 w-full">
        <div class="flex w-full flex-1 items-stretch rounded-lg h-full">
          <div :class="['flex border-none items-center justify-center pl-4 rounded-l-lg border-r-0', isDark ? 'text-[#92adc9] bg-[#233648]' : 'text-[#4c739a] bg-[#e7edf3]']">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 256 256"><path d="M229.66,218.34l-50.07-50.06a88.11,88.11,0,1,0-11.31,11.31l50.06,50.07a8,8,0,0,0,11.32-11.32ZM40,112a72,72,0,1,1,72,72A72.08,72.08,0,0,1,40,112Z"/></svg>
          </div>
          <input v-model="search" placeholder="搜索浏览器" :class="['form-input flex w-full min-w-0 flex-1 resize-none overflow-hidden rounded-lg focus:outline-0 focus:ring-0 border-none h-full px-4 rounded-l-none border-l-0 pl-2 text-base font-normal leading-normal', isDark ? 'bg-[#233648] text-white placeholder:text-[#92adc9]' : 'bg-[#e7edf3] text-[#0d141b] placeholder:text-[#4c739a]']" />
        </div>
      </label>
    </div>
    <div class="flex justify-stretch">
      <div class="flex flex-1 gap-3 flex-wrap px-4 py-3 justify-start">
        <button :class="['flex min-w-[84px] max-w-[480px] cursor-pointer items-center justify-center overflow-hidden rounded-lg h-10 px-4 text-sm font-bold leading-normal tracking-[0.015em]', isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]']" @click="bulkOpen"><span class="truncate">批量启动</span></button>
        <button :class="['flex min-w-[84px] max-w-[480px] cursor-pointer items-center justify-center overflow-hidden rounded-lg h-10 px-4 text-sm font-bold leading-normal tracking-[0.015em]', isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]']" @click="bulkClose"><span class="truncate">批量关闭</span></button>
        <button :class="['flex min-w-[84px] max-w-[480px] cursor-pointer items-center justify-center overflow-hidden rounded-lg h-10 px-4 text-sm font-bold leading-normal tracking-[0.015em]', isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]']" @click="bulkDelete"><span class="truncate">批量删除</span></button>
      </div>
    </div>
    <div class="px-4 py-3 @container">
      <AppTable :isEmpty="filtered.length === 0" :cols="6">
        <template #head>
          <tr :class="isDark ? 'bg-[#192633]' : 'bg-slate-50'">
            <th class="px-4 py-3 text-center w-[120px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'"><input type="checkbox" :class="['h-5 w-5 rounded border-2 bg-transparent focus:ring-0 focus:ring-offset-0 focus:outline-none', isDark ? 'border-[#324d67] text-[#1172d4] checked:bg-[#1172d4] checked:border-[#1172d4] focus:border-[#324d67]' : 'border-[#cfdbe7] text-[#2b8dee] checked:bg-[#2b8dee] checked:border-[#2b8dee] focus:border-[#cfdbe7]']" :checked="allSelected" @change="toggleAll" /></th>
            <th class="px-4 py-3 text-left w-[400px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">名称</th>
            <th class="px-4 py-3 text-left w-[200px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">内核版本</th>
            <th class="px-4 py-3 text-left w-[400px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">代理</th>
            <th class="px-4 py-3 text-left w-[160px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">状态</th>
            <th class="px-4 py-3 text-left w-60 text-sm font-medium leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">操作</th>
          </tr>
        </template>
        <template #body>
          <tr v-for="p in filtered" :key="p.id" :class="['border-t', isDark ? 'border-t-[#324d67]' : 'border-t-[#cfdbe7]']">
            <td class="h-[72px] px-4 py-2 w-[120px] text-center text-sm font-normal leading-normal"><input type="checkbox" :class="['h-5 w-5 rounded border-2 bg-transparent focus:ring-0 focus:ring-offset-0 focus:outline-none', isDark ? 'border-[#324d67] text-[#1172d4] checked:bg-[#1172d4] checked:border-[#1172d4] focus:border-[#324d67]' : 'border-[#cfdbe7] text-[#2b8dee] checked:bg-[#2b8dee] checked:border-[#2b8dee] focus:border-[#cfdbe7]']" :checked="selected.has(p.id)" @change="toggleRow(p.id)" /></td>
            <td class="h-[72px] px-4 py-2 w-[400px] text-sm font-normal leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">{{ p.name }}</td>
            <td class="h-[72px] px-4 py-2 w-[200px] text-sm font-normal leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ p.engineVersion || '-' }}</td>
            <td class="h-[72px] px-4 py-2 w-[400px] text-sm font-normal leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ p.proxy || '未配置' }}</td>
            <td class="h-[72px] px-4 py-2 w-[160px] text-sm font-normal leading-normal"><span :class="statusClass(p.status)">{{ statusLabel(p.status) }}</span></td>
            <td class="h-[72px] px-4 py-2 w-60 text-sm font-bold leading-normal tracking-[0.015em] whitespace-nowrap" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'"><button class="mr-3" :class="isDark ? 'text-white' : 'text-[#0d141b]'" @click="toggleOpen(p)">{{ p.opened ? '关闭' : '启动' }}</button><button class="mr-3" :class="isDark ? 'text-white' : 'text-[#0d141b]'" @click="edit(p)">编辑</button><button :class="isDark ? 'text-white' : 'text-[#0d141b]'" @click="removeOne(p)">删除</button></td>
          </tr>
        </template>
      </AppTable>
    </div>
  </div>
  <BrowserCreateModal v-model="showCreate" @submit="onCreate" />
  <Modal v-model="confirmVisible" :title="'确认删除'" :message="confirmMsg" :showCancel="true" confirmText="删除" cancelText="取消" @confirm="onConfirm" />
</template>

<style scoped>
</style>
