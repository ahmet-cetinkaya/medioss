import { describe, expect, test } from "bun:test";
import { PASSWORD_MIN_LENGTH, validateNewPassword } from "./password-rules";

describe("validateNewPassword", () => {
  test("the minimum is 8", () => {
    expect(PASSWORD_MIN_LENGTH).toBe(8);
  });

  test("7 characters is too short", () => {
    expect(validateNewPassword("abcdefg", "abcdefg")).toBe("tooShort");
  });

  test("8 characters passes", () => {
    expect(validateNewPassword("abcdefgh", "abcdefgh")).toBeNull();
  });

  test("8 code points passes even when UTF-16 length differs", () => {
    expect(validateNewPassword("ğğğğğğğğ", "ğğğğğğğğ")).toBeNull();
    // 7 astral code points are 14 UTF-16 units but still too short.
    const astral = "😀".repeat(7);
    expect(astral.length).toBe(14);
    expect(validateNewPassword(astral, astral)).toBe("tooShort");
  });

  test("an empty password is too short", () => {
    expect(validateNewPassword("", "")).toBe("tooShort");
  });

  test("a mismatch fails", () => {
    expect(validateNewPassword("abcdefgh", "abcdefgH")).toBe("mismatch");
  });

  test("length is checked before mismatch", () => {
    expect(validateNewPassword("short", "other")).toBe("tooShort");
  });
});
