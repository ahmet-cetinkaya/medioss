// The programs line on a favorite's row: every program the meditation
// belongs to, in the server's order, or null when it belongs to none.

import type { ProgramLabel } from "../../api/meditation-types";

const NAME_SEPARATOR = ", ";

export function favoriteProgramNames(
  programs: readonly ProgramLabel[] | undefined,
): string | null {
  if (!programs || programs.length === 0) return null;
  return programs.map((program) => program.title).join(NAME_SEPARATOR);
}
