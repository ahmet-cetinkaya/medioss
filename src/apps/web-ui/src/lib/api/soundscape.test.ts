import { describe, expect, test } from "bun:test";
import { withLang } from "./soundscape";

describe("withLang", () => {
  test("asks the server for the interface language", () => {
    expect(withLang("/api/soundscape/sounds", "tr")).toBe(
      "/api/soundscape/sounds?lang=tr",
    );
  });

  test("leaves the path alone without a language", () => {
    expect(withLang("/api/soundscape/sounds")).toBe("/api/soundscape/sounds");
  });
});
