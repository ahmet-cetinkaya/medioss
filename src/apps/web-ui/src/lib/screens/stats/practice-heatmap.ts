// Pure model for the practice heatmap: one cell per calendar day, in week
// columns starting on Monday, with an intensity level 0..HEATMAP_LEVELS
// derived from that day's practiced minutes (the figure statistics already hold).

import type { StatisticsDay } from "../../api/meditation-types";

export const HEATMAP_LEVELS = 4;
const DAYS_PER_WEEK = 7;
export const MIN_WEEKS = 26;
export const MAX_WEEKS = 52;

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

/** Weeks that fit `availableWidth` px at `pitch` px per column, clamped to the supported span. */
export function weeksThatFit(availableWidth: number, pitch: number): number {
  const fit = Math.floor(availableWidth / pitch);
  return Math.min(MAX_WEEKS, Math.max(MIN_WEEKS, fit));
}

/**
 * Weeks covering the range, newest last. "last30" is the last 30 days and
 * "today" a single cell; "all" is `weeks` whole weeks ending with the current one.
 */
export function buildHeatmap(
  calendar: readonly StatisticsDay[],
  range: "today" | "last30" | "all",
  today: string,
  weeks: number = MAX_WEEKS,
): HeatmapWeek[] {
  const end = parseIso(today);
  const minutesByDate = new Map(calendar.map((d) => [d.date, d.minutes]));
  const maxMinutes = Math.max(0, ...calendar.map((d) => d.minutes));

  const gridEnd = addDays(end, -weekdayIndex(end));
  const start =
    range === "today"
      ? end
      : range === "last30"
        ? addDays(end, -29)
        : addDays(gridEnd, -(weeks - 1) * DAYS_PER_WEEK);

  const gridStart = addDays(start, -weekdayIndex(start));
  const grid: HeatmapWeek[] = [];
  for (
    let cursor = gridStart;
    cursor <= end;
    cursor = addDays(cursor, DAYS_PER_WEEK)
  ) {
    grid.push(
      Array.from({ length: DAYS_PER_WEEK }, (_, offset) => {
        const day = addDays(cursor, offset);
        if (day < start || day > end) return null;
        const date = toIso(day);
        const minutes = minutesByDate.get(date) ?? 0;
        return { date, minutes, level: heatmapLevel(minutes, maxMinutes) };
      }),
    );
  }
  return grid;
}

export type MonthMark = { column: number; date: string };

/** The first week column of each calendar month, for labels above the grid. */
export function monthMarks(grid: readonly HeatmapWeek[]): MonthMark[] {
  const marks: MonthMark[] = [];
  let previousMonth = "";
  grid.forEach((week, column) => {
    const first = week.find((cell) => cell !== null);
    if (!first) return;
    const month = first.date.slice(0, 7);
    if (month !== previousMonth) marks.push({ column, date: first.date });
    previousMonth = month;
  });
  return marks;
}
