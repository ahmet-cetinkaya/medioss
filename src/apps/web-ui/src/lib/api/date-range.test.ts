import { describe, expect, test } from "bun:test";
import {
  MAX_CUSTOM_RANGE_DAYS,
  isCustomRange,
  isValidCustomRange,
  rangeQuery,
  spanInDays,
} from "./date-range";

describe("rangeQuery", () => {
  test("a preset becomes range=<preset>", () => {
    expect(rangeQuery("last7")).toBe("range=last7");
  });

  test("a custom range becomes from/to and never carries range", () => {
    const query = rangeQuery({ from: "2026-09-10", to: "2026-09-20" });
    expect(query).toBe("from=2026-09-10&to=2026-09-20");
    expect(query).not.toContain("range=");
  });
});

describe("isCustomRange", () => {
  test("tells a preset from a custom range", () => {
    expect(isCustomRange("all")).toBe(false);
    expect(isCustomRange({ from: "2026-09-10", to: "2026-09-10" })).toBe(true);
  });
});

describe("custom range validity matches the API rules", () => {
  test("counts both end dates", () => {
    expect(spanInDays({ from: "2026-09-10", to: "2026-09-10" })).toBe(1);
    expect(spanInDays({ from: "2026-09-10", to: "2026-09-20" })).toBe(11);
  });

  test("accepts one day up to the maximum span", () => {
    expect(isValidCustomRange({ from: "2026-09-10", to: "2026-09-10" })).toBe(
      true,
    );
    expect(isValidCustomRange({ from: "2025-09-30", to: "2026-09-30" })).toBe(
      true,
    );
    expect(MAX_CUSTOM_RANGE_DAYS).toBe(366);
  });

  test("rejects a span over the maximum, an inverted range and bad dates", () => {
    expect(isValidCustomRange({ from: "2025-09-29", to: "2026-09-30" })).toBe(
      false,
    );
    expect(isValidCustomRange({ from: "2026-09-20", to: "2026-09-10" })).toBe(
      false,
    );
    expect(isValidCustomRange({ from: "", to: "2026-09-10" })).toBe(false);
  });
});
