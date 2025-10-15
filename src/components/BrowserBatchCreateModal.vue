<script setup lang="ts">
import { computed, reactive } from "vue";
import { resolveEffectiveTheme } from "../state/settings";
defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void; (e: 'submit', payload: any[]): void }>();
const isDark = computed(() => resolveEffectiveTheme() === 'dark');

const form = reactive({
  namePrefix: '', 
  count: 5,
  project: '默认项目',
  fingerprint: {
    userAgent: navigator.userAgent,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC',
    languages: navigator.languages?.join(',') || 'zh-CN,zh',
    platform: navigator.platform || 'MacIntel',
    screen: `${screen.width}x${screen.height}`,
    screenColorDepth: screen.colorDepth,
    screenPixelRatio: window.devicePixelRatio,
    dnt: '0',
    webglVendor: '',
    webglRenderer: '',
    webglVersion: '',
    canvasNoise: true,
    canvasFingerprint: '',
    webrtcPolicy: 'default',
    audioContext: 'default',
    fontFingerprint: 'default',
    timezoneOffset: new Date().getTimezoneOffset(),
    hardwareConcurrency: navigator.hardwareConcurrency,
    deviceMemory: (navigator as any).deviceMemory || '4',
    connection: {
      effectiveType: (navigator as any).connection?.effectiveType || '4g',
      downlink: (navigator as any).connection?.downlink || '10',
      rtt: (navigator as any).connection?.rtt || '100'
    },
    batteryLevel: '',
    plugins: [],
    mimeTypes: [],
    cookiesEnabled: navigator.cookieEnabled,
    doNotTrack: navigator.doNotTrack
  },
  proxy: ''
});

function close(){ emit('update:modelValue', false); }

function submit(){
  // 验证输入
  if (!form.namePrefix.trim()) {
    alert('请输入名称前缀');
    return;
  }
  if (form.count <= 0 || form.count > 100) {
    alert('创建数量必须在1-100之间');
    return;
  }

  // 生成批量创建的数据
  const batchPayload: any[] = [];
  for (let i = 1; i <= form.count; i++) {
    const singleForm = JSON.parse(JSON.stringify(form));
    // 生成名称：前缀-序号
    singleForm.name = `${form.namePrefix}-${i}`;
    // 移除批量创建特有的字段
    delete singleForm.namePrefix;
    delete singleForm.count;
    batchPayload.push(singleForm);
  }
  
  emit('submit', batchPayload);
  close();
}
</script>

<template>
  <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center">
    <div class="absolute inset-0 bg-black/40" @click="close" />
    <div class="relative w-[720px] max-h-[80vh] flex flex-col rounded-lg p-4" :class="isDark ? 'bg-[#111a22] text-white' : 'bg-white text-[#0d141b]'">
      <div class="text-lg font-bold mb-3">批量新建浏览器</div>
      <div class="max-w-4xl mx-auto overflow-y-auto flex-1">
        <!-- 基本信息 -->
        <div class="space-y-4 mb-6">
          <h3 class="font-bold text-lg">基本信息</h3>
          <div class="grid grid-cols-2 gap-3">
            <label class="text-sm">名称前缀<input v-model="form.namePrefix" placeholder="例如：测试" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
            <label class="text-sm">创建数量<input v-model.number="form.count" type="number" min="1" max="100" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
            <label class="text-sm">项目<input v-model="form.project" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
            <label class="text-sm">代理<input v-model="form.proxy" placeholder="例如：http://proxy.example:8080 或 socks5://proxy.example:1080" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" /></label>
          </div>
          <div class="text-xs" :class="isDark ? 'text-[#92adc9]' : 'text-[#4c739a]'">
            将创建 {{ form.count }} 个浏览器，名称为：{{ form.namePrefix || '前缀' }}-1, {{ form.namePrefix || '前缀' }}-2 ... {{ form.namePrefix || '前缀' }}-{{ form.count }}
          </div>
        </div>

        <!-- 浏览器指纹 -->
        <div class="space-y-4">
          <h3 class="font-bold text-lg">浏览器指纹</h3>

          <!-- 基础信息 -->
          <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded">
            <h4 class="font-medium mb-2 text-sm">基础信息</h4>
            <div class="grid grid-cols-2 gap-3">
              <label class="text-sm">User-Agent
                <input v-model="form.fingerprint.userAgent" class="w-full h-9 px-2 rounded border text-xs" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">语言列表
                <input v-model="form.fingerprint.languages" placeholder="zh-CN,zh" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">平台
                <input v-model="form.fingerprint.platform" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">Do-Not-Track
                <select v-model="form.fingerprint.doNotTrack" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option value="null">未设置</option>
                  <option value="1">开启</option>
                  <option value="0">关闭</option>
                </select>
              </label>
            </div>
          </div>

          <!-- 系统信息 -->
          <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded">
            <h4 class="font-medium mb-2 text-sm">系统信息</h4>
            <div class="grid grid-cols-2 gap-3">
              <label class="text-sm">屏幕分辨率
                <input v-model="form.fingerprint.screen" placeholder="1920x1080" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">颜色深度
                <select v-model="form.fingerprint.screenColorDepth" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option value="24">24位</option>
                  <option value="32">32位</option>
                  <option value="16">16位</option>
                  <option value="8">8位</option>
                </select>
              </label>
              <label class="text-sm">像素比
                <select v-model="form.fingerprint.screenPixelRatio" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option value="1">1.0</option>
                  <option value="2">2.0</option>
                  <option value="1.5">1.5</option>
                  <option value="1.25">1.25</option>
                </select>
              </label>
              <label class="text-sm">时区偏移
                <input v-model="form.fingerprint.timezoneOffset" type="number" placeholder="-480" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
            </div>
          </div>

          <!-- 硬件信息 -->
          <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded">
            <h4 class="font-medium mb-2 text-sm">硬件信息</h4>
            <div class="grid grid-cols-2 gap-3">
              <label class="text-sm">CPU 核心数
                <input v-model="form.fingerprint.hardwareConcurrency" type="number" placeholder="8" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">内存(GB)
                <select v-model="form.fingerprint.deviceMemory" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option value="2">2GB</option>
                  <option value="4">4GB</option>
                  <option value="8">8GB</option>
                  <option value="16">16GB</option>
                  <option value="32">32GB</option>
                </select>
              </label>
              <label class="text-sm">电池电量(%)
                <input v-model="form.fingerprint.batteryLevel" type="number" placeholder="80" min="0" max="100" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">Cookie支持
                <select v-model="form.fingerprint.cookiesEnabled" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option :value="true">启用</option>
                  <option :value="false">禁用</option>
                </select>
              </label>
            </div>
          </div>

          <!-- 网络信息 -->
          <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded">
            <h4 class="font-medium mb-2 text-sm">网络信息</h4>
            <div class="grid grid-cols-2 gap-3">
              <label class="text-sm">连接类型
                <select v-model="form.fingerprint.connection.effectiveType" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option value="slow-2g">2G</option>
                  <option value="2g">2G</option>
                  <option value="3g">3G</option>
                  <option value="4g">4G</option>
                  <option value="5g">5G</option>
                </select>
              </label>
              <label class="text-sm">下载速度(Mbps)
                <input v-model="form.fingerprint.connection.downlink" type="number" placeholder="10" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">延迟(ms)
                <input v-model="form.fingerprint.connection.rtt" type="number" placeholder="100" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
            </div>
          </div>

          <!-- WebGL 信息 -->
          <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded">
            <h4 class="font-medium mb-2 text-sm">WebGL 信息</h4>
            <div class="grid grid-cols-2 gap-3">
              <label class="text-sm">WebGL Vendor
                <input v-model="form.fingerprint.webglVendor" placeholder="Google Inc." class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">WebGL Renderer
                <input v-model="form.fingerprint.webglRenderer" placeholder="ANGLE" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">WebGL Version
                <input v-model="form.fingerprint.webglVersion" placeholder="2.0" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
            </div>
          </div>

          <!-- Canvas 和音频 -->
          <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded">
            <h4 class="font-medium mb-2 text-sm">Canvas 和音频</h4>
            <div class="grid grid-cols-2 gap-3">
              <label class="text-sm">Canvas 噪声
                <select v-model="form.fingerprint.canvasNoise" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option :value="true">开启</option>
                  <option :value="false">关闭</option>
                </select>
              </label>
              <label class="text-sm">Canvas 指纹
                <input v-model="form.fingerprint.canvasFingerprint" placeholder="随机字符串" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">音频上下文
                <select v-model="form.fingerprint.audioContext" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option value="default">默认</option>
                  <option value="random">随机</option>
                  <option value="blocked">禁用</option>
                </select>
              </label>
              <label class="text-sm">字体指纹
                <select v-model="form.fingerprint.fontFingerprint" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                  <option value="default">默认</option>
                  <option value="random">随机</option>
                </select>
              </label>
            </div>
          </div>

          <!-- WebRTC 策略 -->
          <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded">
            <h4 class="font-medium mb-2 text-sm">WebRTC 策略</h4>
            <label class="text-sm block">
              <select v-model="form.fingerprint.webrtcPolicy" class="w-full h-9 px-2 rounded border" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'">
                <option value="default">默认</option>
                <option value="proxy_only">仅代理</option>
                <option value="disable_local">禁用内网</option>
                <option value="blocked">完全禁用</option>
              </select>
            </label>
          </div>

          <!-- 高级选项 -->
          <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded">
            <h4 class="font-medium mb-2 text-sm">高级选项</h4>
            <div class="grid grid-cols-1 gap-3">
              <label class="text-sm">插件列表
                <input v-model="form.fingerprint.plugins" placeholder="Chrome PDF Viewer,Native Client" class="w-full h-9 px-2 rounded border text-xs" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
              <label class="text-sm">MIME 类型
                <input v-model="form.fingerprint.mimeTypes" placeholder="application/pdf,text/html" class="w-full h-9 px-2 rounded border text-xs" :class="isDark? 'bg-[#233648] border-[#324d67]': 'bg-white border-[#cfdbe7]'" />
              </label>
            </div>
          </div>
        </div>
      </div>
      <div class="mt-4 flex justify-end gap-2">
        <button class="h-9 px-4 rounded" :class="isDark ? 'bg-[#1a2835] text-[#4c739a]' : 'bg-slate-100 text-slate-400'" @click="close">取消</button>
        <button class="h-9 px-4 rounded" :class="isDark ? 'bg-[#233648] text-white' : 'bg-[#e7edf3] text-[#0d141b]'" @click="submit">批量创建</button>
      </div>
    </div>
  </div>
</template>

<style scoped></style>
