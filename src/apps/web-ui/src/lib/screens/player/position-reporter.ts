// Playback position reporting (PB-2, plan D10). Pure: the media clock and
// the two senders are injected, so `bun test` drives it without a browser.
//
// - While playing, `progress()` reports once the media time is at least
//   `intervalMs` away from the last report (a seek also counts).
// - `pause()` reports immediately.
// - `hide()` reports through the beacon sender (survives unload) while
//   playing, once per hide: `visibilitychange` (hidden) and `pagehide` often
//   fire together, so a second hide sends nothing until `show()`.

export const POSITION_INTERVAL_MS = 5_000;

export type PositionReporterDeps = {
  /** The current media time in milliseconds. */
  mediaTimeMs: () => number;
  /** A regular report (fetch). */
  send: (positionMs: number) => void;
  /** A report that must survive the page going away (sendBeacon). */
  beacon: (positionMs: number) => void;
  intervalMs?: number;
};

export type PositionReporter = {
  /** Sets the last reported position without sending (e.g. after resume). */
  reset(positionMs: number): void;
  play(): void;
  progress(): void;
  pause(): void;
  hide(): void;
  show(): void;
};

export function createPositionReporter({
  mediaTimeMs,
  send,
  beacon,
  intervalMs = POSITION_INTERVAL_MS,
}: PositionReporterDeps): PositionReporter {
  let lastMs = 0;
  let playing = false;
  let hideReported = false;

  const report = (sender: (positionMs: number) => void) => {
    lastMs = Math.max(0, Math.floor(mediaTimeMs()));
    sender(lastMs);
  };

  return {
    reset(positionMs) {
      lastMs = Math.max(0, Math.floor(positionMs));
    },
    play() {
      playing = true;
    },
    progress() {
      if (playing && Math.abs(Math.floor(mediaTimeMs()) - lastMs) >= intervalMs)
        report(send);
    },
    pause() {
      playing = false;
      report(send);
    },
    hide() {
      if (!playing || hideReported) return;
      hideReported = true;
      report(beacon);
    },
    show() {
      hideReported = false;
    },
  };
}
