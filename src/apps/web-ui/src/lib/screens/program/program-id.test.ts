import { describe, expect, test } from "bun:test";
import { parseProgramId } from "./program-id";

describe("parseProgramId", () => {
  test("accepts a positive whole number", () => {
    expect(parseProgramId("?id=1")).toBe(1);
    expect(parseProgramId("?id=42&x=y")).toBe(42);
    expect(parseProgramId("?id=007")).toBe(7);
  });

  test("rejects a missing, empty, non-numeric, zero, or negative id", () => {
    for (const search of [
      "",
      "?",
      "?id=",
      "?id=abc",
      "?id=0",
      "?id=00",
      "?id=-1",
      "?id=1.5",
      "?id=1e3",
      "?id=%201",
      "?id=0x10",
      "?id=99999999999999999999",
      "?other=1",
    ])
      expect(parseProgramId(search)).toBeNull();
  });
});
