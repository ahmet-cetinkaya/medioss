// Static half of SC-019: every `*.messages.ts` catalog under src/ is complete
// in English and Turkish. New catalogs are picked up automatically.

import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "../../..");
const ID_LIKE = /^[a-z]+(\.[a-zA-Z0-9]+)+$/;
const PLACEHOLDER = /\{(\w+)\}/g;

type Entry = { path: string; en: unknown; tr: unknown };

function collect(value: unknown, path: string, out: Entry[]): void {
  if (!value || typeof value !== "object") return;
  if ("en" in value) {
    const message = value as { en: unknown; tr?: unknown };
    out.push({ path, en: message.en, tr: message.tr });
    return;
  }
  for (const [key, child] of Object.entries(value))
    collect(child, `${path}.${key}`, out);
}

const placeholders = (text: string): string[] =>
  [...new Set([...text.matchAll(PLACEHOLDER)].map((m) => m[1]))].sort();

// Every parameter reads as 1, so plural and suffix functions can be sampled.
const anyParams = new Proxy({}, { get: () => 1, has: () => true });

const files = [
  ...new Bun.Glob("src/**/*.messages.ts").scanSync({ cwd: root }),
].sort();

describe("message catalog parity", () => {
  test("finds the shared catalog", () => {
    expect(files).toContain("src/lib/i18n/common.messages.ts");
  });

  for (const file of files) {
    test(file, async () => {
      const entries: Entry[] = [];
      for (const [name, value] of Object.entries(
        await import(resolve(root, file)),
      ))
        collect(value, name, entries);
      expect(entries.length).toBeGreaterThan(0);

      for (const { path, en, tr } of entries) {
        const where = `${file} ${path}`;
        expect(typeof en, where).toBe(typeof tr);
        expect(["string", "function"], where).toContain(typeof en);
        if (typeof en === "string" && typeof tr === "string") {
          expect(en.trim(), where).not.toBe("");
          expect(tr.trim(), where).not.toBe("");
          expect(placeholders(tr), where).toEqual(placeholders(en));
          expect(en, where).not.toMatch(ID_LIKE);
          expect(tr, where).not.toMatch(ID_LIKE);
        }
        if (typeof en === "function" && typeof tr === "function") {
          for (const text of [en(anyParams), tr(anyParams)]) {
            expect(typeof text, where).toBe("string");
            expect(text.trim(), where).not.toBe("");
            expect(text, where).not.toMatch(ID_LIKE);
          }
        }
      }
    });
  }
});
