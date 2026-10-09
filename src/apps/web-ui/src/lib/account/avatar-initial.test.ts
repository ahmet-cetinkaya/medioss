import { describe, expect, test } from "bun:test";
import { avatarInitial } from "./avatar-initial";

describe("avatarInitial", () => {
  test("uses Turkish casing for dotted and dotless i", () => {
    expect(avatarInitial("ilkay")).toBe("İ");
    expect(avatarInitial("ırmak")).toBe("I");
  });

  test("trims leading whitespace", () => {
    expect(avatarInitial("  ayşe")).toBe("A");
  });

  test("returns null without a name", () => {
    expect(avatarInitial("")).toBeNull();
    expect(avatarInitial("   ")).toBeNull();
    expect(avatarInitial(null)).toBeNull();
    expect(avatarInitial(undefined)).toBeNull();
  });

  test("keeps a whole astral code point", () => {
    expect(avatarInitial("😀abc")).toBe("😀");
  });
});
