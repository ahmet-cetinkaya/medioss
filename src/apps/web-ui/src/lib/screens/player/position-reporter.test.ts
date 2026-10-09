import { describe, expect, test } from "bun:test";
import { createPositionReporter } from "./position-reporter";

function setup() {
  let now = 0;
  const sent: number[] = [];
  const beacons: number[] = [];
  const reporter = createPositionReporter({
    mediaTimeMs: () => now,
    send: (ms) => sent.push(ms),
    beacon: (ms) => beacons.push(ms),
  });
  return {
    reporter,
    sent,
    beacons,
    at(ms: number) {
      now = ms;
    },
  };
}

describe("position reporter (PB-2)", () => {
  test("reports at 5.0 s of media time from 0, not at 4.9 s", () => {
    const { reporter, sent, at } = setup();
    reporter.play();
    at(4_900);
    reporter.progress();
    expect(sent).toEqual([]);
    at(5_000);
    reporter.progress();
    expect(sent).toEqual([5_000]);
    at(9_900);
    reporter.progress();
    expect(sent).toEqual([5_000]);
    at(10_000);
    reporter.progress();
    expect(sent).toEqual([5_000, 10_000]);
  });

  test("measures from the resume position, and a seek counts", () => {
    const { reporter, sent, at } = setup();
    reporter.reset(6_000);
    reporter.play();
    at(10_900);
    reporter.progress();
    expect(sent).toEqual([]);
    at(11_000);
    reporter.progress();
    expect(sent).toEqual([11_000]);
    at(1_000);
    reporter.progress();
    expect(sent).toEqual([11_000, 1_000]);
  });

  test("does not report progress while paused", () => {
    const { reporter, sent, at } = setup();
    at(20_000);
    reporter.progress();
    expect(sent).toEqual([]);
  });

  test("reports immediately on pause", () => {
    const { reporter, sent, at } = setup();
    reporter.play();
    at(1_234.9);
    reporter.pause();
    expect(sent).toEqual([1_234]);
  });

  test("sends one beacon per hide while playing", () => {
    const { reporter, sent, beacons, at } = setup();
    reporter.play();
    at(3_000);
    reporter.hide();
    reporter.hide(); // visibilitychange + pagehide of the same hide
    expect(beacons).toEqual([3_000]);
    reporter.show();
    at(4_000);
    reporter.hide();
    expect(beacons).toEqual([3_000, 4_000]);
    expect(sent).toEqual([]);
  });

  test("sends no beacon while paused", () => {
    const { reporter, beacons, at } = setup();
    reporter.play();
    at(2_000);
    reporter.pause();
    reporter.hide();
    expect(beacons).toEqual([]);
  });
});
