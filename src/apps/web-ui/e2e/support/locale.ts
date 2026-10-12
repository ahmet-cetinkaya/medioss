// Locale, Accept-Language, and time zone emulation helpers.
import type { BrowserContextOptions } from "@playwright/test";

export type AppLocale = "en" | "tr";

export const NEW_YORK = "America/New_York";
export const ISTANBUL = "Europe/Istanbul";

const BROWSER_LOCALE: Record<AppLocale, string> = { en: "en-US", tr: "tr-TR" };
const ACCEPT_LANGUAGE: Record<AppLocale, string> = {
  en: "en-US,en;q=0.9",
  tr: "tr-TR,tr;q=0.9",
};

/** Options for `test.use(...)` or `browser.newContext(...)`. */
export function localeOptions(
  locale: AppLocale,
  timezoneId?: string,
): BrowserContextOptions {
  return {
    locale: BROWSER_LOCALE[locale],
    extraHTTPHeaders: { "Accept-Language": ACCEPT_LANGUAGE[locale] },
    ...(timezoneId ? { timezoneId } : {}),
  };
}

/** East-positive UTC offset in minutes of `timeZone` at `at` (New York EDT is -240). */
export function tzOffsetMinutes(
  timeZone: string,
  at: Date = new Date(),
): number {
  const parts = Object.fromEntries(
    new Intl.DateTimeFormat("en-US", {
      timeZone,
      hourCycle: "h23",
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    })
      .formatToParts(at)
      .map((part) => [part.type, part.value]),
  );
  const asUtc = Date.UTC(
    +parts.year,
    +parts.month - 1,
    +parts.day,
    +parts.hour,
    +parts.minute,
    +parts.second,
  );
  return Math.round((asUtc - Math.floor(at.getTime() / 1000) * 1000) / 60_000);
}

/** `YYYY-MM-DD` of `at` in `timeZone`. */
export function localDate(timeZone: string, at: Date = new Date()): string {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(at);
}

/** The UTC instant of wall-clock `date` `HH:MM` in `timeZone` (DST-aware). */
export function zonedTime(timeZone: string, date: string, time: string): Date {
  const guess = new Date(`${date}T${time}:00Z`);
  const offset = tzOffsetMinutes(timeZone, guess);
  const instant = new Date(guess.getTime() - offset * 60_000);
  // Re-check across a DST edge: the offset at the real instant wins.
  const corrected = tzOffsetMinutes(timeZone, instant);
  return corrected === offset
    ? instant
    : new Date(guess.getTime() - corrected * 60_000);
}
