<script setup lang="ts">
import { computed } from "vue";
import { resolveEffectiveTheme } from "../state/settings";
const props = defineProps<{ maxHeight?: string; cols?: number; isEmpty?: boolean; emptyText?: string }>();
const isDark = computed(() => resolveEffectiveTheme() === 'dark');
const style = computed(() => props.maxHeight ? { maxHeight: props.maxHeight } : {});
</script>

<template>
  <div :class="['flex rounded-lg border', isDark ? 'border-[#324d67] bg-[#111a22]' : 'border-[#cfdbe7] bg-slate-50', maxHeight ? 'overflow-y-auto' : 'overflow-hidden']" :style="style">
    <table class="flex-1">
      <thead :class="isDark ? 'border-b border-[#324d67]' : 'border-b border-[#cfdbe7]'"><!-- 统一表头下分割线 -->
        <slot name="head" />
      </thead>
      <tbody>
        <template v-if="!isEmpty">
          <slot name="body" />
        </template>
        <tr v-else>
          <td :colspan="cols || 1" class="py-10 text-center text-sm border-t" :class="isDark ? 'text-[#92adc9] border-t-[#324d67]' : 'text-[#4c739a] border-t-[#cfdbe7]'">{{ emptyText || '暂无数据' }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped></style>
