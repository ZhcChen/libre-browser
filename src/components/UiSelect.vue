<script setup lang="ts">
import { computed } from "vue";

type Option = { label: string; value: string };

const props = defineProps<{
  modelValue: string;
  options: Option[];
  placeholder?: string;
  theme?: "dark" | "light";
  size?: "md" | "lg";
}>();

const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();

const inputClass = computed(() => {
  const t = props.theme ?? "dark";
  const h = props.size === "lg" ? "h-14 p-[15px]" : "h-10 px-3";
  if (t === "light")
    return [
      "form-input w-full min-w-0 rounded-lg border-none focus:outline-0 focus:ring-0 bg-[#e7edf3] text-[#0d141b]",
      "bg-[image:var(--select-button-svg)] bg-no-repeat [background-position:right_12px_center]",
      h,
    ].join(" ");
  return [
    "form-input w-full min-w-0 rounded-lg border-none focus:outline-0 focus:ring-0 bg-[#233648] text-white",
    "bg-[image:var(--select-button-svg)] bg-no-repeat [background-position:right_12px_center] placeholder:text-[#92adc9]",
    h,
  ].join(" ");
});

function onChange(e: Event) {
  emit("update:modelValue", (e.target as HTMLSelectElement).value);
}
</script>

<template>
  <select :class="inputClass" :value="modelValue" @change="onChange">
    <option v-if="placeholder" value="">{{ placeholder }}</option>
    <option v-for="opt in options" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
  </select>
</template>

<style scoped>
</style>
