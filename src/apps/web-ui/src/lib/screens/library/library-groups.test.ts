import { describe, expect, test } from "bun:test";
import type { Category, Program } from "../../api/meditation";
import { groupPrograms, UNCATEGORIZED } from "./library-groups";

const program = (id: number, category_id: number | null): Program => ({
  id,
  title: `Program ${id}`,
  description: null,
  has_cover: false,
  category_id,
  created_at: "2026-01-01T00:00:00Z",
  progress: 0,
});

const categories: Category[] = [
  { id: 2, name: "B" },
  { id: 1, name: "A" },
  { id: 3, name: "C" },
];

describe("groupPrograms", () => {
  test("follows category order, Uncategorized last, empty category omitted", () => {
    const groups = groupPrograms(
      [program(10, null), program(11, 1), program(12, 2), program(13, 1)],
      categories,
    );
    expect(groups.map((group) => group.key)).toEqual([
      "category-2",
      "category-1",
      UNCATEGORIZED,
    ]);
    expect(groups.map((group) => group.category?.name ?? null)).toEqual([
      "B",
      "A",
      null,
    ]);
    expect(groups[1].programs.map((p) => p.id)).toEqual([11, 13]);
    expect(groups[2].programs.map((p) => p.id)).toEqual([10]);
  });

  test("no Uncategorized group without uncategorized programs", () => {
    const groups = groupPrograms([program(1, 1)], categories);
    expect(groups.map((group) => group.key)).toEqual(["category-1"]);
  });

  test("a program with an unknown category is shown as uncategorized", () => {
    const groups = groupPrograms([program(1, 99)], categories);
    expect(groups.map((group) => group.key)).toEqual([UNCATEGORIZED]);
  });

  test("no programs → no groups", () => {
    expect(groupPrograms([], categories)).toEqual([]);
  });
});
