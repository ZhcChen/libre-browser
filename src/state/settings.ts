import { ref, watch } from "vue";

export type Theme = "system" | "light" | "dark";
export type Language = "zh-CN" | "en-US";

const STORAGE_KEY = "libre_settings";

export const language = ref<Language>("zh-CN");
export const theme = ref<Theme>("system");
export const maxBrowsers = ref<number>(100);

function detectSystemTheme(): Exclude<Theme, "system"> {
  try {
    return window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } catch {
    return "dark";
  }
}

export function detectSystemLanguage(): Language {
  const l = (navigator.language || navigator.languages?.[0] || "en").toLowerCase();
  return l.startsWith("zh") ? "zh-CN" : "en-US";
}

export function loadSettings() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const s = JSON.parse(raw) as Partial<{
        language: Language;
        theme: Theme;
        maxBrowsers: number;
      }>;
      if (s.language) language.value = s.language;
      if (s.theme) theme.value = s.theme;
      if (typeof s.maxBrowsers === "number") maxBrowsers.value = s.maxBrowsers;
    } else {
      language.value = detectSystemLanguage();
      theme.value = "system";
      maxBrowsers.value = 100;
    }
  } catch {
    language.value = detectSystemLanguage();
  }
  applyTheme();
}

export function saveSettings() {
  localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({ language: language.value, theme: theme.value, maxBrowsers: maxBrowsers.value })
  );
}

export function resolveEffectiveTheme(): "light" | "dark" {
  return theme.value === "system" ? detectSystemTheme() : theme.value;
}

export function applyTheme() {
  const eff = resolveEffectiveTheme();
  const root = document.documentElement;
  root.classList.remove("theme-dark", "theme-light");
  root.classList.add(eff === "dark" ? "theme-dark" : "theme-light");
}

watch([language, theme, maxBrowsers], () => {
  saveSettings();
  applyTheme();
});
