import { language, type Language } from "./state/settings";

const dict: Record<Language, Record<string, string>> = {
  "zh-CN": {
    "settings.title": "设置",
    "settings.tabs.language": "语言设置",
    "settings.tabs.theme": "主题设置",
    "settings.tabs.quota": "数量限制",
    "settings.language.label": "语言",
    "settings.theme.label": "主题",
    "settings.theme.system": "跟随系统",
    "settings.theme.light": "浅色",
    "settings.theme.dark": "深色",
    "settings.quota.label": "数量限制",
    "settings.quota.maxBrowsers": "最大可创建的浏览器数量",
    "settings.quota.placeholder": "不设置则不限数量",
  },
  "en-US": {
    "settings.title": "Settings",
    "settings.tabs.language": "Language",
    "settings.tabs.theme": "Theme",
    "settings.tabs.quota": "Quota Limit",
    "settings.language.label": "Language",
    "settings.theme.label": "Theme",
    "settings.theme.system": "System",
    "settings.theme.light": "Light",
    "settings.theme.dark": "Dark",
    "settings.quota.label": "Quota Limit",
    "settings.quota.maxBrowsers": "Max browser count",
    "settings.quota.placeholder": "Leave blank for unlimited",
  },
};

export function t(key: string): string {
  const l = language.value;
  return dict[l]?.[key] ?? key;
}
