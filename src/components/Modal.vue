<script setup lang="ts">
import { computed } from "vue";
import { resolveEffectiveTheme } from "../state/settings";
const props = defineProps<{ modelValue: boolean; title?: string; message?: string; confirmText?: string; cancelText?: string; showCancel?: boolean }>();
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void; (e: 'confirm'): void }>();
const isDark = computed(() => resolveEffectiveTheme() === 'dark');
function close() { emit('update:modelValue', false); }
function onConfirm() { emit('confirm'); emit('update:modelValue', false); }
</script>

<template>
  <Transition name="fade">
    <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/40" @click="close" />
      <Transition name="zoom">
        <div class="relative w-[420px] rounded-lg p-4" :class="isDark ? 'bg-[#111a22] text-white' : 'bg-white text-[#0d141b]'">
          <div class="text-lg font-bold mb-2">{{ title || '提示' }}</div>
          <div class="text-sm mb-4" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">{{ message }}</div>
          <div class="flex justify-end gap-2">
            <button v-if="props.showCancel !== false" class="h-9 px-4 rounded" :class="isDark ? 'bg-[#1a2835] text-[#4c739a]' : 'bg-slate-100 text-slate-600'" @click="close">{{ props.cancelText || '取消' }}</button>
            <button class="h-9 px-4 rounded" :class="isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]'" @click="onConfirm">{{ props.confirmText || '确定' }}</button>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
.fade-enter-active,.fade-leave-active { transition: opacity .15s ease; }
.fade-enter-from,.fade-leave-to { opacity: 0; }
.zoom-enter-active,.zoom-leave-active { transition: transform .18s ease, opacity .18s ease; }
.zoom-enter-from,.zoom-leave-to { transform: scale(0.96); opacity: 0; }
</style>
