// Pre-hydration boot script that sets `<html lang>` early (FR-116).
// Injected with `<script is:inline set:html={localeBootScript}>`. It is a
// self-contained IIFE that mirrors `detectLocale(readStoredLocale(), navigator.languages)`;
// `locale-boot.test.ts` checks parity.

import { LOCALE_STORAGE_KEY } from "./locale-storage";

export const localeBootScript = `(function () {
  var lang = "en";
  try {
    var stored = null;
    try {
      stored = localStorage.getItem(${JSON.stringify(LOCALE_STORAGE_KEY)});
    } catch (e) {}
    if (stored === "en" || stored === "tr") {
      lang = stored;
    } else {
      var languages = (navigator && navigator.languages) || [];
      for (var i = 0; i < languages.length; i++) {
        var primary = String(languages[i]).split(/[-_]/)[0].toLowerCase();
        if (primary === "en" || primary === "tr") {
          lang = primary;
          break;
        }
      }
    }
  } catch (e) {}
  document.documentElement.lang = lang;
})();`;
