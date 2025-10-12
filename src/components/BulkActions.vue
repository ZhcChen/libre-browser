<script setup lang="ts">
const props = defineProps<{
  allSelected: boolean;
  selectedCount: number;
  total: number;
}>();

const emit = defineEmits<{
  (e: "toggle-all"): void;
  (e: "bulk-open"): void;
  (e: "bulk-close"): void;
  (e: "bulk-delete"): void;
}>();
</script>

<template>
  <div class="bulk">
    <label class="select-all">
      <input type="checkbox" :checked="props.allSelected" @change="emit('toggle-all')" />
      <span>全选 ({{ props.selectedCount }}/{{ props.total }})</span>
    </label>
    <div class="ops">
      <button class="ghost" @click="emit('bulk-open')">打开</button>
      <button class="ghost" @click="emit('bulk-close')">关闭</button>
      <button class="danger" @click="emit('bulk-delete')">删除</button>
    </div>
  </div>
</template>

<style scoped>
.bulk{display:flex;align-items:center;gap:12px}
.select-all{display:flex;align-items:center;gap:6px;color:#6b7280}
.ops{display:flex;align-items:center;gap:8px}
.ghost{background:#fff;border:1px solid #e5e7eb;padding:6px 10px;border-radius:6px;cursor:pointer}
.danger{background:#fee2e2;color:#991b1b;border:1px solid #fecaca;padding:6px 10px;border-radius:6px;cursor:pointer}
</style>
