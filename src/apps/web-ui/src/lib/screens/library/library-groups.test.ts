import { describe, expect, test } from "bun:test";
import type { Category, Program } from "../../api/meditation";
import {
  filterGroups,
  groupPrograms,
  programState,
  UNCATEGORIZED,
} from "./library-groups";

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

describe("programState", () => {
  test("0 is not started, between is in progress, 1 or more is completed", () => {
    expect(programState(0)).toBe("not-started");
    expect(programState(0.01)).toBe("in-progress");
    expect(programState(0.99)).toBe("in-progress");
    expect(programState(1)).toBe("completed");
    expect(programState(1.2)).toBe("completed");
  });
});

describe("filterGroups", () => {
  const withProgress = (id: number, category_id: number, progress: number) => ({
    ...program(id, category_id),
    progress,
  });
  const groups = groupPrograms(
    [
      withProgress(1, 1, 1),
      withProgress(2, 1, 0.5),
      withProgress(3, 2, 0),
      withProgress(4, 2, 0),
    ],
    categories,
  );

  const ids = (filter: Parameters<typeof filterGroups>[1]) =>
    filterGroups(groups, filter).map((group) =>
      group.programs.map((p) => p.id),
    );

  test("all keeps every group and program", () => {
    expect(ids("all")).toEqual([
      [3, 4],
      [1, 2],
    ]);
  });

  test("each state keeps only its programs", () => {
    expect(ids("completed")).toEqual([[1]]);
    expect(ids("in-progress")).toEqual([[2]]);
    expect(ids("not-started")).toEqual([[3, 4]]);
  });

  test("a group left empty by the filter is dropped, not shown as a bare heading", () => {
    expect(filterGroups(groups, "completed").map((g) => g.key)).toEqual([
      "category-1",
    ]);
  });

  test("a filter nothing matches returns no groups", () => {
    expect(
      filterGroups(
        groupPrograms([withProgress(1, 1, 0)], categories),
        "completed",
      ),
    ).toEqual([]);
  });
});
