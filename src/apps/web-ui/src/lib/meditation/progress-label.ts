// Progress labels (PB-19, FR-042), localized. Pure: the caller passes the
// locale (components pass `locale.current`, which keeps them reactive).
//
// "NN" is formatted with Intl percent style and no fraction digits, so the
// sign sits where each locale puts it: "40%" in English, "%40" in Turkish.
// The rounding is the legacy one (api/meditation.ts), unchanged.

import { formatPercent } from "../i18n/formatting/number-format";
import type { Locale } from "../i18n/message";
import { translate } from "../i18n/translate";
import { progressMessages } from "./progress.messages";

const percent = (whole: number, locale: Locale): string =>
  formatPercent(whole / 100, locale, { maximumFractionDigits: 0 });

/**
 * Program progress from 0..1: "Completed" at 1 or more, otherwise
 * "NN% complete" with NN = Math.round(progress * 100). Null while loading.
 */
export function programProgressLabel(
  progress: number | null | undefined,
  locale: Locale,
): string | null {
  if (progress == null) return null;
  if (progress >= 1) return translate(progressMessages.completed, locale);
  return translate(progressMessages.programPercent, locale, {
    percent: percent(Math.round(progress * 100), locale),
  });
}

/**
 * A meditation's own state: "Completed" when completed, otherwise "NN%" from
 * `progress_percent` (0..100). Null while loading.
 */
export function meditationProgressLabel(
  meditation:
    { completed: boolean; progress_percent: number } | null | undefined,
  locale: Locale,
): string | null {
  if (!meditation) return null;
  if (meditation.completed)
    return translate(progressMessages.completed, locale);
  return percent(meditation.progress_percent, locale);
}
