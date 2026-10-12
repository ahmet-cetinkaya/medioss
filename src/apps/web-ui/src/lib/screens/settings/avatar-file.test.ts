import { describe, expect, test } from "bun:test";
import { AVATAR_MAX_BYTES, avatarFileProblem } from "./avatar-file";

describe("avatarFileProblem", () => {
  test("rejects types other than JPEG, PNG, and WebP", () => {
    expect(avatarFileProblem({ type: "image/gif", size: 10 })).toBe("type");
    expect(avatarFileProblem({ type: "", size: 10 })).toBe("type");
    expect(avatarFileProblem({ type: "image/svg+xml", size: 10 })).toBe("type");
  });

  test("accepts JPEG, PNG, and WebP up to the limit", () => {
    expect(AVATAR_MAX_BYTES).toBe(2_097_152);
    for (const type of ["image/jpeg", "image/png", "image/webp"])
      expect(avatarFileProblem({ type, size: 1 })).toBeNull();
    expect(
      avatarFileProblem({ type: "image/png", size: 2_097_152 }),
    ).toBeNull();
  });

  test("rejects files one byte over 2 MB", () => {
    expect(avatarFileProblem({ type: "image/png", size: 2_097_153 })).toBe(
      "size",
    );
  });
});
