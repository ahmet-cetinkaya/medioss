import { describe, expect, test } from "bun:test";
import {
  formatCalendarDay,
  formatDate,
  formatDateTime,
} from "./date-time-format";

const instant = "2026-03-29T14:05:00Z";

describe("date and time formatting", () => {
  test("formatDateTime uses the local time zone per locale", () => {
    for (const locale of ["en", "tr"] as const) {
      const expected = new Intl.DateTimeFormat(locale, {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(new Date(instant));
      expect(formatDateTime(instant, locale)).toBe(expected);
    }
    expect(formatDateTime(instant, "en")).not.toBe(
      formatDateTime(instant, "tr"),
    );
  });

  test("formatDate per locale", () => {
    for (const locale of ["en", "tr"] as const) {
      expect(formatDate(instant, locale)).toBe(
        new Intl.DateTimeFormat(locale, { dateStyle: "medium" }).format(
          new Date(instant),
        ),
      );
    }
    expect(formatDate(instant, "en")).not.toBe(formatDate(instant, "tr"));
  });

  test("invalid input is returned unchanged instead of throwing", () => {
    expect(formatDate("not a date", "en")).toBe("not a date");
    expect(formatCalendarDay("2026-3-9", "tr")).toBe("2026-3-9");
  });

  test("formatCalendarDay never shifts the day (PB-11)", () => {
    for (const locale of ["en", "tr"] as const) {
      const expected = new Intl.DateTimeFormat(locale, {
        dateStyle: "long",
        timeZone: "UTC",
      }).format(Date.UTC(2026, 2, 29));
      expect(
        formatCalendarDay("2026-03-29", locale, { dateStyle: "long" }),
      ).toBe(expected);
    }
    expect(formatCalendarDay("2026-03-29", "en", { dateStyle: "long" })).toBe(
      "March 29, 2026",
    );
    expect(formatCalendarDay("2026-03-29", "tr", { dateStyle: "long" })).toBe(
      "29 Mart 2026",
    );
  });

  test("formatCalendarDay is stable across time zones, including a DST change day", () => {
    const previous = process.env.TZ;
    const outputs: string[] = [];
    try {
      for (const zone of [
        "America/New_York",
        "Pacific/Kiritimati",
        "Pacific/Pago_Pago",
        "Europe/Istanbul",
      ]) {
        process.env.TZ = zone;
        outputs.push(
          formatCalendarDay("2026-03-29", "en", {
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
          }),
        );
        // A local-time parse of the same day would land on 03/28 west of UTC.
        if (zone === "America/New_York")
          expect(new Date("2026-03-29").getDate()).toBe(28);
      }
    } finally {
      process.env.TZ = previous;
    }
    expect(new Set(outputs)).toEqual(new Set(["03/29/2026"]));
  });
});
