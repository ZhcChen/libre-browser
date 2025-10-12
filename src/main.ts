import { createApp } from "vue";
import "./styles/tailwind.css";
import "./assets/fonts.css";
import "./assets/theme.css";
import { loadSettings, applyTheme } from "./state/settings";
import App from "./App.vue";
import router from "./router";

loadSettings();
applyTheme();

createApp(App).use(router).mount("#app");

// 根据物理屏幕尺寸在启动时调整窗口大小并居中（兜底最小尺寸）
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

(() => {
  try {
    const win = (WebviewWindow as any).getByLabel
      ? (WebviewWindow as any).getByLabel("main")
      : null;
    if (!win) return;
    const sw = window.screen?.availWidth || 1280;
    const sh = window.screen?.availHeight || 800;
    const width = Math.max(1024, Math.floor(sw * 0.85));
    const height = Math.max(700, Math.floor(sh * 0.85));
    win.setSize({ width, height } as any);
    win.center();
  } catch {}
})();
