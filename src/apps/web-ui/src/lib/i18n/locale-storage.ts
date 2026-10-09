// Persisted language choice (FR-115). Storage may be unavailable (strict
// private browsing, sandboxed frames); failures fall back silently.
// Only `locale.set()` calls `writeStoredLocale`, so only an explicit choice persists.

import type { Locale } from "./message";

export const LOCALE_STORAGE_KEY = "medioss.locale";

export type LocaleStorage = Pick<Storage, "getItem" | "setItem">;

/** Returns the raw stored value, or null when absent or storage fails. */
export function readStoredLocale(storage?: LocaleStorage): string | null {
  try {
    return (
      (storage ?? globalThis.localStorage)?.getItem(LOCALE_STORAGE_KEY) ?? null
    );
  } catch {
    return null;
  }
}

export function writeStoredLocale(
  locale: Locale,
  storage?: LocaleStorage,
): void {
  try {
    (storage ?? globalThis.localStorage)?.setItem(LOCALE_STORAGE_KEY, locale);
  } catch {
    // Storage unavailable: the choice lasts for this page only.
  }
}
