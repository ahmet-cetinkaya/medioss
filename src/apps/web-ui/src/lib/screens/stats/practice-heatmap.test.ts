import { describe, expect, test } from "bun:test";
import { buildHeatmap, heatmapLevel, HEATMAP_LEVELS } from "./practice-heatmap";

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

  test("all time covers at least 12 weeks and reaches the first practice day", () => {
    expect(cells(buildHeatmap([], "all", today))).toHaveLength(84);
    const old = buildHeatmap(
      [{ date: "2025-12-01", minutes: 10 }],
      "all",
      today,
    );
    expect(cells(old)[0]?.date).toBe("2025-12-01");
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
