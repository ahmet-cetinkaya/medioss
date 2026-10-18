import { describe, expect, test } from "bun:test";
import {
  buildHeatmap,
  heatmapLevel,
  HEATMAP_LEVELS,
  monthMarks,
  weeksThatFit,
} from "./practice-heatmap";

describe("heatmapLevel", () => {
  test("zero minutes is level 0", () => {
    expect(heatmapLevel(0, 30)).toBe(0);
  });
  test("any practice is at least level 1 and the busiest day is the top level", () => {
    expect(heatmapLevel(1, 120)).toBe(1);
    expect(heatmapLevel(120, 120)).toBe(HEATMAP_LEVELS);
  });
});

describe("buildHeatmap", () => {
  const today = "2026-10-17"; // a Saturday
  const cells = (weeks: ReturnType<typeof buildHeatmap>) =>
    weeks.flat().filter((cell) => cell !== null);

  test("last30 has one cell per day, ending today", () => {
    const weeks = buildHeatmap([], "last30", today);
    const all = cells(weeks);
    expect(all).toHaveLength(30);
    expect(all.at(-1)?.date).toBe(today);
    expect(all[0]?.date).toBe("2026-09-18");
  });

  test("weeks start on Monday", () => {
    const weeks = buildHeatmap([], "last30", today);
    // 2026-09-18 is a Friday: Monday..Thursday of the first week are padding.
    expect(weeks[0].slice(0, 4)).toEqual([null, null, null, null]);
    expect(weeks[0][4]?.date).toBe("2026-09-18");
  });

  test("today range is a single cell", () => {
    expect(
      cells(buildHeatmap([{ date: today, minutes: 5 }], "today", today)),
    ).toEqual([{ date: today, minutes: 5, level: HEATMAP_LEVELS }]);
  });

  test("all time is the requested number of whole weeks ending with the current one", () => {
    const weeks = buildHeatmap([], "all", today, 30);
    expect(weeks).toHaveLength(30);
    expect(weeks.at(-1)?.[5]?.date).toBe(today);
    expect(weeks.at(-1)?.[6]).toBeNull();
  });

  test("a custom range spans its own first to last day", () => {
    const all = cells(
      buildHeatmap(
        [{ date: "2026-10-05", minutes: 10 }],
        { from: "2026-10-01", to: "2026-10-10" },
        today,
      ),
    );
    expect(all).toHaveLength(10);
    expect(all[0]?.date).toBe("2026-10-01");
    expect(all.at(-1)?.date).toBe("2026-10-10");
    expect(all.find((cell) => cell?.date === "2026-10-05")?.level).toBe(
      HEATMAP_LEVELS,
    );
  });

  test("a custom range ending after today stops at today", () => {
    const all = cells(
      buildHeatmap([], { from: "2026-10-15", to: "2026-12-31" }, today),
    );
    expect(all.map((cell) => cell?.date)).toEqual([
      "2026-10-15",
      "2026-10-16",
      "2026-10-17",
    ]);
  });

  test("a one-day custom range is a single cell", () => {
    expect(
      cells(buildHeatmap([], { from: "2026-10-12", to: "2026-10-12" }, today)),
    ).toHaveLength(1);
  });

  test("weeksThatFit clamps to 26..52", () => {
    expect(weeksThatFit(100, 20)).toBe(26);
    expect(weeksThatFit(800, 20)).toBe(40);
    expect(weeksThatFit(5000, 20)).toBe(52);
  });

  test("monthMarks labels each new month once", () => {
    const marks = monthMarks(buildHeatmap([], "last30", today));
    expect(marks.map((m) => m.date.slice(0, 7))).toEqual([
      "2026-09",
      "2026-10",
    ]);
  });

  test("minutes land on their day and scale by the busiest day", () => {
    const found = cells(
      buildHeatmap(
        [
          { date: "2026-10-10", minutes: 10 },
          { date: "2026-10-11", minutes: 40 },
        ],
        "last30",
        today,
      ),
    );
    expect(found.find((c) => c?.date === "2026-10-10")?.level).toBe(1);
    expect(found.find((c) => c?.date === "2026-10-11")?.level).toBe(
      HEATMAP_LEVELS,
    );
  });
});
