// The Program primary action (FR-039, FR-041): which meditation it opens and
// how it is labeled. Members are in program order.

export type ProgramMember = {
  id: number;
  completed: boolean;
  progress_percent: number;
};

export type ProgramPrimaryAction =
  | { kind: "start" | "continue" | "playAgain"; meditationId: number }
  | { kind: "unavailable" };

/**
 * Empty → unavailable. All completed → "playAgain" on the first. Otherwise
 * the first not-completed member: "continue" when any member has progress
 * (completed or a position), else "start".
 */
export function programPrimaryAction(
  members: readonly ProgramMember[],
): ProgramPrimaryAction {
  if (members.length === 0) return { kind: "unavailable" };
  const next = members.find((member) => !member.completed);
  if (!next) return { kind: "playAgain", meditationId: members[0].id };
  const started = members.some(
    (member) => member.completed || member.progress_percent > 0,
  );
  return { kind: started ? "continue" : "start", meditationId: next.id };
}
