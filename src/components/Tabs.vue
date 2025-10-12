<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount, onUpdated, nextTick } from "vue";
import { resolveEffectiveTheme } from "../state/settings";
type TabItem = { key: string; label: string };

defineProps<{
  items: TabItem[];
  modelValue: string;
}>();
const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();

function select(k: string) {
  emit("update:modelValue", k);
}

const isDark = computed(() => resolveEffectiveTheme() === "dark");

const root = ref<HTMLElement | null>(null);
const indicatorLeft = ref(0);
const indicatorWidth = ref(0);

function updateIndicator() {
  const activeBtn = root.value?.querySelector('[data-active="true"]') as HTMLElement | null;
  if (!root.value || !activeBtn) return;
  const label = activeBtn.querySelector('p') as HTMLElement | null;
  const target = label || activeBtn;
  const rootBox = root.value.getBoundingClientRect();
  const box = target.getBoundingClientRect();
  indicatorLeft.value = Math.round(box.left - rootBox.left);
  indicatorWidth.value = Math.round(box.width);
}

function onResize() { updateIndicator(); }

onMounted(() => { nextTick(() => { updateIndicator(); setTimeout(updateIndicator, 0); }); window.addEventListener("resize", onResize); });
onBeforeUnmount(() => { window.removeEventListener("resize", onResize); });
onUpdated(() => { nextTick(updateIndicator); });
</script>

<template>
  <div ref="root" class="relative flex border-b px-4 gap-8" :class="isDark ? 'border-[#324d67]' : 'border-[#cfdbe7]'">
    <button
      v-for="it in items"
      :key="it.key"
      class="flex flex-col items-center justify-center border-b-[3px] border-b-transparent pb-[13px] pt-4"
      :class="[
        it.key === modelValue
          ? (isDark ? 'text-white' : 'text-[#0d141b]')
          : (isDark ? 'text-[#92adc9]' : 'text-[#4c739a]')
      ]"
      :data-active="it.key === modelValue"
      @click="select(it.key)"
    >
      <p class="text-sm font-bold leading-normal tracking-[0.015em]">{{ it.label }}</p>
    </button>
    <div
      class="absolute bottom-0 left-0 h-[3px] pointer-events-none"
      :class="isDark ? 'bg-[#2b8dee]' : 'bg-[#2b8dee]'"
      :style="{ transform: `translateX(${indicatorLeft}px)`, width: indicatorWidth + 'px', transition: 'transform 200ms ease, width 200ms ease' }"
    />
  </div>
  
</template>

<style scoped>
</style>
