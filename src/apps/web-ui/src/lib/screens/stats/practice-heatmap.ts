// Pure model for the practice heatmap: one cell per calendar day, in week
// columns starting on Monday, with an intensity level 0..HEATMAP_LEVELS
// derived from that day's practiced minutes (the figure statistics already hold).

import type { StatisticsDay } from "../../api/meditation-types";

export const HEATMAP_LEVELS = 4;
const DAYS_PER_WEEK = 7;
const MIN_WEEKS = 12;
const MAX_WEEKS = 53;

export type HeatmapCell = {
  date: string;
  minutes: number;
  /** 0 means no practice; 1..HEATMAP_LEVELS scales with the busiest day. */
  level: number;
  /** Days after `end` in the last week are padding and have no date. */
};

export type HeatmapWeek = (HeatmapCell | null)[];

const toIso = (date: Date): string => date.toISOString().slice(0, 10);

const parseIso = (iso: string): Date => new Date(`${iso}T00:00:00Z`);

const addDays = (date: Date, days: number): Date =>
  new Date(date.getTime() + days * 86_400_000);

/** Monday-based weekday index, 0..6. */
const weekdayIndex = (date: Date): number =>
  (date.getUTCDay() + DAYS_PER_WEEK - 1) % DAYS_PER_WEEK;

export function heatmapLevel(minutes: number, maxMinutes: number): number {
  if (minutes <= 0 || maxMinutes <= 0) return 0;
  return Math.max(1, Math.ceil((minutes / maxMinutes) * HEATMAP_LEVELS));
}

/**
 * Weeks covering `[start, end]`. The window is the last 30 days for "last30",
 * today only for "today", otherwise from the first practice day (at least
 * MIN_WEEKS, at most MAX_WEEKS back).
 */
export function buildHeatmap(
  calendar: readonly StatisticsDay[],
  range: "today" | "last30" | "all",
  today: string,
): HeatmapWeek[] {
  const end = parseIso(today);
  const minutesByDate = new Map(calendar.map((d) => [d.date, d.minutes]));
  const maxMinutes = Math.max(0, ...calendar.map((d) => d.minutes));

  let start: Date;
  if (range === "today") start = end;
  else if (range === "last30") start = addDays(end, -29);
  else {
    const first = calendar.map((d) => d.date).sort()[0];
    const earliest = addDays(end, -(MAX_WEEKS * DAYS_PER_WEEK - 1));
    const floor = addDays(end, -(MIN_WEEKS * DAYS_PER_WEEK - 1));
    const wanted = first ? parseIso(first) : floor;
    start = new Date(
      Math.max(earliest.getTime(), Math.min(wanted.getTime(), floor.getTime())),
    );
  }

  const gridStart = addDays(start, -weekdayIndex(start));
  const weeks: HeatmapWeek[] = [];
  for (
    let cursor = gridStart;
    cursor <= end;
    cursor = addDays(cursor, DAYS_PER_WEEK)
  ) {
    weeks.push(
      Array.from({ length: DAYS_PER_WEEK }, (_, offset) => {
        const day = addDays(cursor, offset);
        if (day < start || day > end) return null;
        const date = toIso(day);
        const minutes = minutesByDate.get(date) ?? 0;
        return { date, minutes, level: heatmapLevel(minutes, maxMinutes) };
      }),
    );
  }
  return weeks;
}
