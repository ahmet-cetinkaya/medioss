import { describe, expect, test } from "bun:test";
import {
  programPrimaryAction,
  type ProgramMember,
} from "./program-primary-action";

const member = (
  id: number,
  progress_percent = 0,
  completed = false,
): ProgramMember => ({ id, progress_percent, completed });

describe("programPrimaryAction", () => {
  test("no progress → Start on the first meditation", () => {
    expect(programPrimaryAction([member(5), member(6), member(7)])).toEqual({
      kind: "start",
      meditationId: 5,
    });
  });

  test("some progress → Continue on the first not-completed one", () => {
    expect(programPrimaryAction([member(5), member(6, 40)])).toEqual({
      kind: "continue",
      meditationId: 5,
    });
    expect(
      programPrimaryAction([member(5, 100, true), member(6), member(7)]),
    ).toEqual({ kind: "continue", meditationId: 6 });
  });

  test("an earlier completed and a later in progress → Continue on the first not-completed", () => {
    expect(
      programPrimaryAction([member(5, 100, true), member(6), member(7, 30)]),
    ).toEqual({ kind: "continue", meditationId: 6 });
  });

  test("all completed → Play again on the first", () => {
    expect(
      programPrimaryAction([member(5, 100, true), member(6, 100, true)]),
    ).toEqual({ kind: "playAgain", meditationId: 5 });
  });

  test("empty → unavailable", () => {
    expect(programPrimaryAction([])).toEqual({ kind: "unavailable" });
  });
});
