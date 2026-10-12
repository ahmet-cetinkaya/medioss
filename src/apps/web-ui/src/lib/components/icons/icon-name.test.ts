import { describe, expect, test } from "bun:test";
import { ICON_NAMES } from "./icon-name";

describe("ICON_NAMES", () => {
  test("has no duplicates", () => {
    expect(new Set(ICON_NAMES).size).toBe(ICON_NAMES.length);
  });

  test("every name is kebab-case", () => {
    for (const name of ICON_NAMES) {
      expect(name).toMatch(/^[a-z]+(-[a-z]+)*$/);
    }
  });
});
