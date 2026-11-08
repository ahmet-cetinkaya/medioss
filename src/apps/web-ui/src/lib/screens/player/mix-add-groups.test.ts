import { describe, expect, test } from "bun:test";
import type { SoundscapeCategory, SoundscapeSound } from "$lib/api/soundscape";
import { groupAddableSounds, UNCATEGORIZED } from "./mix-add-groups";

const sound = (id: number, category_id: number): SoundscapeSound => ({
  id,
  name: `Sound ${id}`,
  category_id,
  content_type: "audio/ogg",
  size_bytes: 1,
  duration_ms: 1_000,
  created_at: "2026-01-01T00:00:00Z",
  builtin: false,
  icon_key: null,
  has_icon: false,
});

const categories: SoundscapeCategory[] = [
  { id: 2, name: "Weather", slug: "weather" },
  { id: 1, name: "Studio", slug: "studio" },
  { id: 3, name: "Empty", slug: "empty" },
];

describe("groupAddableSounds", () => {
  test("follows category order, Uncategorized last, empty category omitted", () => {
    const groups = groupAddableSounds(
      [sound(10, 99), sound(11, 1), sound(12, 2), sound(13, 1)],
      categories,
    );
    expect(groups.map((group) => group.key)).toEqual([
      "category-2",
      "category-1",
      UNCATEGORIZED,
    ]);
    expect(groups.map((group) => group.category?.name ?? null)).toEqual([
      "Weather",
      "Studio",
      null,
    ]);
    expect(groups[1].sounds.map((s) => s.id)).toEqual([11, 13]);
    expect(groups[2].sounds.map((s) => s.id)).toEqual([10]);
  });

  test("no Uncategorized group without stray sounds", () => {
    const groups = groupAddableSounds([sound(1, 1)], categories);
    expect(groups.map((group) => group.key)).toEqual(["category-1"]);
  });

  test("no sounds → no groups", () => {
    expect(groupAddableSounds([], categories)).toEqual([]);
  });
});
