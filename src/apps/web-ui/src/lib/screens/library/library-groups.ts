// Library program groups (FR-031/032). Pure: programs grouped by meditation
// category in the server's category order, "Uncategorized" last. A category
// with no programs gets no group, so it gets no chip and no heading.

import type { Category, Program } from "../../api/meditation";

export const UNCATEGORIZED = "uncategorized";

export type LibraryGroup = {
  /** Chip value and DOM id part: `category-<id>` or `uncategorized`. */
  key: string;
  /** `null` for the Uncategorized group. */
  category: Category | null;
  programs: Program[];
};

export function groupPrograms(
  programs: readonly Program[],
  categories: readonly Category[],
): LibraryGroup[] {
  const known = new Set(categories.map((category) => category.id));
  const groups: LibraryGroup[] = categories
    .map((category) => ({
      key: `category-${category.id}`,
      category,
      programs: programs.filter(
        (program) => program.category_id === category.id,
      ),
    }))
    .filter((group) => group.programs.length > 0);
  // A program whose category is unknown is still shown, as uncategorized.
  const rest = programs.filter(
    (program) =>
      program.category_id === null || !known.has(program.category_id),
  );
  if (rest.length > 0)
    groups.push({ key: UNCATEGORIZED, category: null, programs: rest });
  return groups;
}
