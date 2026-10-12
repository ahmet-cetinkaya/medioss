// Sleep timer rules (PB-7): whole minutes from 5 to 120, a fade over the last
// 60 s, then stop. Pure: callers pass the clock in.

export const TIMER_MIN_MINUTES = 5;
export const TIMER_MAX_MINUTES = 120;
export const FADE_MS = 60_000;

/** Whole minutes from 5 to 120 inclusive; anything else is refused. */
export function isValidTimerMinutes(value: unknown): value is number {
  return (
    typeof value === "number" &&
    Number.isInteger(value) &&
    value >= TIMER_MIN_MINUTES &&
    value <= TIMER_MAX_MINUTES
  );
}

export interface TimerTick {
  /** Whole seconds left, rounded up (the m:ss countdown). */
  remainingSeconds: number;
  /** Fade gain for the audio: 1 before the last 60 s, then linear to 0. */
  gain: number;
  /** The deadline has passed: stop playback. */
  done: boolean;
}

export function timerTick(deadline: number, now: number): TimerTick {
  const remaining = Math.max(0, deadline - now);
  return {
    remainingSeconds: Math.ceil(remaining / 1000),
    gain: remaining >= FADE_MS ? 1 : remaining / FADE_MS,
    done: remaining === 0,
  };
}

/** The gain once the timer is cancelled or idle: the slider volumes apply. */
export const IDLE_GAIN = 1;
