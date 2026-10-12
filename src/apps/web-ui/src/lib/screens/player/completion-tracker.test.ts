import { describe, expect, test } from "bun:test";
import { createCompletionTracker } from "./completion-tracker";

const T0 = Date.UTC(2026, 9, 4, 8, 0, 0);

function setup() {
  let now = T0;
  const sent: [string, string][] = [];
  const tracker = createCompletionTracker({
    now: () => now,
    send: (startedAt, finishedAt) => sent.push([startedAt, finishedAt]),
  });
  return {
    tracker,
    sent,
    at(ms: number) {
      now = ms;
    },
  };
}

const iso = (ms: number) => new Date(ms).toISOString();

describe("completion tracker (PB-3)", () => {
  test("sends exactly one completion over two ended events", () => {
    const { tracker, sent, at } = setup();
    tracker.play();
    at(T0 + 20_000);
    tracker.ended(20_000);
    at(T0 + 21_000);
    tracker.ended(20_000);
    expect(sent).toEqual([[iso(T0), iso(T0 + 20_000)]]);
  });

  test("started_at is the first play, not a later one", () => {
    const { tracker, sent, at } = setup();
    tracker.play();
    at(T0 + 5_000);
    tracker.play();
    at(T0 + 30_000);
    tracker.ended(20_000);
    expect(sent).toEqual([[iso(T0), iso(T0 + 30_000)]]);
  });

  test("without a play, started_at is the end minus the duration", () => {
    const { tracker, sent, at } = setup();
    at(T0 + 60_000);
    tracker.ended(20_000);
    expect(sent).toEqual([[iso(T0 + 40_000), iso(T0 + 60_000)]]);
  });

  test("a play after the end starts a new play-through", () => {
    const { tracker, sent, at } = setup();
    tracker.play();
    at(T0 + 20_000);
    tracker.ended(20_000);
    at(T0 + 25_000);
    tracker.play();
    at(T0 + 45_000);
    tracker.ended(20_000);
    expect(sent).toEqual([
      [iso(T0), iso(T0 + 20_000)],
      [iso(T0 + 25_000), iso(T0 + 45_000)],
    ]);
  });
});
