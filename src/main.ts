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

// 全局关闭输入框首字母自动大写/自动纠正，避免内容被系统输入法改写
function normalizeInputBehavior() {
  const skipInputTypes = new Set([
    "checkbox",
    "radio",
    "range",
    "color",
    "file",
    "date",
    "datetime-local",
    "month",
    "week",
    "time",
    "number",
  ]);

  const applyToNode = (node: Element | ParentNode) => {
    if (!(node instanceof Element)) return;

    const applyToElement = (el: Element) => {
      if (!(el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement)) return;
      if (el instanceof HTMLInputElement && skipInputTypes.has((el.type || "").toLowerCase())) {
        return;
      }
      el.setAttribute("autocapitalize", "off");
      el.setAttribute("autocorrect", "off");
      el.setAttribute("spellcheck", "false");
    };

    if (node.matches("input, textarea")) {
      applyToElement(node);
    }
    node.querySelectorAll("input, textarea").forEach((el) => applyToElement(el));
  };

  applyToNode(document.documentElement);

  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      mutation.addedNodes.forEach((added) => {
        if (added instanceof Element) {
          applyToNode(added);
        }
      });
    }
  });

  observer.observe(document.documentElement, { childList: true, subtree: true });
}

normalizeInputBehavior();
