import type { SoundIconKey } from "./icon-name";

const BY_CATEGORY: Record<string, SoundIconKey> = {
  rain: "rain",
  nature: "forest",
  animals: "birds",
  binaural: "heartbeat",
  noise: "noise",
  places: "noise",
  mechanical: "noise",
  transport: "city",
  urban: "city",
  things: "music",
  ambient: "music",
};

/** Glyph for a sound without its own icon, chosen by category slug. */
export const categoryIconKey = (slug: string): SoundIconKey =>
  BY_CATEGORY[slug] ?? "music";
