import i18n from "i18next";
import { initReactI18next, useTranslation } from "react-i18next";
import { enUSMessages } from "./locales/en-US";
import { zhCNMessages } from "./locales/zh-CN";
import type { Locale, MessageMap } from "./types";

const STORAGE_KEY = "gm_locale";
const LOCALE_EVENT = "gm:locale-change";

const messages: Record<Locale, MessageMap> = {
  "zh-CN": zhCNMessages,
  "en-US": enUSMessages,
};

const resources = {
  "zh-CN": { translation: messages["zh-CN"] },
  "en-US": { translation: messages["en-US"] },
};

function safeLocalStorageGet(key: string): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(key);
}

function safeLocalStorageSet(key: string, value: string) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(key, value);
}

export function getLocale(): Locale {
  const raw = safeLocalStorageGet(STORAGE_KEY);
  if (raw === "en-US" || raw === "zh-CN") {
    return raw;
  }
  return "zh-CN";
}

function normalizeLocale(raw?: string | null): Locale {
  return raw === "en-US" ? "en-US" : "zh-CN";
}

if (!i18n.isInitialized) {
  void i18n.use(initReactI18next).init({
    resources,
    lng: getLocale(),
    fallbackLng: "zh-CN",
    interpolation: { escapeValue: false },
    keySeparator: false,
  });
}

export function setLocale(locale: Locale) {
  safeLocalStorageSet(STORAGE_KEY, locale);
  void i18n.changeLanguage(locale);
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent(LOCALE_EVENT, { detail: locale }));
  }
}

export function translate(
  key: string,
  params?: Record<string, string | number>,
  locale?: Locale,
): string {
  return i18n.t(key, { ...(params ?? {}), lng: locale ?? undefined });
}

export function useI18n() {
  const { t, i18n: instance } = useTranslation();
  const locale = normalizeLocale(instance.resolvedLanguage ?? instance.language);

  return {
    locale,
    setLocale,
    t: (key: string, params?: Record<string, string | number>) =>
      t(key, params),
  };
}
