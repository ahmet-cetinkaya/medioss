import { describe, expect, test } from "bun:test";
import {
  appendUnique,
  distributionRows,
  formatMean,
  trendPoints,
} from "./mood-insights";

describe("formatMean", () => {
  test("one decimal in English", () => {
    expect(formatMean(3.5, "en")).toBe("3.5");
    expect(formatMean(3, "en")).toBe("3.0");
    expect(formatMean(3.4666, "en")).toBe("3.5");
    expect(formatMean(2.04, "en")).toBe("2.0");
  });

  test("one decimal with a comma in Turkish", () => {
    expect(formatMean(3.5, "tr")).toBe("3,5");
    expect(formatMean(3, "tr")).toBe("3,0");
    expect(formatMean(4.96, "tr")).toBe("5,0");
  });
});

describe("distributionRows", () => {
  const counts = [1, 0, 2, 0, 0];
  const percentages = [100 / 3, 0, 200 / 3, 0, 0];

  test("scores 1–5 with counts and whole percents in English", () => {
    expect(distributionRows(counts, percentages, "en")).toEqual([
      { score: 1, count: 1, share: percentages[0] / 100, percent: "33%" },
      { score: 2, count: 0, share: 0, percent: "0%" },
      { score: 3, count: 2, share: percentages[2] / 100, percent: "67%" },
      { score: 4, count: 0, share: 0, percent: "0%" },
      { score: 5, count: 0, share: 0, percent: "0%" },
    ]);
  });

  test("Turkish puts the sign first", () => {
    const rows = distributionRows(counts, percentages, "tr");
    expect(rows.map((row) => row.percent)).toEqual([
      "%33",
      "%0",
      "%67",
      "%0",
      "%0",
    ]);
  });

  test("100% and missing values", () => {
    const rows = distributionRows([0, 0, 0, 4], [0, 0, 0, 100], "en");
    expect(rows[3]).toEqual({ score: 4, count: 4, share: 1, percent: "100%" });
    expect(rows[4]).toEqual({ score: 5, count: 0, share: 0, percent: "0%" });
  });
});

describe("trendPoints", () => {
  test("spreads days evenly; 5 at the top, 1 at the bottom", () => {
    expect(trendPoints([1, 3, 5], 120, 60, 10)).toEqual([
      { x: 10, y: 50 },
      { x: 60, y: 30 },
      { x: 110, y: 10 },
    ]);
  });

  test("a single day sits in the middle", () => {
    expect(trendPoints([4], 120, 60, 10)).toEqual([{ x: 60, y: 20 }]);
  });

  test("no days, no points", () => {
    expect(trendPoints([], 120, 60, 10)).toEqual([]);
  });
});

describe("appendUnique", () => {
  test("appends in order and skips ids already listed", () => {
    const first = [{ id: 3 }, { id: 2 }];
    expect(appendUnique(first, [{ id: 2 }, { id: 1 }])).toEqual([
      { id: 3 },
      { id: 2 },
      { id: 1 },
    ]);
    expect(first).toEqual([{ id: 3 }, { id: 2 }]);
  });
});
