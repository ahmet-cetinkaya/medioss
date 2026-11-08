import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { soundTitle } from "./builtin-titles";
import { registerLocaleGetter, translate } from "$lib/i18n/translate";
import {
  builtinCategoryTitles,
  builtinSoundTitles,
} from "./builtin-titles.messages";

type ManifestEntry = { name: string; category_name: string };

const manifest: ManifestEntry[] = JSON.parse(
  readFileSync(
    new URL(
      "../../../../../../../assets/sounds/soundscape-builtin/manifest.json",
      import.meta.url,
    ),
    "utf8",
  ),
);

// Same rule as the server seeder: "cat-purring" is stored as "Cat Purring".
function storedName(slug: string): string {
  return slug
    .split("-")
    .filter(Boolean)
    .map((word) => word[0].toUpperCase() + word.slice(1))
    .join(" ");
}

describe("built-in titles", () => {
  test("every manifest sound has an English and a Turkish title", () => {
    for (const entry of manifest) {
      const name = storedName(entry.name);
      const message =
        builtinSoundTitles[name as keyof typeof builtinSoundTitles];
      expect(message, name).toBeDefined();
      expect(message.en, name).toBe(name);
      expect(translate(message, "tr").length, name).toBeGreaterThan(0);
    }
  });

  test("every manifest category has an English and a Turkish title", () => {
    for (const entry of manifest) {
      const message =
        builtinCategoryTitles[
          entry.category_name as keyof typeof builtinCategoryTitles
        ];
      expect(message, entry.category_name).toBeDefined();
      expect(message.en).toBe(entry.category_name);
      expect(translate(message, "tr").length).toBeGreaterThan(0);
    }
  });

  test("the maps hold no names the manifest lacks", () => {
    expect(Object.keys(builtinSoundTitles).sort()).toEqual(
      [...new Set(manifest.map((e) => storedName(e.name)))].sort(),
    );
    expect(Object.keys(builtinCategoryTitles).sort()).toEqual(
      [...new Set(manifest.map((e) => e.category_name))].sort(),
    );
  });

  test("a sound you uploaded keeps its name, even one a built-in also has", () => {
    registerLocaleGetter(() => "tr");
    expect(soundTitle({ name: "Beehive", builtin: true })).toBe("Arı Kovanı");
    expect(soundTitle({ name: "Beehive", builtin: false })).toBe("Beehive");
    expect(soundTitle({ name: "My storm", builtin: true })).toBe("My storm");
  });
});
