import { describe, expect, test } from "bun:test";
import { SOUND_ICON_KEYS, isSoundIconKey } from "./icon-name";
import { soundIconMessages } from "../../screens/admin/admin.messages";
import { categoryIconKey } from "./sound-icon-category";

describe("categoryIconKey", () => {
  test("maps known slugs and falls back to music", () => {
    expect(categoryIconKey("rain")).toBe("rain");
    expect(categoryIconKey("custom-category")).toBe("music");
  });
  test("always yields a bundled glyph key", () => {
    for (const slug of ["nature", "animals", "urban", "", "x"])
      expect(isSoundIconKey(categoryIconKey(slug))).toBe(true);
  });
});

describe("SOUND_ICON_KEYS", () => {
  test("has 24 unique kebab-case keys, each with en and tr labels", () => {
    expect(new Set(SOUND_ICON_KEYS).size).toBe(SOUND_ICON_KEYS.length);
    expect(SOUND_ICON_KEYS.length).toBe(24);
    for (const key of SOUND_ICON_KEYS) {
      expect(key).toMatch(/^[a-z0-9-]{1,64}$/);
      const label = soundIconMessages[key];
      expect(label.en).toBeTruthy();
      expect(label.tr).toBeTruthy();
    }
  });
});
