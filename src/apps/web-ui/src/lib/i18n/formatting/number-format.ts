// Locale-aware numbers and percentages (FR-117). Formatters are cached per locale and options.

import type { Locale } from "../message";

const cache = new Map<string, Intl.NumberFormat>();

export function numberFormatter(
  locale: Locale,
  options: Intl.NumberFormatOptions = {},
): Intl.NumberFormat {
  const key = `${locale}|${JSON.stringify(options)}`;
  let formatter = cache.get(key);
  if (!formatter)
    cache.set(key, (formatter = new Intl.NumberFormat(locale, options)));
  return formatter;
}

export function formatNumber(
  value: number,
  locale: Locale,
  options?: Intl.NumberFormatOptions,
): string {
  return numberFormatter(locale, options).format(value);
}

/** Formats a fraction (0.375) as a percentage ("38%", "%38"). */
export function formatPercent(
  fraction: number,
  locale: Locale,
  options?: Intl.NumberFormatOptions,
): string {
  return numberFormatter(locale, { style: "percent", ...options }).format(
    fraction,
  );
}
