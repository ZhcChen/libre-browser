<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Tabs from "../components/Tabs.vue";
import UiSelect from "../components/UiSelect.vue";
import { language, theme, maxBrowsers, resolveEffectiveTheme } from "../state/settings";
import { t } from "../i18n";

const isDark = computed(() => resolveEffectiveTheme() === "dark");
const activeTab = ref<"language" | "theme" | "quota" | "cleanup">("language");
const cleanupLoading = ref(false);
const cleanupResult = ref("");
const tabItems = computed(() => [
  { key: "language", label: t("settings.tabs.language") },
  { key: "theme", label: t("settings.tabs.theme") },
  { key: "quota", label: t("settings.tabs.quota") },
  { key: "cleanup", label: t("settings.tabs.cleanup") },
]);

function loadActiveBrowserLabels(): string[] {
  try {
    const raw = localStorage.getItem("libre_browser_profiles");
    if (!raw) return [];
    const arr = JSON.parse(raw) as Array<{ id?: string }>;
    return arr
      .map((item) => (typeof item.id === "string" ? item.id : ""))
      .filter((id) => id.trim().length > 0);
  } catch {
    return [];
  }
}

async function cleanupStaleCache() {
  if (cleanupLoading.value) return;
  cleanupLoading.value = true;
  cleanupResult.value = "";
  try {
    const labels = loadActiveBrowserLabels();
    const res = await invoke<any>("cleanup_stale_browser_cache", {
      activeLabels: labels,
      active_labels: labels,
    });
    const removedProfiles = Number(res?.removed_profiles ?? 0);
    const removedCustomApps = Number(res?.removed_custom_apps ?? 0);
    const removedCustomIcons = Number(res?.removed_custom_icons ?? 0);
    const warnings = Array.isArray(res?.warnings) ? res.warnings.length : 0;
    cleanupResult.value = t("settings.cleanup.done")
      .replace("{profiles}", String(removedProfiles))
      .replace("{apps}", String(removedCustomApps))
      .replace("{icons}", String(removedCustomIcons))
      .replace("{warnings}", String(warnings));
  } catch (e: any) {
    cleanupResult.value = `${t("settings.cleanup.failed")}: ${e?.message || String(e)}`;
  } finally {
    cleanupLoading.value = false;
  }
}
</script>

<template>
  <div class="layout-content-container flex flex-col flex-1 min-w-[600px] shrink-0 p-0 settings-root" :class="isDark ? 'text-white' : 'text-[#0d141b]'">
    <div class="flex flex-wrap justify-between gap-3 p-4">
      <p class="tracking-light text-[32px] font-bold leading-tight min-w-72" :class="isDark ? 'text-white' : 'text-[#0d141b]'">{{ t('settings.title') }}</p>
    </div>

    <div class="pb-3">
      <Tabs :items="tabItems" v-model="activeTab" />
    </div>

    <div v-if="activeTab === 'language'">
      <h3 class="text-lg font-bold leading-tight tracking-[-0.015em] px-4 pb-2 pt-4" :class="isDark ? 'text-white' : 'text-[#0d141b]'">{{ t('settings.language.label') }}</h3>
      <div class="flex max-w-[480px] flex-wrap items-end gap-4 px-4 py-3">
        <label class="flex flex-col min-w-40 flex-1">
          <UiSelect
            v-model="language"
            :options="[
              { label: '简体中文', value: 'zh-CN' },
              { label: 'English', value: 'en-US' },
            ]"
            :theme="isDark ? 'dark' : 'light'"
            size="lg"
          />
        </label>
      </div>
    </div>

    <div v-if="activeTab === 'theme'">
      <h3 class="text-lg font-bold leading-tight tracking-[-0.015em] px-4 pb-2 pt-4" :class="isDark ? 'text-white' : 'text-[#0d141b]'">{{ t('settings.theme.label') }}</h3>
      <div class="flex max-w-[480px] flex-wrap items-end gap-4 px-4 py-3">
        <label class="flex flex-col min-w-40 flex-1">
          <UiSelect
            v-model="theme"
            :options="[
              { label: t('settings.theme.system'), value: 'system' },
              { label: t('settings.theme.light'), value: 'light' },
              { label: t('settings.theme.dark'), value: 'dark' },
            ]"
            :theme="isDark ? 'dark' : 'light'"
            size="lg"
          />
        </label>
      </div>
    </div>

    <div v-if="activeTab === 'quota'" class="px-4 py-3">
      <h3 class="text-lg font-bold leading-tight tracking-[-0.015em] pb-2 pt-1" :class="isDark ? 'text-white' : 'text-[#0d141b]'">{{ t('settings.quota.label') }}</h3>
      <div class="flex max-w-[480px] items-center gap-4">
        <label class="flex flex-col min-w-40 flex-1">
          <span class="text-sm text-[#92adc9] mb-1">{{ t('settings.quota.maxBrowsers') }}</span>
          <input
            v-model.number="maxBrowsers"
            type="number"
            min="1"
            :class="[
              'form-input w-full h-12 rounded-lg border-none px-4',
              isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]'
            ]"
            :placeholder="t('settings.quota.placeholder')"
          />
        </label>
      </div>
    </div>

    <div v-if="activeTab === 'cleanup'" class="px-4 py-3">
      <h3 class="text-lg font-bold leading-tight tracking-[-0.015em] pb-2 pt-1" :class="isDark ? 'text-white' : 'text-[#0d141b]'">{{ t('settings.cleanup.label') }}</h3>
      <p class="text-sm mb-3" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ t('settings.cleanup.desc') }}</p>
      <button
        class="h-10 px-4 rounded-lg text-sm font-medium"
        :class="[
          isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]',
          cleanupLoading ? 'opacity-60 cursor-not-allowed' : ''
        ]"
        :disabled="cleanupLoading"
        @click="cleanupStaleCache"
      >
        {{ cleanupLoading ? t('settings.cleanup.running') : t('settings.cleanup.action') }}
      </button>
      <p v-if="cleanupResult" class="text-sm mt-3" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">
        {{ cleanupResult }}
      </p>
    </div>
  </div>
  
</template>

<style scoped>
.settings-root { 
  font-family: Inter, "Noto Sans", sans-serif;
}
</style>
