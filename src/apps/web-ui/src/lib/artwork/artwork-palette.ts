// Generated artwork palettes (DESIGN_SYSTEM §7). Feeds the `aurora` utility
// (--aurora-base, --aurora-1..3). Deterministic: the seed only picks the
// color order and a mirror flag, never colors outside the category palette.
//
// Palettes per §7: meditation pink/violet/orange; sleep mint/sage/peach;
// soundscape green/peach; mood coral/orange; statistics blue/peach.
// Not in §7, chosen here: library uses the canvas stops (rose/lavender/peach,
// §2.1) as the "everything" palette; history uses lavender/periwinkle/rose,
// a cooler, quieter look for the past.

export type ArtworkCategory =
  | "meditation"
  | "sleep"
  | "soundscape"
  | "mood"
  | "statistics"
  | "history"
  | "library";

export type ArtworkPalette = {
  base: string;
  colors: readonly [string, string, string];
  /** Flip the blobs horizontally for variety (same colors). */
  mirrored: boolean;
};

export const CATEGORY_PALETTES: Record<
  ArtworkCategory,
  { base: string; colors: readonly [string, string, string] }
> = {
  meditation: { base: "#f6e8ee", colors: ["#f4b6c8", "#c9aaf2", "#ffc2a0"] },
  sleep: { base: "#eef4ef", colors: ["#bfe8d6", "#c8d5b9", "#ffd3b8"] },
  soundscape: { base: "#eff5ec", colors: ["#b5dfc0", "#ffcba8", "#d4ecc4"] },
  mood: { base: "#fbede6", colors: ["#f7a08a", "#ffc27a", "#f9c1b0"] },
  statistics: { base: "#edf1f8", colors: ["#a9c8f0", "#ffcba8", "#c7dbf5"] },
  library: { base: "#f3ecef", colors: ["#eaa8b8", "#c9aaf2", "#ebc0a6"] },
  history: { base: "#f1edf7", colors: ["#c9aaf2", "#b9c8f2", "#eaa8b8"] },
};

/** FNV-1a 32-bit hash of the seed's string form. */
function hash(seed: string | number): number {
  let h = 0x811c9dc5;
  for (const char of String(seed)) {
    h ^= char.codePointAt(0)!;
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

export function artworkPalette(
  seed: string | number,
  category: ArtworkCategory,
): ArtworkPalette {
  const { base, colors } = CATEGORY_PALETTES[category];
  const h = hash(seed);
  const shift = h % 3;
  return {
    base,
    colors: [colors[shift], colors[(shift + 1) % 3], colors[(shift + 2) % 3]],
    mirrored: (h & 4) !== 0,
  };
}
