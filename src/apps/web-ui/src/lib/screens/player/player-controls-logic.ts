/** Seconds the skip buttons and Media Session seek handlers jump. */
export const SKIP_SECONDS = 10;

/** Playback speeds the single speed button cycles through (not remembered). */
export const SPEEDS = [1, 1.25, 1.5] as const;

/** Clamp a seek target into `[0, duration]`. */
export const clampSeek = (seconds: number, duration: number) =>
  Math.min(Math.max(0, seconds), duration);

/** The next speed index, wrapping after the last speed. */
export const nextSpeedIndex = (index: number) => (index + 1) % SPEEDS.length;

/** Whether an element-reported duration can replace the stored one. */
export const isUsableDuration = (seconds: number) =>
  Number.isFinite(seconds) && seconds > 0;
