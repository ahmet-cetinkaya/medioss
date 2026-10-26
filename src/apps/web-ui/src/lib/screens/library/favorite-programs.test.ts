import { describe, expect, test } from "bun:test";
import { favoriteProgramNames } from "./favorite-programs";

describe("favoriteProgramNames", () => {
  test("names a single program", () => {
    expect(favoriteProgramNames([{ id: 1, title: "Calm Mornings" }])).toBe(
      "Calm Mornings",
    );
  });

  test("lists every program in the given order", () => {
    expect(
      favoriteProgramNames([
        { id: 2, title: "Calm Mornings" },
        { id: 1, title: "Deep Sleep" },
      ]),
    ).toBe("Calm Mornings, Deep Sleep");
  });

  test("returns null when the meditation belongs to no program", () => {
    expect(favoriteProgramNames([])).toBeNull();
    expect(favoriteProgramNames(undefined)).toBeNull();
  });
});
