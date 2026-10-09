// Locale-aware durations (FR-117): `Intl.DurationFormat` when the runtime has
// it, otherwise `Intl.NumberFormat` unit parts joined by `Intl.ListFormat`.

import type { Locale } from "../message";
import { numberFormatter } from "./number-format";

export type DurationStyle = "long" | "short";

export type DurationParts = { hours: number; minutes: number; seconds: number };

type DurationFormatter = { format(duration: Partial<DurationParts>): string };
type DurationFormatConstructor = new (
  locale: string,
  options: { style: DurationStyle },
) => DurationFormatter;

const durationCache = new Map<string, DurationFormatter>();
const listCache = new Map<string, Intl.ListFormat>();

/** Splits whole seconds (negative or non-finite count as 0) into h/m/s. */
export function splitSeconds(totalSeconds: number): DurationParts {
  const total = Number.isFinite(totalSeconds)
    ? Math.max(0, Math.floor(totalSeconds))
    : 0;
  return {
    hours: Math.floor(total / 3600),
    minutes: Math.floor((total % 3600) / 60),
    seconds: total % 60,
  };
}

/** Non-zero parts as unit strings ("1 hour", "5 dakika"); "0 seconds" when all are zero. */
export function formatUnitParts(
  parts: DurationParts,
  locale: Locale,
  style: DurationStyle = "long",
): string[] {
  const units = [
    ["hour", parts.hours],
    ["minute", parts.minutes],
    ["second", parts.seconds],
  ] as const;
  const present = units.filter(([, value]) => value > 0);
  return (present.length ? present : [units[2]]).map(([unit, value]) =>
    numberFormatter(locale, { style: "unit", unit, unitDisplay: style }).format(
      value,
    ),
  );
}

/** The fallback path, exported so it is tested even where `DurationFormat` exists. */
export function formatDurationFallback(
  ms: number,
  locale: Locale,
  style: DurationStyle = "long",
): string {
  const key = `${locale}|${style}`;
  let list = listCache.get(key);
  if (!list)
    listCache.set(
      key,
      (list = new Intl.ListFormat(locale, { type: "unit", style })),
    );
  return list.format(
    formatUnitParts(splitSeconds(Math.round(ms / 1000)), locale, style),
  );
}

export function formatDuration(
  ms: number,
  locale: Locale,
  style: DurationStyle = "long",
): string {
  const DurationFormat = (
    Intl as unknown as { DurationFormat?: DurationFormatConstructor }
  ).DurationFormat;
  const parts = splitSeconds(Math.round(ms / 1000));
  if (
    typeof DurationFormat !== "function" ||
    parts.hours + parts.minutes + parts.seconds === 0
  ) {
    return formatDurationFallback(ms, locale, style);
  }
  const key = `${locale}|${style}`;
  let formatter = durationCache.get(key);
  if (!formatter)
    durationCache.set(key, (formatter = new DurationFormat(locale, { style })));
  return formatter.format(parts);
}
