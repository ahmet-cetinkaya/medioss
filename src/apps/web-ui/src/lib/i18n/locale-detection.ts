// Locale detection (FR-114). Pure: it never reads or writes storage.
// `locale-boot.ts` mirrors this algorithm; keep them in sync.

import { DEFAULT_LOCALE, isLocale, type Locale } from "./message";

/** A valid stored choice wins; else the first en/tr primary subtag in order; else English. */
export function detectLocale(
  stored: string | null | undefined,
  languages: readonly string[] | null | undefined,
): Locale {
  if (isLocale(stored)) return stored;
  for (const tag of languages ?? []) {
    const primary = String(tag).split(/[-_]/)[0].toLowerCase();
    if (isLocale(primary)) return primary;
  }
  return DEFAULT_LOCALE;
}
