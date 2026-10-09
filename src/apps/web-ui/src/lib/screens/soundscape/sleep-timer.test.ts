import { describe, expect, test } from "bun:test";
import { audioVolume } from "$lib/soundscape/mix-volumes";
import {
  FADE_MS,
  IDLE_GAIN,
  isValidTimerMinutes,
  timerTick,
} from "./sleep-timer";

describe("isValidTimerMinutes", () => {
  test("accepts whole minutes from 5 to 120", () => {
    for (const minutes of [5, 6, 60, 119, 120])
      expect(isValidTimerMinutes(minutes)).toBe(true);
  });

  test("rejects 4, 121, 7.5 and non-numbers", () => {
    for (const value of [4, 121, 7.5, 0, -5, Number.NaN, "10", null])
      expect(isValidTimerMinutes(value)).toBe(false);
  });
});

describe("timerTick", () => {
  const deadline = 10 * 60_000;

  test("gain is 1 before the last 60 s", () => {
    expect(timerTick(deadline, 0)).toEqual({
      remainingSeconds: 600,
      gain: 1,
      done: false,
    });
    expect(timerTick(deadline, deadline - FADE_MS).gain).toBe(1);
  });

  test("gain falls linearly to 0 over the last 60 s, then stops", () => {
    expect(timerTick(deadline, deadline - 45_000).gain).toBeCloseTo(0.75);
    expect(timerTick(deadline, deadline - 30_000).gain).toBeCloseTo(0.5);
    expect(timerTick(deadline, deadline - 1_000).gain).toBeCloseTo(1 / 60);
    expect(timerTick(deadline, deadline - 1_000).done).toBe(false);
    expect(timerTick(deadline, deadline)).toEqual({
      remainingSeconds: 0,
      gain: 0,
      done: true,
    });
    expect(timerTick(deadline, deadline + 5_000).done).toBe(true);
  });

  test("the countdown rounds up to whole seconds", () => {
    expect(timerTick(deadline, deadline - 59_001).remainingSeconds).toBe(60);
    expect(timerTick(deadline, deadline - 500).remainingSeconds).toBe(1);
  });

  test("cancel restores the pre-fade volumes", () => {
    const sliders = [30, 70];
    const faded = timerTick(deadline, deadline - 15_000).gain;
    expect(sliders.map((v) => audioVolume(v, faded))).toEqual([
      0.3 * 0.25,
      0.7 * 0.25,
    ]);
    expect(sliders.map((v) => audioVolume(v, IDLE_GAIN))).toEqual([0.3, 0.7]);
  });
});
