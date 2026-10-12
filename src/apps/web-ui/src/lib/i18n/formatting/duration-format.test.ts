import { describe, expect, test } from "bun:test";
import {
  formatDuration,
  formatDurationFallback,
  formatUnitParts,
  splitSeconds,
} from "./duration-format";

const unit = (
  locale: string,
  name: string,
  value: number,
  unitDisplay: "long" | "short" = "long",
) =>
  new Intl.NumberFormat(locale, {
    style: "unit",
    unit: name,
    unitDisplay,
  }).format(value);
const list = (
  locale: string,
  parts: string[],
  style: "long" | "short" = "long",
) => new Intl.ListFormat(locale, { type: "unit", style }).format(parts);

describe("duration formatting", () => {
  test("splitSeconds", () => {
    expect(splitSeconds(3930)).toEqual({ hours: 1, minutes: 5, seconds: 30 });
    expect(splitSeconds(-5)).toEqual({ hours: 0, minutes: 0, seconds: 0 });
    expect(splitSeconds(Number.NaN)).toEqual({
      hours: 0,
      minutes: 0,
      seconds: 0,
    });
  });

  test("formatUnitParts omits zero units and keeps 0 seconds at zero", () => {
    expect(formatUnitParts(splitSeconds(600), "en")).toEqual([
      unit("en", "minute", 10),
    ]);
    expect(formatUnitParts(splitSeconds(0), "tr")).toEqual([
      unit("tr", "second", 0),
    ]);
  });

  for (const locale of ["en", "tr"] as const) {
    test(`fallback path in ${locale} (600000 ms, 3930000 ms, short)`, () => {
      expect(formatDurationFallback(600000, locale)).toBe(
        unit(locale, "minute", 10),
      );
      expect(formatDurationFallback(3930000, locale)).toBe(
        list(locale, [
          unit(locale, "hour", 1),
          unit(locale, "minute", 5),
          unit(locale, "second", 30),
        ]),
      );
      expect(formatDurationFallback(3930000, locale, "short")).toBe(
        list(
          locale,
          [
            unit(locale, "hour", 1, "short"),
            unit(locale, "minute", 5, "short"),
            unit(locale, "second", 30, "short"),
          ],
          "short",
        ),
      );
    });

    test(`formatDuration in ${locale}`, () => {
      const DurationFormat = (
        Intl as unknown as {
          DurationFormat?: new (
            l: string,
            o: object,
          ) => { format(d: object): string };
        }
      ).DurationFormat;
      const expected = (d: object, fallbackMs: number) =>
        typeof DurationFormat === "function"
          ? new DurationFormat(locale, { style: "long" }).format(d)
          : formatDurationFallback(fallbackMs, locale);
      expect(formatDuration(600000, locale)).toBe(
        expected({ hours: 0, minutes: 10, seconds: 0 }, 600000),
      );
      expect(formatDuration(3930000, locale)).toBe(
        expected({ hours: 1, minutes: 5, seconds: 30 }, 3930000),
      );
      expect(formatDuration(0, locale)).toBe(unit(locale, "second", 0));
    });
  }

  test("formatDuration takes the fallback when Intl.DurationFormat is missing", () => {
    const intl = Intl as unknown as { DurationFormat?: unknown };
    const original = Object.getOwnPropertyDescriptor(intl, "DurationFormat");
    Object.defineProperty(intl, "DurationFormat", {
      value: undefined,
      configurable: true,
      writable: true,
    });
    try {
      expect(formatDuration(3930000, "tr")).toBe(
        list("tr", [
          unit("tr", "hour", 1),
          unit("tr", "minute", 5),
          unit("tr", "second", 30),
        ]),
      );
      expect(formatDuration(600000, "en")).toBe(unit("en", "minute", 10));
    } finally {
      if (original) Object.defineProperty(intl, "DurationFormat", original);
      else delete intl.DurationFormat;
    }
  });

  test("en and tr differ", () => {
    expect(formatDuration(3930000, "en")).toBe("1 hour, 5 minutes, 30 seconds");
    expect(formatDuration(3930000, "tr")).toBe("1 saat 5 dakika 30 saniye");
    expect(formatDurationFallback(3930000, "en")).toBe(
      "1 hour, 5 minutes, 30 seconds",
    );
    expect(formatDurationFallback(3930000, "tr")).toBe(
      "1 saat 5 dakika 30 saniye",
    );
  });
});
