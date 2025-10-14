<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import Tabs from "../components/Tabs.vue";
import Pagination from "../components/Pagination.vue";
import Modal from "../components/Modal.vue";
import AppTable from "../components/AppTable.vue";
import { resolveEffectiveTheme } from "../state/settings";
import { fetch as httpFetch } from "@tauri-apps/plugin-http";
import { invoke } from "@tauri-apps/api/core";
// opener not needed for Rust-handled downloads

type Engine = { name: string; version: string; date?: string; size?: string; installed?: boolean; default?: boolean; downloadUrl?: string; installedAt?: string };

const installed = ref<Engine[]>([]);

const available = ref<Engine[]>([]);
const installedVersions = ref<string[]>([]);
const defaultVersion = ref<string | null>(null);
const installedSet = ref(new Set<string>());
const downloading = ref(new Set<string>());
const extracting = ref(new Set<string>());
const errVisible = ref(false);
const errMsg = ref("");

const q = ref("");
const filtered = computed(() => available.value.filter(e => (e.name + e.version).toLowerCase().includes(q.value.toLowerCase())));
const page = ref(1);
const pageSize = ref(20);
const paged = computed(() => {
  const start = (page.value - 1) * pageSize.value;
  return filtered.value.slice(start, start + pageSize.value);
});
watch([q, () => available.value.length], () => { page.value = 1; });
const isDark = computed(() => resolveEffectiveTheme() === "dark");

const activeTab = ref<"installed" | "available">("installed");
const tabItems = computed(() => [
  { key: "installed", label: "已安装" },
  { key: "available", label: "可用" },
]);

const loading = ref(false);
const loadError = ref<string | null>(null);
let abortCtrl: AbortController | null = null;

async function log(msg: string) {
  try { await invoke("log_info", { message: `[Engines] ${msg}` }); } catch {}
  try { console.debug(`[Engines] ${msg}`); } catch {}
}

function snippet(s: string, n = 256) {
  if (!s) return "";
  return s.length > n ? s.slice(0, n) + `...(${s.length})` : s;
}

function fmtDate(d: string | undefined) {
  if (!d) return "-";
  const dt = new Date(d);
  if (Number.isNaN(dt.getTime())) return d;
  const y = dt.getFullYear();
  const m = String(dt.getMonth() + 1).padStart(2, "0");
  const day = String(dt.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function cmpVerDesc(a: string, b: string) {
  const pa = a.split(/[.-]/).map((x) => parseInt(x, 10));
  const pb = b.split(/[.-]/).map((x) => parseInt(x, 10));
  const len = Math.max(pa.length, pb.length);
  for (let i = 0; i < len; i++) {
    const va = Number.isFinite(pa[i]) ? pa[i] : 0;
    const vb = Number.isFinite(pb[i]) ? pb[i] : 0;
    if (va !== vb) return vb - va;
  }
  return 0;
}

async function loadAvailableFromGoogle() {
  loading.value = true;
  loadError.value = null;
  try {
    abortCtrl?.abort();
    abortCtrl = new AbortController();
    // 主源：chrome-for-testing
    const primary = "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";
    const headers: Record<string, string> = {
      accept: "application/json,text/plain,*/*",
      "user-agent": "libre-browser/0.1 (tauri)"
    };
    await log(`start fetch: ${primary}`);
    let resp = await httpFetch(primary, { method: "GET", signal: abortCtrl.signal, headers });
    // 记录状态与类型
    const status: any = (resp as any).status;
    const url: any = (resp as any).url || primary;
    let contentType = "";
    try { contentType = (resp as any).headers?.get?.("content-type") ?? ""; } catch {}
    await log(`resp status=${status} url=${url} ct=${contentType}`);

    let txt = await resp.text();
    await log(`resp text head=${snippet(txt)}`);

    let data: Array<Record<string, any>>;
    try {
      const parsed = JSON.parse(txt) as any;
      // 从 CFT 构建
      if (Array.isArray(parsed?.versions)) {
        const arr = parsed.versions as Array<any>;
        const seen = new Set<string>();
        const items: Engine[] = [];
        // 只取最新的一段，避免过大遍历；从尾部向前迭代确保优先收集最新版本
        const tail = arr.slice(-1500);
        for (let i = tail.length - 1; i >= 0; i--) {
          const it = tail[i];
          const ver = String(it?.version || "");
          if (!ver || seen.has(ver)) continue;
          seen.add(ver);
          const ts = it?.timestamp || it?.time || undefined;
          let durl: string | undefined;
          try {
            const dl = Array.isArray(it?.downloads?.chrome) ? it.downloads.chrome : [];
            const hitArm = dl.find((x: any) => x?.platform === "mac-arm64");
            const hitX64 = dl.find((x: any) => x?.platform === "mac-x64");
            durl = (hitArm?.url || hitX64?.url);
          } catch {}
          items.push({ name: "Chrome", version: ver, date: fmtDate(ts), size: "-", downloadUrl: durl });
          if (items.length >= 400) break;
        }
        // 理论上已按新→旧收集，仍做一次排序兜底
        items.sort((a, b) => cmpVerDesc(a.version, b.version));
        available.value = items.slice(0, 200);
        await log(`primary CFT ok, arr_len=${arr.length}, tail_len=${tail.length}, items=${available.value.length}, top=${available.value[0]?.version}`);
        await refreshInstalled();
        return;
      }
      // 非 CFT 结构，按原始数组处理
      data = parsed as Array<Record<string, any>>;
    } catch (je: any) {
      await log(`json parse failed(primary): ${je?.message || String(je)}`);
      // 备用1：versionhistory（去掉 sort，手动排序）
      const fallback1 = "https://versionhistory.googleapis.com/v1/chrome/platforms/mac/channels/stable/versions?page_size=200&order_by=version";
      await log(`try fallback1: ${fallback1}`);
      try {
        resp = await httpFetch(fallback1, { method: "GET", signal: abortCtrl.signal, headers });
        const status2: any = (resp as any).status;
        let txt2 = await resp.text();
        await log(`fallback1 status=${status2} text head=${snippet(txt2)}`);
        let parsed: any = JSON.parse(txt2);
        const versions = Array.isArray(parsed?.versions) ? parsed.versions : [];
        let items: Engine[] = [];
        const seen = new Set<string>();
        for (const v of versions) {
          const ver = String(v?.version || "");
          if (!ver || seen.has(ver)) continue;
          seen.add(ver);
          items.push({ name: "Chrome", version: ver, date: fmtDate(v?.publishedAt), size: "-" });
          if (items.length >= 400) break;
        }
        // 手动按版本降序
        items.sort((a, b) => cmpVerDesc(a.version, b.version));
        available.value = items.slice(0, 50);
        await log(`fallback1 ok, items=${available.value.length}`);
        if (available.value.length > 0) { await refreshInstalled(); return; }
      } catch (e2: any) {
        await log(`fallback1 failed: ${e2?.message || String(e2)}`);
      }
      // 备用2：chromiumdash

      // 备用3：chromiumdash
      const fallback2 = "https://chromiumdash.appspot.com/fetch_releases?channel=Stable&platform=Mac&num=100";
      await log(`try fallback2: ${fallback2}`);
      resp = await httpFetch(fallback2, { method: "GET", signal: abortCtrl.signal, headers });
      const status3: any = (resp as any).status;
      let txt3 = await resp.text();
      await log(`fallback2 status=${status3} text head=${snippet(txt3)}`);
      let parsed2: any = JSON.parse(txt3);
      const list = Array.isArray(parsed2) ? parsed2 : Array.isArray(parsed2?.releases) ? parsed2.releases : [];
      const items2: Engine[] = [];
      const seen2 = new Set<string>();
      for (const r of list) {
        const ver = String(r?.version || r?.milestone || "");
        if (!ver || seen2.has(ver)) continue;
        seen2.add(ver);
        const ts = r?.time || r?.published_at || r?.publishedAt || r?.date || undefined;
        items2.push({ name: "Chrome", version: ver, date: fmtDate(ts), size: "-" });
        if (items2.length >= 200) break;
      }
      items2.sort((a, b) => cmpVerDesc(a.version, b.version));
      available.value = items2.slice(0, 50);
      await log(`fallback2 ok, items=${available.value.length}`);
      await refreshInstalled();
      return;
    }

    const seen = new Set<string>();
    const items: Engine[] = [];
    for (const it of data) {
      const ver = String(it.version || "");
      if (!ver || seen.has(ver)) continue;
      seen.add(ver);
      items.push({ name: "Chrome", version: ver, date: fmtDate(it.time), size: "-" });
      if (items.length >= 50) break;
    }
    available.value = items;
    await log(`fetch ok, items=${items.length}`);
    await refreshInstalled();
  } catch (e: any) {
    const name = e?.name || typeof e;
    const msg = e?.message || String(e);
    const stack = e?.stack || "";
    loadError.value = msg;
    await log(`fetch error: name=${name} msg=${msg}`);
    if (stack) await log(`stack: ${stack.split("\n")[0]}`);
  } finally {
    loading.value = false;
  }
}
onMounted(async () => {
  if (activeTab.value === "available") await loadAvailableFromGoogle();
  try { const raw = localStorage.getItem("libre_default_engine"); if (raw) defaultVersion.value = raw; } catch {}
  await refreshInstalled();
});

watch(activeTab, async (t) => {
  if (t === "available" && available.value.length === 0 && !loading.value) {
    await loadAvailableFromGoogle();
  } else if (t !== "available") {
    abortCtrl?.abort();
  }
});
function setDownloading(ver: string, val: boolean) {
  const s = new Set(downloading.value);
  if (val) s.add(ver); else s.delete(ver);
  downloading.value = s;
}
function setExtracting(ver: string, val: boolean) {
  const s = new Set(extracting.value);
  if (val) s.add(ver); else s.delete(ver);
  extracting.value = s;
}
async function refreshInstalled() {
  try {
    const list = await invoke<any[]>("list_installed_engines");
    installedVersions.value = list.map(item => item.version);
    installedSet.value = new Set(installedVersions.value);
    installed.value = list.map(item => ({
      name: "Chrome",
      version: item.version,
      date: "-",
      size: "-",
      installed: true,
      default: item.version === defaultVersion.value,
      installedAt: item.installedAt
    }));
  } catch {}
}
async function downloadEngine(e: Engine) {
  if (!e.downloadUrl) { await log(`no download url for ${e.version}`); return; }
  try {
    setDownloading(e.version, true);
    await invoke<string>("download_engine_archive", { version: e.version, url: e.downloadUrl });
    setDownloading(e.version, false);
    setExtracting(e.version, true);
    await invoke<string>("extract_engine_archive", { version: e.version });
    setExtracting(e.version, false);
    await refreshInstalled();
  } catch (err: any) {
    const m = err?.message || String(err);
    await log(`download/extract failed: ${m}`);
    errMsg.value = m;
    errVisible.value = true;
  } finally {
    setDownloading(e.version, false);
    setExtracting(e.version, false);
  }
}
function toggleDefault(e: Engine, checked: boolean) {
  if (!checked) return;
  defaultVersion.value = e.version;
  try { localStorage.setItem("libre_default_engine", e.version); } catch {}
  installed.value = installed.value.map(x => ({ ...x, default: x.version === e.version }));
}
</script>
<template>
  <div class="layout-content-container flex flex-col flex-1 min-w-[600px] shrink-0 p-0">
    <div class="flex flex-wrap justify-between gap-3 p-4">
      <p class="tracking-light text-[32px] font-bold leading-tight min-w-72" :class="isDark ? 'text-white' : 'text-[#0d141b]'">内核管理</p>
    </div>

    <div class="pb-3">
      <Tabs :items="tabItems" v-model="activeTab" />
    </div>

    <template v-if="activeTab === 'installed'">
      <h2 class="text-[22px] font-bold leading-tight tracking-[-0.015em] px-4 pb-3 pt-5" :class="isDark ? 'text-white' : 'text-[#0d141b]'">已安装的内核</h2>
      <div class="px-4 py-3 @container">
        <AppTable :isEmpty="installed.length === 0" :cols="6">
          <template #head>
            <tr :class="isDark ? 'bg-[#192633]' : 'bg-slate-50'">
              <th class="px-4 py-3 text-left w-[250px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">版本</th>
              <th class="px-4 py-3 text-left w-[200px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">安装时间</th>
              <th class="px-4 py-3 text-left w-[200px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">发布日期</th>
              <th class="px-4 py-3 text-left w-[150px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">大小</th>
              <th class="px-4 py-3 text-center w-[120px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">默认</th>
              <th class="px-4 py-3 text-left w-60 text-sm font-medium leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">操作</th>
            </tr>
          </template>
          <template #body>
            <tr v-for="e in installed" :key="e.name+e.version" :class="['border-t', isDark ? 'border-t-[#324d67]' : 'border-t-[#cfdbe7]']">
              <td class="h-[72px] px-4 py-2 w-[250px] text-sm font-normal leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">{{ e.name }} {{ e.version }}</td>
              <td class="h-[72px] px-4 py-2 w-[200px] text-sm font-normal leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ e.installedAt || '未知' }}</td>
              <td class="h-[72px] px-4 py-2 w-[200px] text-sm font-normal leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ e.date }}</td>
              <td class="h-[72px] px-4 py-2 w-[150px] text-sm font-normal leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ e.size }}</td>
              <td class="h-[72px] px-4 py-2 w-[120px] text-center text-sm font-normal leading-normal">
                <input type="checkbox" :checked="e.default" @change="(ev:any)=> toggleDefault(e, ev.target?.checked)" :class="['h-5 w-5 rounded border-2 bg-transparent checked:bg-[image:--checkbox-tick-svg] focus:ring-0 focus:ring-offset-0 focus:outline-none mx-auto block', isDark ? 'border-[#324d67] text-[#1172d4] checked:bg-[#1172d4] checked:border-[#1172d4] focus:border-[#324d67]' : 'border-[#cfdbe7] text-[#2b8dee] checked:bg-[#2b8dee] checked:border-[#2b8dee] focus:border-[#cfdbe7]']" />
              </td>
              <td class="h-[72px] px-4 py-2 w-60 text-sm font-bold leading-normal tracking-[0.015em]" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">编辑 | 删除</td>
            </tr>
          </template>
        </AppTable>
      </div>
    </template>

    <template v-if="activeTab === 'available'">
      <h2 class="text-[22px] font-bold leading-tight tracking-[-0.015em] px-4 pb-3 pt-5" :class="isDark ? 'text-white' : 'text-[#0d141b]'">可用的内核</h2>
      <div class="px-4 py-3">
        <label class="flex flex-col min-w-40 h-12 w-full">
          <div class="flex w-full flex-1 items-stretch rounded-lg h-full">
            <div :class="['flex border-none items-center justify-center pl-4 rounded-l-lg border-r-0', isDark ? 'text-[#92adc9] bg-[#233648]' : 'text-[#4c739a] bg-[#e7edf3]']">
              <svg xmlns="http://www.w3.org/2000/svg" width="24px" height="24px" fill="currentColor" viewBox="0 0 256 256"><path d="M229.66,218.34l-50.07-50.06a88.11,88.11,0,1,0-11.31,11.31l50.06,50.07a8,8,0,0,0,11.32-11.32ZM40,112a72,72,0,1,1,72,72A72.08,72.08,0,0,1,40,112Z"></path></svg>
            </div>
            <input v-model="q" placeholder="搜索内核..." :class="['form-input flex w-full min-w-0 flex-1 resize-none overflow-hidden rounded-lg focus:outline-0 focus:ring-0 border-none focus:border-none h-full px-4 rounded-l-none border-l-0 pl-2 text-base font-normal leading-normal', isDark ? 'bg-[#233648] text-white placeholder:text-[#92adc9]' : 'bg-[#e7edf3] text-[#0d141b] placeholder:text-[#4c739a]']" />
          </div>
        </label>
      </div>

      <div class="px-4 py-3 @container">
        <AppTable :maxHeight="'480px'" :isEmpty="!loading && !loadError && paged.length === 0" :cols="5">
          <template #head>
            <tr :class="[isDark ? 'bg-[#192633]' : 'bg-slate-50', 'sticky top-0 z-10']">
              <th class="px-4 py-3 text-left w-[400px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">名称/版本</th>
              <th class="px-4 py-3 text-left w-[400px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">发布日期</th>
              <th class="px-4 py-3 text-left w-[400px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">大小</th>
              <th class="px-4 py-3 text-left w-60 text-sm font-medium leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">操作</th>
              <th class="px-4 py-3 text-left w-[400px] text-sm font-medium leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">状态</th>
            </tr>
          </template>
          <template #body>
            <tr v-if="loading">
              <td class="px-4 py-4 text-sm" :class="isDark ? 'text-white' : 'text-[#0d141b]'" colspan="5">加载中...</td>
            </tr>
            <tr v-else-if="loadError">
              <td class="px-4 py-4 text-sm text-red-400" colspan="5">加载失败：{{ loadError }}</td>
            </tr>
            <tr v-else v-for="e in paged" :key="e.name+e.version" :class="['border-t', isDark ? 'border-t-[#324d67]' : 'border-t-[#cfdbe7]']">
              <td class="h-[72px] px-4 py-2 w-[400px] text-sm font-normal leading-normal" :class="isDark ? 'text-white' : 'text-[#0d141b]'">{{ e.name }} {{ e.version }}</td>
              <td class="h-[72px] px-4 py-2 w-[400px] text-sm font-normal leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ e.date || '-' }}</td>
              <td class="h-[72px] px-4 py-2 w-[400px] text-sm font-normal leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ e.size || '-' }}</td>
              <td class="h-[72px] px-4 py-2 w-60 text-sm font-bold leading-normal tracking-[0.015em]" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">
                <button :disabled="!e.downloadUrl || downloading.has(e.version) || extracting.has(e.version)" @click="downloadEngine(e)" :class="['px-3 h-8 rounded', (!e.downloadUrl || downloading.has(e.version) || extracting.has(e.version)) ? (isDark ? 'bg-[#1a2835] text-[#4c739a]' : 'bg-slate-100 text-slate-400') : (isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]') ]">下载</button>
              </td>
              <td class="h-[72px] px-4 py-2 w-[400px] text-sm font-normal leading-normal" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">
                {{ downloading.has(e.version) ? '下载中' : (extracting.has(e.version) ? '解压中' : (installedSet.has(e.version) ? '已安装' : '未安装')) }}
              </td>
            </tr>
          </template>
        </AppTable>
        <Pagination class="px-1" :page="page" :pageSize="pageSize" :total="filtered.length" @update:page="(v:number)=> page=v" @update:pageSize="(v:number)=> pageSize=v" />
        <Modal v-model="errVisible" title="操作失败" :message="errMsg" />
      </div>
    </template>
  </div>
</template>
<style scoped></style>
