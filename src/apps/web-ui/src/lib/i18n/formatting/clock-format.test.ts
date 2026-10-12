import { describe, expect, test } from "bun:test";
import { formatClock, formatClockSpoken } from "./clock-format";

describe("clock formatting", () => {
  test("m:ss below an hour, h:mm:ss from an hour on", () => {
    expect(formatClock(0)).toBe("0:00");
    expect(formatClock(59)).toBe("0:59");
    expect(formatClock(61)).toBe("1:01");
    expect(formatClock(3599.9)).toBe("59:59");
    expect(formatClock(3600)).toBe("1:00:00");
    expect(formatClock(3661)).toBe("1:01:01");
    expect(formatClock(-3)).toBe("0:00");
  });

  test("spoken position in English", () => {
    expect(formatClockSpoken(185, "en")).toBe("3 minutes 5 seconds");
    expect(formatClockSpoken(0, "en")).toBe("0 seconds");
    expect(formatClockSpoken(59, "en")).toBe("59 seconds");
    expect(formatClockSpoken(61, "en")).toBe("1 minute 1 second");
    expect(formatClockSpoken(180, "en")).toBe("3 minutes");
    expect(formatClockSpoken(3661, "en")).toBe("1 hour 1 minute 1 second");
  });

  test("spoken position in Turkish", () => {
    expect(formatClockSpoken(185, "tr")).toBe("3 dakika 5 saniye");
    expect(formatClockSpoken(0, "tr")).toBe("0 saniye");
    expect(formatClockSpoken(61, "tr")).toBe("1 dakika 1 saniye");
    expect(formatClockSpoken(3661, "tr")).toBe("1 saat 1 dakika 1 saniye");
  });
});
