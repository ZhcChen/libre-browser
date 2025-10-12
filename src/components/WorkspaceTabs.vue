<script setup lang="ts">
import { reactive } from "vue";

type Tab = { id: string; name: string; closable?: boolean };

const tabs = reactive<Tab[]>([
  { id: "default", name: "默认工作区", closable: false },
]);

const activeId = reactive({ value: tabs[0].id });

function addTab() {
  const name = prompt("新建工作区名称", "未命名工作区");
  if (!name) return;
  const id = `${Date.now()}`;
  tabs.push({ id, name, closable: true });
  activeId.value = id;
}

function closeTab(t: Tab) {
  if (!t.closable) return;
  const idx = tabs.findIndex((x) => x.id === t.id);
  if (idx >= 0) tabs.splice(idx, 1);
  if (!tabs.find((x) => x.id === activeId.value)) {
    activeId.value = tabs[0]?.id || "default";
  }
}

defineExpose({ tabs, activeId });
</script>

<template>
  <div class="tabs">
    <div
      v-for="t in tabs"
      :key="t.id"
      class="tab"
      :class="{ active: t.id === activeId.value }"
      @click="activeId.value = t.id"
    >
      <span>{{ t.name }}</span>
      <button v-if="t.closable" class="x" @click.stop="closeTab(t)">×</button>
    </div>
    <button class="add" @click="addTab">＋</button>
  </div>
</template>

<style scoped>
.tabs{display:flex;align-items:center;gap:8px}
.tab{display:flex;align-items:center;gap:6px;padding:6px 10px;border:1px solid #e5e7eb;border-radius:8px;background:#fff;color:#374151;cursor:pointer}
.tab.active{border-color:#c7d2fe;background:#eef2ff;color:#1d4ed8}
.x{border:none;background:transparent;color:#6b7280;cursor:pointer}
.add{border:1px dashed #d1d5db;background:#fff;border-radius:8px;padding:6px 10px;color:#6b7280;cursor:pointer}
</style>
