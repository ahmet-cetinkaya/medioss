// Static half of SC-019/FR-102: no Svelte component carries a hard-coded
// user-facing literal. Text nodes and aria-label/title/placeholder/alt
// attribute literals must come from the message catalogs; only proper names
// may appear verbatim. The shadcn primitives under components/ui are skipped.

import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "../../..");
const ALLOWED = new Set(["MediOSS", "English", "Türkçe"]);
const ATTRIBUTES = new Set(["aria-label", "title", "placeholder", "alt"]);
const HAS_LETTER = /\p{L}/u;

type Literal = { line: number; where: string; value: string };

/** Blanks `<script>`, `<style>`, and HTML comments; newlines stay so line numbers hold. */
const blank = (source: string): string =>
  source.replace(
    /<!--[\s\S]*?-->|<script\b[\s\S]*?<\/script>|<style\b[\s\S]*?<\/style>/g,
    (match) => match.replace(/[^\n]/g, " "),
  );

/** Index just past the `{...}` expression opening at `from` (strings and templates aware). */
function skipBraces(text: string, from: number): number {
  let depth = 0;
  for (let i = from; i < text.length; i++) {
    const char = text[i];
    if (char === '"' || char === "'" || char === "`") {
      for (i++; i < text.length && text[i] !== char; i++)
        if (text[i] === "\\") i++;
    } else if (char === "{") depth++;
    else if (char === "}" && --depth === 0) return i + 1;
  }
  return text.length;
}

/** Index just past the tag opening at `from`; collects its literal attributes. */
function readTag(
  text: string,
  from: number,
  found: { offset: number; where: string; value: string }[],
): number {
  let i = from + 1;
  while (i < text.length && text[i] !== ">") {
    if (text[i] === "{") {
      i = skipBraces(text, i);
      continue;
    }
    const name = /^[^\s=>/{}"']+/.exec(text.slice(i))?.[0];
    if (!name) {
      i++;
      continue;
    }
    i += name.length;
    const rest = /^\s*=\s*/.exec(text.slice(i));
    if (!rest) continue;
    i += rest[0].length;
    const quote = text[i];
    if (quote === '"' || quote === "'") {
      const end = text.indexOf(quote, i + 1);
      const value = text.slice(i + 1, end);
      // A value with `{...}` mixes expressions and literal text.
      const literal = value.replace(/\{[^}]*\}/g, " ");
      if (ATTRIBUTES.has(name) && HAS_LETTER.test(literal))
        found.push({ offset: i, where: `${name}=`, value: value.trim() });
      i = end + 1;
    } else if (quote === "{") i = skipBraces(text, i);
    else i += /^[^\s>]*/.exec(text.slice(i))![0].length;
  }
  return i + 1;
}

function literals(source: string): Literal[] {
  const text = blank(source);
  const found: { offset: number; where: string; value: string }[] = [];
  // Text between tags, with `{...}` expressions and `{#if}` / `{/if}` markers cut out.
  let i = 0;
  let textStart = 0;
  let buffer = "";
  let firstLetter = -1;
  const take = (end: number) => {
    const chunk = text.slice(textStart, end).replace(/&[#\w]+;/g, " ");
    const at = chunk.search(HAS_LETTER);
    if (at >= 0 && firstLetter < 0) firstLetter = textStart + at;
    buffer += chunk;
  };
  while (i < text.length) {
    if (text[i] === "{") {
      take(i);
      i = skipBraces(text, i);
      textStart = i;
    } else if (text[i] === "<" && /[A-Za-z/]/.test(text[i + 1] ?? "")) {
      take(i);
      if (firstLetter >= 0)
        found.push({
          offset: firstLetter,
          where: "text",
          value: buffer.trim(),
        });
      buffer = "";
      firstLetter = -1;
      i = readTag(text, i, found);
      textStart = i;
    } else i++;
  }
  return found
    .filter(({ value }) => !ALLOWED.has(value.replace(/\s+/g, " ")))
    .map(({ offset, where, value }) => ({
      line: source.slice(0, offset).split("\n").length,
      where,
      value,
    }));
}

const files = [...new Bun.Glob("src/**/*.svelte").scanSync({ cwd: root })]
  .filter((file) => !file.startsWith("src/lib/components/ui/"))
  .sort();

describe("literal string guard", () => {
  test("scans the screens, shell, and shared components", () => {
    expect(files.length).toBeGreaterThan(40);
    expect(files).toContain("src/lib/components/shell/sign-out.svelte");
  });

  test("detects literals and ignores expressions, comments, and proper names", () => {
    const sample = `<script lang="ts">const a = "Hello";</script>
<!-- Comment words -->
<p>Hello</p>
<p>{t(m.a)}</p>
<p>MediOSS</p>
<button aria-label="Close" title={t(m.b)}>{#if x}{t(m.c)}{/if}</button>
<img alt="" src="a.png" />
<input placeholder="Search" />`;
    expect(literals(sample).map(({ line, value }) => [line, value])).toEqual([
      [3, "Hello"],
      [6, "Close"],
      [8, "Search"],
    ]);
  });

  for (const file of files) {
    test(file, async () => {
      const violations = literals(await Bun.file(resolve(root, file)).text());
      expect(
        violations.map((v) => `${file}:${v.line} ${v.where} "${v.value}"`),
      ).toEqual([]);
    });
  }
});
