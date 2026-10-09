import { describe, expect, test } from "bun:test";
import {
  artworkPalette,
  CATEGORY_PALETTES,
  type ArtworkCategory,
} from "./artwork-palette";

const sorted = (colors: readonly string[]) => [...colors].sort();

describe("artworkPalette", () => {
  test("the same seed gives the same palette", () => {
    expect(artworkPalette("Evening calm", "meditation")).toEqual(
      artworkPalette("Evening calm", "meditation"),
    );
    expect(artworkPalette(42, "sleep")).toEqual(artworkPalette(42, "sleep"));
    expect(artworkPalette(42, "sleep")).toEqual(artworkPalette("42", "sleep"));
  });

  test("seeds vary the order, not the colors", () => {
    const orders = new Set<string>();
    for (let seed = 0; seed < 50; seed++) {
      const palette = artworkPalette(seed, "mood");
      expect(sorted(palette.colors)).toEqual(
        sorted(CATEGORY_PALETTES.mood.colors),
      );
      orders.add(palette.colors.join());
    }
    expect(orders.size).toBeGreaterThan(1);
  });

  // §7: meditation pink/violet/orange; sleep mint/sage/peach; soundscape
  // green/peach; mood coral/orange; statistics blue/peach.
  const expected: Record<ArtworkCategory, string[]> = {
    meditation: ["#f4b6c8", "#c9aaf2", "#ffc2a0"],
    sleep: ["#bfe8d6", "#c8d5b9", "#ffd3b8"],
    soundscape: ["#b5dfc0", "#ffcba8", "#d4ecc4"],
    mood: ["#f7a08a", "#ffc27a", "#f9c1b0"],
    statistics: ["#a9c8f0", "#ffcba8", "#c7dbf5"],
    library: ["#eaa8b8", "#c9aaf2", "#ebc0a6"],
    history: ["#c9aaf2", "#b9c8f2", "#eaa8b8"],
  };

  for (const [category, colors] of Object.entries(expected)) {
    test(`${category} maps to its palette`, () => {
      const palette = artworkPalette("seed", category as ArtworkCategory);
      expect(sorted(palette.colors)).toEqual(sorted(colors));
      expect(palette.base).toBe(
        CATEGORY_PALETTES[category as ArtworkCategory].base,
      );
    });
  }

  test("different categories differ", () => {
    const sets = Object.keys(CATEGORY_PALETTES).map((c) =>
      sorted(artworkPalette(1, c as ArtworkCategory).colors).join(),
    );
    expect(new Set(sets).size).toBe(sets.length);
  });
});
