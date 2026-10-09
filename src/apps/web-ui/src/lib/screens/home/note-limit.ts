// Quick check-in note limit (FR-108). The server trims the note and counts
// Unicode code points, so the client counts the same way. Over the limit the
// text is kept and saving is disabled; nothing is truncated.

export const NOTE_LIMIT = 10_000;

/** Code points of the trimmed note, as the server counts them. */
export function noteLength(note: string): number {
  return [...note.trim()].length;
}

/** How many characters the note is over the limit (0 when it fits). */
export function noteOverLimit(note: string): number {
  return Math.max(0, noteLength(note) - NOTE_LIMIT);
}
