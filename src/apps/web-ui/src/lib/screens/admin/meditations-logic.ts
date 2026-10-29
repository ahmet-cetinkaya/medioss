import type { Category, Meditation, Program } from "$lib/api/meditation";

export type MeditationsData = {
  programs: Program[];
  categories: Category[];
  library: Meditation[];
};

/** Category filter: every program, only uncategorized ones, or one category id. */
export type CategoryFilter = "all" | "none" | number;

export const DESCRIPTION_MAX = 2000;

export const descriptionLength = (text: string) => [...text].length;

// `/admin/meditations/` → null; `/admin/meditations/12/` → 12.
export function parseProgramId(pathname: string): number | null {
  const match = pathname.match(/^\/admin\/meditations\/(\d+)\/?$/);
  return match ? Number(match[1]) : null;
}

/**
 * The program a deep link may open: `null` when the link names no program or
 * one that no longer exists, so a stale URL falls back to the programs list.
 */
export function programToOpen(
  requestedId: number | null,
  programs: readonly Program[],
): number | null {
  if (requestedId === null) return null;
  return programs.some((program) => program.id === requestedId)
    ? requestedId
    : null;
}

export function matchesFilter(
  program: Program,
  filter: CategoryFilter,
): boolean {
  if (filter === "all") return true;
  if (filter === "none") return program.category_id === null;
  return program.category_id === filter;
}

/** Copy of `list` with the items at `from` and `to` swapped. */
export function swapped<T>(list: readonly T[], from: number, to: number): T[] {
  const next = [...list];
  [next[from], next[to]] = [next[to], next[from]];
  return next;
}
