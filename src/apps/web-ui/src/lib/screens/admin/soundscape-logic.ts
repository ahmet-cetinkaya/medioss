import type { SoundscapeCategory, SoundscapeSound } from "$lib/api/soundscape";

export type SoundscapeData = {
  categories: SoundscapeCategory[];
  sounds: SoundscapeSound[];
};

/** The chosen category, or the first one while nothing valid is chosen. */
export const selectedCategory = (
  categories: SoundscapeCategory[],
  selectedId: number | null,
) => categories.find((c) => c.id === selectedId) ?? categories[0] ?? null;

/** The sounds that belong to one category. */
export const soundsInCategory = (
  sounds: SoundscapeSound[],
  categoryId: number,
) => sounds.filter((sound) => sound.category_id === categoryId);
