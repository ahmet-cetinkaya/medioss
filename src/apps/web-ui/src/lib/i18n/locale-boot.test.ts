import { describe, expect, test } from "bun:test";
import { localeBootScript } from "./locale-boot";
import { detectLocale } from "./locale-detection";
import { LOCALE_STORAGE_KEY } from "./locale-storage";

const THROWS = Symbol("throws");

const cases: { stored: string | null | typeof THROWS; languages: string[] }[] =
  [
    { stored: null, languages: ["tr-TR", "en"] },
    { stored: null, languages: ["en-US", "tr"] },
    { stored: null, languages: ["de-DE"] },
    { stored: null, languages: ["de", "tr"] },
    { stored: null, languages: [] },
    { stored: "tr", languages: ["en"] },
    { stored: "en", languages: ["tr-TR"] },
    { stored: "fr", languages: ["en"] },
    { stored: "", languages: ["TR_tr"] },
    { stored: THROWS, languages: ["tr-TR"] },
    { stored: THROWS, languages: ["de"] },
  ];

function runBoot(
  stored: string | null | typeof THROWS,
  languages: string[],
): string {
  const documentStub = { documentElement: { lang: "" } };
  const storageStub = {
    getItem(key: string) {
      if (stored === THROWS) throw new DOMException("denied", "SecurityError");
      return key === LOCALE_STORAGE_KEY ? stored : null;
    },
  };
  const navigatorStub = { languages };
  new Function(
    "window",
    "document",
    "navigator",
    "localStorage",
    localeBootScript,
  )(
    {
      document: documentStub,
      navigator: navigatorStub,
      localStorage: storageStub,
    },
    documentStub,
    navigatorStub,
    storageStub,
  );
  return documentStub.documentElement.lang;
}

describe("locale boot script", () => {
  for (const { stored, languages } of cases) {
    const label =
      stored === THROWS ? "throwing storage" : JSON.stringify(stored);
    test(`stored=${label} languages=${JSON.stringify(languages)} matches detectLocale`, () => {
      const expected = detectLocale(
        stored === THROWS ? null : stored,
        languages,
      );
      expect(runBoot(stored, languages)).toBe(expected);
    });
  }

  test("is a self-contained IIFE without imports", () => {
    expect(localeBootScript.trim().startsWith("(function")).toBe(true);
    expect(localeBootScript).not.toMatch(/\bimport\b|\brequire\(/);
    expect(localeBootScript).toContain(JSON.stringify(LOCALE_STORAGE_KEY));
  });
});
