<script setup lang="ts">
import { computed } from "vue";
import { resolveEffectiveTheme } from "../state/settings";

const props = defineProps<{ page: number; pageSize: number; total: number; pageSizes?: number[] }>();
const emit = defineEmits<{
  (e: "update:page", value: number): void
  (e: "update:pageSize", value: number): void
}>();
const isDark = computed(() => resolveEffectiveTheme() === "dark");
const sizes = computed(() => props.pageSizes ?? [10, 20, 50, 100]);
const totalPages = computed(() => Math.max(1, Math.ceil((props.total || 0) / (props.pageSize || 1))));

function clampPage(p: number) { return Math.min(totalPages.value, Math.max(1, p)); }
function prev() { emit("update:page", clampPage(props.page - 1)); }
function next() { emit("update:page", clampPage(props.page + 1)); }
function onSizeChange(e: Event) {
  const v = parseInt((e.target as HTMLSelectElement).value, 10) || props.pageSize;
  emit("update:pageSize", v);
  emit("update:page", 1);
}
</script>

<template>
  <div class="flex items-center justify-between mt-3">
    <div class="flex items-center gap-2">
      <button class="h-8 px-3 rounded border" :class="isDark ? 'text-white border-[#324d67] bg-[#233648]' : 'text-[#0d141b] border-[#cfdbe7] bg-slate-50'" @click="prev">上一页</button>
      <button class="h-8 px-3 rounded border" :class="isDark ? 'text-white border-[#324d67] bg-[#233648]' : 'text-[#0d141b] border-[#cfdbe7] bg-slate-50'" @click="next">下一页</button>
      <span class="text-sm" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">第 {{ page }} / {{ totalPages }} 页</span>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-sm" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">每页</span>
      <select class="h-8 px-2 rounded border" :class="isDark ? 'text-white border-[#324d67] bg-[#233648]' : 'text-[#0d141b] border-[#cfdbe7] bg-white'" :value="pageSize" @change="onSizeChange">
        <option v-for="s in sizes" :key="s" :value="s">{{ s }}</option>
      </select>
      <span class="text-sm" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">条，共 {{ total }} 条</span>
    </div>
  </div>
</template>

<style scoped></style>
