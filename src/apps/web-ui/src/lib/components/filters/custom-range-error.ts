// Why a custom range cannot be applied, as a message key (null = valid).
// Mirrors the server rules (api/date-range.ts) so the popover refuses
// locally instead of round-tripping a 400.

import {
  MAX_CUSTOM_RANGE_DAYS,
  isValidCustomRange,
  spanInDays,
  type CustomRange,
} from "$lib/api/date-range";

export type CustomRangeError = "missing" | "order" | "tooLong";

export function customRangeError(range: CustomRange): CustomRangeError | null {
  if (!range.from || !range.to) return "missing";
  if (isValidCustomRange(range)) return null;
  return spanInDays(range) > MAX_CUSTOM_RANGE_DAYS ? "tooLong" : "order";
}
