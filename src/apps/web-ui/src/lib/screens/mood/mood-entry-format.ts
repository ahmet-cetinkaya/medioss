import type { MoodEntry } from "$lib/api/mood";
import { moodScoreMessages } from "$lib/components/mood/mood-score.messages";
import type { MoodScore } from "$lib/components/mood/score-selector.svelte";
import { formatDateTime } from "$lib/i18n/formatting/date-time-format";
import { locale, t } from "$lib/i18n/locale.svelte";
import { moodMessages as m } from "./mood.messages";

/** The recorded time of an entry, in the current locale. */
export const entryTime = (entry: MoodEntry) =>
  formatDateTime(entry.recorded_at, locale.current);

/** "Mood 4 · Good": the entry's score and its label. */
export const entrySource = (entry: MoodEntry) =>
  t(m.entrySource, {
    score: entry.score,
    label: t(moodScoreMessages[entry.score as MoodScore]),
  });
