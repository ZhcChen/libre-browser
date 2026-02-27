import { language, type Language } from "./state/settings";

const dict: Record<Language, Record<string, string>> = {
  "zh-CN": {
    "settings.title": "设置",
    "settings.tabs.language": "语言设置",
    "settings.tabs.theme": "主题设置",
    "settings.tabs.quota": "数量限制",
    "settings.tabs.cleanup": "缓存清理",
    "settings.language.label": "语言",
    "settings.theme.label": "主题",
    "settings.theme.system": "跟随系统",
    "settings.theme.light": "浅色",
    "settings.theme.dark": "深色",
    "settings.quota.label": "数量限制",
    "settings.quota.maxBrowsers": "最大可创建的浏览器数量",
    "settings.quota.placeholder": "不设置则不限数量",
    "settings.cleanup.label": "缓存清理",
    "settings.cleanup.desc": "清理不在浏览器列表中的历史 profile、图标和自定义 app 残留文件。",
    "settings.cleanup.action": "清理历史缓存",
    "settings.cleanup.running": "清理中...",
    "settings.cleanup.done": "清理完成：profiles {profiles} 个，custom apps {apps} 个，icons {icons} 个，告警 {warnings} 条",
    "settings.cleanup.failed": "清理失败",
  },
  "en-US": {
    "settings.title": "Settings",
    "settings.tabs.language": "Language",
    "settings.tabs.theme": "Theme",
    "settings.tabs.quota": "Quota Limit",
    "settings.tabs.cleanup": "Cache Cleanup",
    "settings.language.label": "Language",
    "settings.theme.label": "Theme",
    "settings.theme.system": "System",
    "settings.theme.light": "Light",
    "settings.theme.dark": "Dark",
    "settings.quota.label": "Quota Limit",
    "settings.quota.maxBrowsers": "Max browser count",
    "settings.quota.placeholder": "Leave blank for unlimited",
    "settings.cleanup.label": "Cache Cleanup",
    "settings.cleanup.desc": "Clean residual profile files, icons and custom app bundles that are no longer in the browser list.",
    "settings.cleanup.action": "Clean stale cache",
    "settings.cleanup.running": "Cleaning...",
    "settings.cleanup.done": "Done: profiles {profiles}, custom apps {apps}, icons {icons}, warnings {warnings}",
    "settings.cleanup.failed": "Cleanup failed",
  },
};

export function t(key: string): string {
  const l = language.value;
  return dict[l]?.[key] ?? key;
}
