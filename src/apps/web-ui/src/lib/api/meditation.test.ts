import { describe, expect, test } from "bun:test";
import { programCoverUrl } from "./meditation";

describe("programCoverUrl", () => {
  test("returns the cover URL without a version", () => {
    expect(programCoverUrl(12)).toBe("/api/meditation/programs/12/cover");
  });

  test("adds a version query to refresh a replaced cover", () => {
    expect(programCoverUrl(12, 3)).toBe(
      "/api/meditation/programs/12/cover?v=3",
    );
  });
});
