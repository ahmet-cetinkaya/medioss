// A date filter shared by History and Statistics: one of the server presets
// or a custom span of inclusive local dates (`YYYY-MM-DD`). The API takes
// either `range=<preset>` or `from=&to=`, never both; the client builds
// exactly one form.

/** The longest custom span the API accepts, in inclusive days. */
export const MAX_CUSTOM_RANGE_DAYS = 366;

export interface CustomRange {
  from: string;
  to: string;
}

export type DateRangeValue<Preset extends string = string> =
  Preset | CustomRange;

export const isCustomRange = (value: DateRangeValue): value is CustomRange =>
  typeof value === "object";

/** Query-string part for a preset or a custom range. */
export function rangeQuery(value: DateRangeValue): string {
  return isCustomRange(value)
    ? `from=${encodeURIComponent(value.from)}&to=${encodeURIComponent(value.to)}`
    : `range=${value}`;
}

const MS_PER_DAY = 86_400_000;

/** Inclusive number of days from `from` to `to`; NaN when either is invalid. */
export function spanInDays(range: CustomRange): number {
  const from = Date.parse(`${range.from}T00:00:00Z`);
  const to = Date.parse(`${range.to}T00:00:00Z`);
  return Math.round((to - from) / MS_PER_DAY) + 1;
}

/** True when `from <= to` and the span fits the server maximum. */
export function isValidCustomRange(range: CustomRange): boolean {
  const days = spanInDays(range);
  return days >= 1 && days <= MAX_CUSTOM_RANGE_DAYS;
}
