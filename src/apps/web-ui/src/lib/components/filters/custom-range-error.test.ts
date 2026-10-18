import { describe, expect, test } from "bun:test";
import { customRangeError } from "./custom-range-error";

describe("customRangeError mirrors the server rules", () => {
  test("a valid span has no error", () => {
    expect(
      customRangeError({ from: "2026-09-10", to: "2026-09-20" }),
    ).toBeNull();
    expect(
      customRangeError({ from: "2026-09-10", to: "2026-09-10" }),
    ).toBeNull();
  });

  test("a missing date is reported first", () => {
    expect(customRangeError({ from: "", to: "2026-09-20" })).toBe("missing");
    expect(customRangeError({ from: "2026-09-10", to: "" })).toBe("missing");
  });

  test("from after to is an order error", () => {
    expect(customRangeError({ from: "2026-09-20", to: "2026-09-10" })).toBe(
      "order",
    );
  });

  test("more than 366 days is a too-long error", () => {
    expect(customRangeError({ from: "2025-09-29", to: "2026-09-30" })).toBe(
      "tooLong",
    );
    expect(
      customRangeError({ from: "2025-09-30", to: "2026-09-30" }),
    ).toBeNull();
  });
});
