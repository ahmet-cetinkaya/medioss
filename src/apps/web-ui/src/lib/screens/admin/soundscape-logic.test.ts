import { describe, expect, test } from "bun:test";
import type { SoundscapeCategory, SoundscapeSound } from "$lib/api/soundscape";
import { selectedCategory, soundsInCategory } from "./soundscape-logic";

const category = (id: number) => ({ id }) as SoundscapeCategory;
const sound = (id: number, category_id: number) =>
  ({ id, category_id }) as SoundscapeSound;

describe("selectedCategory", () => {
  const categories = [category(1), category(2)];
  test("finds the chosen category", () =>
    expect(selectedCategory(categories, 2)?.id).toBe(2));
  test("falls back to the first for null or unknown ids", () => {
    expect(selectedCategory(categories, null)?.id).toBe(1);
    expect(selectedCategory(categories, 99)?.id).toBe(1);
  });
  test("is null without categories", () =>
    expect(selectedCategory([], null)).toBeNull());
});

describe("soundsInCategory", () => {
  test("keeps only the category's sounds", () =>
    expect(
      soundsInCategory([sound(1, 1), sound(2, 2), sound(3, 1)], 1).map(
        (s) => s.id,
      ),
    ).toEqual([1, 3]));
});
