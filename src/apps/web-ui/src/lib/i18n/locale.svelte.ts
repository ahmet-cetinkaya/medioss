// Active interface locale (plan D3). Importing this module registers the
// reactive locale getter used by `t()` (see the header of `translate.ts`).
// Browser access is guarded, so SSR and `astro build` see English.

import { detectLocale } from "./locale-detection";
import { readStoredLocale, writeStoredLocale } from "./locale-storage";
import { isLocale, type Locale } from "./message";
import { registerLocaleGetter, t } from "./translate";

function initialLocale(): Locale {
  if (typeof window === "undefined") return "en";
  return detectLocale(readStoredLocale(), window.navigator?.languages);
}

class LocaleState {
  #current = $state<Locale>(initialLocale());

  get current(): Locale {
    return this.#current;
  }

  /** Explicit user choice: switches text, persists it (FR-115), and updates `<html lang>`. */
  set = (next: Locale): void => {
    if (!isLocale(next)) return;
    this.#current = next;
    writeStoredLocale(next);
    if (typeof document !== "undefined") document.documentElement.lang = next;
  };
}

export const locale = new LocaleState();

registerLocaleGetter(() => locale.current);

export { t };
