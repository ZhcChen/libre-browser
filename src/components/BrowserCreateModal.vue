<script setup lang="ts">
import { computed, reactive } from "vue";
import { resolveEffectiveTheme } from "../state/settings";
defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void; (e: 'submit', payload: any): void }>();
const isDark = computed(() => resolveEffectiveTheme() === 'dark');
const form = reactive({
  name: '', project: '默认项目',
  fingerprint: {
    userAgent: navigator.userAgent,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC',
    languages: navigator.languages?.join(',') || 'zh-CN,zh',
    platform: navigator.platform || 'MacIntel',
    screen: `${screen.width}x${screen.height}`,
    dnt: '0',
    webglVendor: '', webglRenderer: '',
    canvasNoise: true,
    webrtcPolicy: 'default',
  },
  proxy: ''
});
function close(){ emit('update:modelValue', false); }
function submit(){ if(!form.name.trim()) return; emit('submit', JSON.parse(JSON.stringify(form))); close(); }
</script>

<template>
  <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center">
    <div class="absolute inset-0 bg-black/40" @click="close" />
    <div class="relative w-[720px] max-h-[80vh] overflow-y-auto rounded-lg p-4" :class="isDark ? 'bg-[#111a22] text-white' : 'bg-white text-[#0d141b]'">
      <div class="text-lg font-bold mb-3">新建浏览器</div>
      <div class="grid grid-cols-2 gap-3">
        <label class="text-sm">名称<input v-model="form.name" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">项目<input v-model="form.project" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm col-span-2">代理<input v-model="form.proxy" placeholder="http://user:pass@host:port 或 socks5://..." class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <div class="col-span-2 font-bold mt-2">指纹设置</div>
        <label class="text-sm">User-Agent<input v-model="form.fingerprint.userAgent" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">时区<input v-model="form.fingerprint.timezone" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">语言列表<input v-model="form.fingerprint.languages" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">平台<input v-model="form.fingerprint.platform" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">屏幕分辨率<input v-model="form.fingerprint.screen" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">Do-Not-Track<input v-model="form.fingerprint.dnt" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">WebGL Vendor<input v-model="form.fingerprint.webglVendor" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">WebGL Renderer<input v-model="form.fingerprint.webglRenderer" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
        <label class="text-sm">Canvas 噪声
          <select v-model="form.fingerprint.canvasNoise" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'"><option :value="true">开启</option><option :value="false">关闭</option></select>
        </label>
        <label class="text-sm">WebRTC 策略
          <select v-model="form.fingerprint.webrtcPolicy" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'"><option value="default">默认</option><option value="proxy_only">仅代理</option><option value="disable_local">禁用内网</option></select>
        </label>
      </div>
      <div class="mt-4 flex justify-end gap-2">
        <button class="h-9 px-4 rounded" :class="isDark ? 'bg-[#1a2835] text-[#4c739a]' : 'bg-slate-100 text-slate-400'" @click="close">取消</button>
        <button class="h-9 px-4 rounded" :class="isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]'" @click="submit">创建</button>
      </div>
    </div>
  </div>
</template>

<style scoped></style>
