import { describe, expect, test } from "bun:test";
import { detectLocale } from "./locale-detection";
import type { Locale } from "./message";

const detectionMatrix: {
  stored: string | null;
  languages: string[];
  expected: Locale;
}[] = [
  { stored: null, languages: ["tr-TR", "en"], expected: "tr" },
  { stored: null, languages: ["en-US", "tr"], expected: "en" },
  { stored: null, languages: ["de-DE"], expected: "en" },
  { stored: null, languages: ["de", "tr"], expected: "tr" },
  { stored: null, languages: [], expected: "en" },
  { stored: "tr", languages: ["en"], expected: "tr" },
  { stored: "en", languages: ["tr-TR"], expected: "en" },
  { stored: "fr", languages: ["en"], expected: "en" },
  { stored: "medioss.tr", languages: ["tr"], expected: "tr" },
  { stored: "", languages: ["TR_tr"], expected: "tr" },
];

describe("detectLocale", () => {
  for (const { stored, languages, expected } of detectionMatrix) {
    test(`stored=${JSON.stringify(stored)} languages=${JSON.stringify(languages)} -> ${expected}`, () => {
      expect(detectLocale(stored, languages)).toBe(expected);
    });
  }

  test("tolerates missing languages", () => {
    expect(detectLocale(undefined, undefined)).toBe("en");
  });
});
