import { describe, expect, test } from "bun:test";
import {
  clampSeek,
  isUsableDuration,
  nextSpeedIndex,
  SPEEDS,
} from "./player-controls-logic";

describe("clampSeek", () => {
  test("keeps in-range targets", () => expect(clampSeek(30, 60)).toBe(30));
  test("clamps below zero", () => expect(clampSeek(-5, 60)).toBe(0));
  test("clamps above duration", () => expect(clampSeek(90, 60)).toBe(60));
});

describe("nextSpeedIndex", () => {
  test("advances", () => expect(nextSpeedIndex(0)).toBe(1));
  test("wraps after the last speed", () =>
    expect(nextSpeedIndex(SPEEDS.length - 1)).toBe(0));
});

describe("isUsableDuration", () => {
  test("accepts positive finite", () =>
    expect(isUsableDuration(12)).toBe(true));
  test("rejects zero, NaN and Infinity", () => {
    expect(isUsableDuration(0)).toBe(false);
    expect(isUsableDuration(Number.NaN)).toBe(false);
    expect(isUsableDuration(Number.POSITIVE_INFINITY)).toBe(false);
  });
});
