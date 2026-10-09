// Translation core (plan D3, FR-119).
//
// Reactivity wiring: `translate()` is pure and takes the locale explicitly.
// `t()` reads the active locale through a getter that `locale.svelte.ts`
// registers when it is imported; that getter reads the `$state` field
// `locale.current`, so every inline `{t(m.title)}` in a template tracks the
// locale and re-renders on a switch without remounting (FR-113).
// This module stays rune-free so `bun test` and `*.messages.ts` can load it.
// Components import `t` from here; shell code imports it from
// `locale.svelte.ts` (which re-exports it), which guarantees the getter is
// registered on every page that renders the shell. Before registration `t()`
// renders English and, in a browser, warns once instead of failing silently.

import {
  DEFAULT_LOCALE,
  type Locale,
  type Message,
  type MessageParams,
  type MessageParamsOf,
} from "./message";

const PLACEHOLDER = /\{(\w+)\}/g;

function interpolate(text: string, params: MessageParams | undefined): string {
  if (!params) return text;
  return text.replace(PLACEHOLDER, (match, name: string) =>
    Object.hasOwn(params, name) ? String(params[name]) : match,
  );
}

function render(value: unknown, params: MessageParams | undefined): string {
  if (typeof value === "function") {
    const text: unknown = value(params ?? {});
    return typeof text === "string" ? text : "";
  }
  return typeof value === "string" ? interpolate(value, params) : "";
}

/**
 * Returns the message text in `locale`. Missing or empty text falls back to
 * English; it never returns an id. Unknown `{param}` placeholders stay intact.
 */
export function translate<M extends Message>(
  message: M,
  locale: Locale,
  params?: MessageParamsOf<M>,
): string {
  const text = render(message?.[locale], params);
  if (text !== "" || locale === DEFAULT_LOCALE) return text;
  return render(message?.[DEFAULT_LOCALE], params);
}

let localeGetter: (() => Locale) | null = null;
let warned = false;

/** Called once by `locale.svelte.ts`; the getter must read reactive state. */
export function registerLocaleGetter(getter: () => Locale): void {
  localeGetter = getter;
}

/** Translates into the active locale; reactive inside Svelte templates. */
export function t<M extends Message>(
  message: M,
  params?: MessageParamsOf<M>,
): string {
  if (!localeGetter && !warned && typeof window !== "undefined") {
    warned = true;
    console.warn(
      "i18n: t() called before locale.svelte.ts was imported; rendering English.",
    );
  }
  return translate(
    message,
    localeGetter ? localeGetter() : DEFAULT_LOCALE,
    params,
  );
}
