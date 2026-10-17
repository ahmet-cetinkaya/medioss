import { describe, expect, test } from "bun:test";
import { regionStatus } from "./mood-status";

describe("regionStatus", () => {
  test("a failure wins over everything", () =>
    expect(regionStatus(true, true, false)).toBe("error"));
  test("not loaded is loading", () =>
    expect(regionStatus(false, false, true)).toBe("loading"));
  test("loaded and empty is empty", () =>
    expect(regionStatus(false, true, true)).toBe("empty"));
  test("loaded with content is ready", () =>
    expect(regionStatus(false, true, false)).toBe("ready"));
});
