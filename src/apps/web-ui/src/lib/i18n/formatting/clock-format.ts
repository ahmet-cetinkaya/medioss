// Playback clock: digits for display, words for assistive technology (FR-045).

import type { Locale } from "../message";
import { formatUnitParts, splitSeconds } from "./duration-format";

const pad = (value: number): string => String(value).padStart(2, "0");

/** "m:ss" below an hour, "h:mm:ss" from an hour on. */
export function formatClock(sec: number): string {
  const { hours, minutes, seconds } = splitSeconds(sec);
  return hours > 0
    ? `${hours}:${pad(minutes)}:${pad(seconds)}`
    : `${minutes}:${pad(seconds)}`;
}

/** "3 minutes 5 seconds" / "3 dakika 5 saniye"; "0 seconds" at zero. */
export function formatClockSpoken(sec: number, locale: Locale): string {
  return formatUnitParts(splitSeconds(sec), locale, "long").join(" ");
}
