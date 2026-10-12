import { describe, expect, test } from "bun:test";
import { NOTE_LIMIT, noteLength, noteOverLimit } from "./note-limit";

describe("note limit", () => {
  test("is 10 000 characters", () => {
    expect(NOTE_LIMIT).toBe(10_000);
  });

  test("counts code points of the trimmed note", () => {
    expect(noteLength("")).toBe(0);
    expect(noteLength("  calm  ")).toBe(4);
    expect(noteLength("ş🙂")).toBe(2);
  });

  test("is over only beyond the limit", () => {
    expect(noteOverLimit("a".repeat(NOTE_LIMIT))).toBe(0);
    expect(noteOverLimit(`  ${"a".repeat(NOTE_LIMIT)}  `)).toBe(0);
    expect(noteOverLimit("a".repeat(NOTE_LIMIT + 1))).toBe(1);
    expect(noteOverLimit("🙂".repeat(NOTE_LIMIT + 5))).toBe(5);
  });
});
