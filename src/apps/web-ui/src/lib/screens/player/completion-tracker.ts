// Completion recording (PB-3). Pure: the wall clock and the sender are
// injected. One completion per play-through: repeated `ended` events send
// nothing more until a play after the end starts a new play-through.
// `started_at` is the first play of the play-through, or the end time minus
// the duration when no play was observed.

export type CompletionTrackerDeps = {
  /** Wall-clock time in epoch milliseconds. */
  now: () => number;
  send: (startedAt: string, finishedAt: string) => void;
};

export type CompletionTracker = {
  play(): void;
  ended(durationMs: number): void;
};

export function createCompletionTracker({
  now,
  send,
}: CompletionTrackerDeps): CompletionTracker {
  let startedAt: number | null = null;
  let finished = false;

  return {
    play() {
      if (finished) {
        finished = false;
        startedAt = null;
      }
      startedAt ??= now();
    },
    ended(durationMs) {
      if (finished) return;
      finished = true;
      const end = now();
      const start = startedAt ?? end - Math.max(0, durationMs);
      send(new Date(start).toISOString(), new Date(end).toISOString());
    },
  };
}
