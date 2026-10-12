// Locale-aware dates and times (FR-117). Formatters are cached per locale and options.

import type { Locale } from "../message";

const cache = new Map<string, Intl.DateTimeFormat>();

function dateTimeFormatter(
  locale: Locale,
  options: Intl.DateTimeFormatOptions,
): Intl.DateTimeFormat {
  const key = `${locale}|${JSON.stringify(options)}`;
  let formatter = cache.get(key);
  if (!formatter)
    cache.set(key, (formatter = new Intl.DateTimeFormat(locale, options)));
  return formatter;
}

function format(
  date: Date,
  fallback: string,
  locale: Locale,
  options: Intl.DateTimeFormatOptions,
): string {
  return Number.isNaN(date.getTime())
    ? fallback
    : dateTimeFormatter(locale, options).format(date);
}

/** An instant (ISO 8601) as a date and time in the user's local time zone. */
export function formatDateTime(
  iso: string,
  locale: Locale,
  options: Intl.DateTimeFormatOptions = {
    dateStyle: "medium",
    timeStyle: "short",
  },
): string {
  return format(new Date(iso), iso, locale, options);
}

/** An instant (ISO 8601) as a date in the user's local time zone. */
export function formatDate(
  iso: string,
  locale: Locale,
  options: Intl.DateTimeFormatOptions = { dateStyle: "medium" },
): string {
  return format(new Date(iso), iso, locale, options);
}

/**
 * A calendar day ("YYYY-MM-DD") exactly as given, never shifted by the local
 * time zone (PB-11): built with `Date.UTC` and formatted in UTC.
 */
export function formatCalendarDay(
  day: string,
  locale: Locale,
  options: Intl.DateTimeFormatOptions = { dateStyle: "medium" },
): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(day);
  const date = match
    ? new Date(
        Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3])),
      )
    : new Date(NaN);
  return format(date, day, locale, { ...options, timeZone: "UTC" });
}
