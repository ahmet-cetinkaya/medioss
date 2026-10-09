// Mood insights (FR-057) and entry paging (FR-058), localized. Pure: the
// caller passes the locale (components pass `locale.current`).

import {
  formatNumber,
  formatPercent,
} from "../../i18n/formatting/number-format";
import type { Locale } from "../../i18n/message";

export const MOOD_SCORES = [1, 2, 3, 4, 5] as const;
export type Score = (typeof MOOD_SCORES)[number];

/** The mean score to one decimal: "3.5" in English, "3,5" in Turkish. */
export function formatMean(average: number, locale: Locale): string {
  return formatNumber(average, locale, {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  });
}

export interface DistributionRow {
  score: Score;
  count: number;
  /** 0–1, for the bar width. */
  share: number;
  /** Whole percent: "33%" in English, "%33" in Turkish. */
  percent: string;
}

/** Rows for scores 1–5; the server sends percentages as 0–100. */
export function distributionRows(
  counts: readonly number[],
  percentages: readonly number[],
  locale: Locale,
): DistributionRow[] {
  return MOOD_SCORES.map((score, index) => {
    const share = (percentages[index] ?? 0) / 100;
    return {
      score,
      count: counts[index] ?? 0,
      share,
      percent: formatPercent(share, locale, { maximumFractionDigits: 0 }),
    };
  });
}

/**
 * SVG coordinates for daily averages inside `width` x `height` minus `pad`:
 * days spread evenly left to right (one day sits in the middle), score 5 at
 * the top and 1 at the bottom.
 */
export function trendPoints(
  averages: readonly number[],
  width: number,
  height: number,
  pad: number,
): { x: number; y: number }[] {
  const step =
    averages.length > 1 ? (width - 2 * pad) / (averages.length - 1) : 0;
  return averages.map((average, index) => ({
    x: averages.length > 1 ? pad + index * step : width / 2,
    y: pad + ((5 - average) / 4) * (height - 2 * pad),
  }));
}

/** Appends a further page, skipping ids already listed. */
export function appendUnique<T extends { id: number }>(
  existing: readonly T[],
  incoming: readonly T[],
): T[] {
  const seen = new Set(existing.map((item) => item.id));
  return [...existing, ...incoming.filter((item) => !seen.has(item.id))];
}

export interface DayGroup<T> {
  /** Local calendar day, "YYYY-MM-DD". */
  key: string;
  /** Local midnight-agnostic instant (first entry), for the heading format. */
  iso: string;
  items: T[];
}

/** Groups entries (newest first) by local calendar day, keeping order. */
export function groupByDay<T extends { recorded_at: string }>(
  entries: readonly T[],
): DayGroup<T>[] {
  const groups: DayGroup<T>[] = [];
  for (const entry of entries) {
    const d = new Date(entry.recorded_at);
    const key = `${d.getFullYear()}-${d.getMonth() + 1}-${d.getDate()}`;
    const last = groups[groups.length - 1];
    if (last && last.key === key) last.items.push(entry);
    else groups.push({ key, iso: entry.recorded_at, items: [entry] });
  }
  return groups;
}
