// Addable-sound groups for the Player's mixer dialog (owner feedback round
// 2). Pure: sounds grouped by their soundscape category in category order,
// "Uncategorized" last (the Soundscape screen's library pattern). A category
// with no addable sound gets no group.

import type { SoundscapeCategory, SoundscapeSound } from "$lib/api/soundscape";

export const UNCATEGORIZED = "uncategorized";

export type MixAddGroup = {
  /** DOM id part: `category-<id>` or `uncategorized`. */
  key: string;
  /** null for the Uncategorized group. */
  category: SoundscapeCategory | null;
  sounds: SoundscapeSound[];
};

export function groupAddableSounds(
  sounds: readonly SoundscapeSound[],
  categories: readonly SoundscapeCategory[],
): MixAddGroup[] {
  const known = new Set(categories.map((category) => category.id));
  const groups: MixAddGroup[] = categories
    .map((category) => ({
      key: `category-${category.id}`,
      category,
      sounds: sounds.filter((sound) => sound.category_id === category.id),
    }))
    .filter((group) => group.sounds.length > 0);
  // A sound whose category is unknown is still addable, as uncategorized.
  const rest = sounds.filter((sound) => !known.has(sound.category_id));
  if (rest.length > 0)
    groups.push({ key: UNCATEGORIZED, category: null, sounds: rest });
  return groups;
}
