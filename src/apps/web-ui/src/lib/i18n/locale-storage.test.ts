import { describe, expect, spyOn, test } from "bun:test";
import {
  LOCALE_STORAGE_KEY,
  readStoredLocale,
  writeStoredLocale,
  type LocaleStorage,
} from "./locale-storage";

function memoryStorage(): LocaleStorage & {
  data: Map<string, string>;
  writes: number;
} {
  const data = new Map<string, string>();
  return {
    data,
    writes: 0,
    getItem: (key) => data.get(key) ?? null,
    setItem(key, value) {
      this.writes++;
      data.set(key, value);
    },
  };
}

const throwingStorage: LocaleStorage = {
  getItem() {
    throw new DOMException("denied", "SecurityError");
  },
  setItem() {
    throw new DOMException("quota", "QuotaExceededError");
  },
};

describe("locale storage", () => {
  test("round-trips under the medioss.locale key", () => {
    const storage = memoryStorage();
    writeStoredLocale("tr", storage);
    expect(storage.data.get(LOCALE_STORAGE_KEY)).toBe("tr");
    expect(readStoredLocale(storage)).toBe("tr");
  });

  test("returns null when nothing is stored", () => {
    expect(readStoredLocale(memoryStorage())).toBeNull();
  });

  test("a throwing storage reads null, swallows the write, and reports nothing", () => {
    const error = spyOn(console, "error");
    const warn = spyOn(console, "warn");
    expect(readStoredLocale(throwingStorage)).toBeNull();
    expect(() => writeStoredLocale("tr", throwingStorage)).not.toThrow();
    expect(error).not.toHaveBeenCalled();
    expect(warn).not.toHaveBeenCalled();
    error.mockRestore();
    warn.mockRestore();
  });

  test("works without any storage (non-browser runtime)", () => {
    expect(readStoredLocale()).toBeNull();
    expect(() => writeStoredLocale("en")).not.toThrow();
  });

  test("reading never writes", () => {
    const storage = memoryStorage();
    readStoredLocale(storage);
    expect(storage.writes).toBe(0);
  });

  test("only locale.set() writes the stored locale (FR-115)", async () => {
    const root = `${import.meta.dir}/../../..`;
    const writers: string[] = [];
    for await (const file of new Bun.Glob("src/**/*.{ts,svelte,astro}").scan({
      cwd: root,
    })) {
      if (/\.test\.ts$/.test(file) || file.endsWith("i18n/locale-storage.ts"))
        continue;
      const source = await Bun.file(`${root}/${file}`).text();
      if (
        /writeStoredLocale\s*\(|setItem\s*\(\s*(["'`]medioss\.locale|LOCALE_STORAGE_KEY)/.test(
          source,
        )
      )
        writers.push(file);
    }
    expect(writers).toEqual(["src/lib/i18n/locale.svelte.ts"]);

    const localeModule = await Bun.file(
      `${root}/src/lib/i18n/locale.svelte.ts`,
    ).text();
    expect(localeModule.match(/writeStoredLocale\s*\(/g)).toHaveLength(1);
    const setBody = localeModule.slice(localeModule.indexOf("set = "));
    expect(setBody).toMatch(/writeStoredLocale\s*\(/);
  });
});
