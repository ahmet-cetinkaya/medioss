import { describe, expect, test } from "bun:test";
import { formatNumber, formatPercent, numberFormatter } from "./number-format";

describe("number formatting", () => {
  test("formats 1234.5 per locale", () => {
    expect(formatNumber(1234.5, "en")).toBe(
      new Intl.NumberFormat("en").format(1234.5),
    );
    expect(formatNumber(1234.5, "tr")).toBe(
      new Intl.NumberFormat("tr").format(1234.5),
    );
    expect(formatNumber(1234.5, "en")).toBe("1,234.5");
    expect(formatNumber(1234.5, "tr")).toBe("1.234,5");
  });

  test("formats 0.375 as a percentage per locale", () => {
    expect(formatPercent(0.375, "en")).toBe(
      new Intl.NumberFormat("en", { style: "percent" }).format(0.375),
    );
    expect(formatPercent(0.375, "tr")).toBe(
      new Intl.NumberFormat("tr", { style: "percent" }).format(0.375),
    );
    expect(formatPercent(0.375, "en")).not.toBe(formatPercent(0.375, "tr"));
    expect(formatPercent(0.375, "en", { maximumFractionDigits: 1 })).toBe(
      "37.5%",
    );
  });

  test("caches formatters per locale and options", () => {
    expect(numberFormatter("en")).toBe(numberFormatter("en"));
    expect(numberFormatter("en")).not.toBe(numberFormatter("tr"));
    expect(numberFormatter("tr", { style: "percent" })).not.toBe(
      numberFormatter("tr"),
    );
  });
});
