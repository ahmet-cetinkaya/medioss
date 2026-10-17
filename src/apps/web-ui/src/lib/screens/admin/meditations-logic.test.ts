import { describe, expect, test } from "bun:test";
import type { Program } from "$lib/api/meditation";
import {
  descriptionLength,
  matchesFilter,
  parseProgramId,
  swapped,
} from "./meditations-logic";

const program = (categoryId: number | null) =>
  ({ category_id: categoryId }) as Program;

describe("parseProgramId", () => {
  test("reads the program id from the address", () => {
    expect(parseProgramId("/admin/meditations/")).toBeNull();
    expect(parseProgramId("/admin/meditations/12/")).toBe(12);
    expect(parseProgramId("/admin/meditations/12")).toBe(12);
    expect(parseProgramId("/admin/meditations/x/")).toBeNull();
  });
});

describe("matchesFilter", () => {
  test("all, none and one category", () => {
    expect(matchesFilter(program(null), "all")).toBe(true);
    expect(matchesFilter(program(null), "none")).toBe(true);
    expect(matchesFilter(program(3), "none")).toBe(false);
    expect(matchesFilter(program(3), 3)).toBe(true);
    expect(matchesFilter(program(3), 4)).toBe(false);
  });
});

describe("swapped", () => {
  test("returns a swapped copy and leaves the input alone", () => {
    const input = [1, 2, 3];
    expect(swapped(input, 0, 1)).toEqual([2, 1, 3]);
    expect(input).toEqual([1, 2, 3]);
  });
});

describe("descriptionLength", () => {
  test("counts code points", () => {
    expect(descriptionLength("a😀")).toBe(2);
  });
});
