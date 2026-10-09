import { describe, expect, test } from "bun:test";
import { appendPage, eventKey } from "./timeline-pages";

const event = (kind: string, id: number) => ({ kind, id });

describe("appendPage", () => {
  test("an overlapping item is kept once; nothing is omitted", () => {
    const first = [event("Mood", 3), event("MeditationNote", 3)];
    const second = [event("MeditationNote", 3), event("Mood", 2)];
    expect(appendPage(first, second)).toEqual([
      event("Mood", 3),
      event("MeditationNote", 3),
      event("Mood", 2),
    ]);
  });

  test("the same id under another kind is a different event", () => {
    expect(
      appendPage([event("Mood", 1)], [event("MeditationSession", 1)]),
    ).toHaveLength(2);
  });

  test("duplicates inside one page collapse; order is preserved", () => {
    expect(
      appendPage([], [event("Mood", 5), event("Mood", 5), event("Mood", 4)]),
    ).toEqual([event("Mood", 5), event("Mood", 4)]);
  });

  test("merging many pages yields every key exactly once", () => {
    const all = Array.from({ length: 63 }, (_, i) =>
      event(i % 3 ? "Mood" : "MeditationNote", i),
    );
    // Pages of 50 that overlap by one item at the boundary.
    const merged = appendPage(appendPage([], all.slice(0, 50)), all.slice(49));
    expect(merged.map(eventKey)).toEqual(all.map(eventKey));
  });

  test("does not mutate its inputs", () => {
    const current = [event("Mood", 1)];
    appendPage(current, [event("Mood", 2)]);
    expect(current).toEqual([event("Mood", 1)]);
  });
});
