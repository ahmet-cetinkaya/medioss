// SC-019 text sweep: interface strings must be in the active language and
// never expose internal identifiers. The catalogs come in as plain string
// sets, so callers (T25) can feed the message catalogs in later.
import type { Page } from "@playwright/test";

export type TextSweepInput = {
  /** Every message value of the active language. */
  active: string[];
  /** Every message value of the other language. */
  other: string[];
  /** Seeded content, built-in sound names, and proper names (never hits). */
  exclusions: string[];
};

export type SweptValue = { value: string; source: string };
export type TextSweepHit = SweptValue & {
  reason: "other-language" | "id-like";
};

export const ID_LIKE = /^[a-z]+(\.[a-zA-Z0-9]+)+$/;

const PLACEHOLDER = /\{[^}]+\}/;

const normalize = (value: string): string => value.replace(/\s+/g, " ").trim();

const escapeRegExp = (value: string): string =>
  value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** Exact match for plain messages; `{param}` templates match any filled-in value. */
function matcher(values: string[]): (value: string) => boolean {
  const exact = new Set<string>();
  const patterns: RegExp[] = [];
  for (const value of values.map(normalize).filter(Boolean)) {
    if (PLACEHOLDER.test(value)) {
      patterns.push(
        new RegExp(
          `^${value
            .split(/\{[^}]+\}/)
            .map(escapeRegExp)
            .join(".+")}$`,
          "s",
        ),
      );
    } else {
      exact.add(value);
    }
  }
  return (value) =>
    exact.has(value) || patterns.some((pattern) => pattern.test(value));
}

/** Visible text nodes, aria-label/title/placeholder/alt of rendered elements, and document.title. */
export async function collectText(page: Page): Promise<SweptValue[]> {
  return page.evaluate(() => {
    const out: { value: string; source: string }[] = [];
    const rendered = (element: Element) =>
      element.checkVisibility({
        visibilityProperty: true,
      } as CheckVisibilityOptions);
    const walker = document.createTreeWalker(
      document.body,
      NodeFilter.SHOW_TEXT,
    );
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const parent = node.parentElement;
      const value = node.textContent?.replace(/\s+/g, " ").trim();
      if (
        !value ||
        !parent ||
        parent.closest("script, style, noscript, template")
      )
        continue;
      if (rendered(parent))
        out.push({ value, source: `text <${parent.tagName.toLowerCase()}>` });
    }
    for (const attribute of ["aria-label", "title", "placeholder", "alt"]) {
      for (const element of document.body.querySelectorAll(`[${attribute}]`)) {
        const value = element
          .getAttribute(attribute)
          ?.replace(/\s+/g, " ")
          .trim();
        if (value && rendered(element))
          out.push({
            value,
            source: `@${attribute} <${element.tagName.toLowerCase()}>`,
          });
      }
    }
    if (document.title.trim())
      out.push({ value: document.title.trim(), source: "document.title" });
    return out;
  });
}

/** Pure check: a value is a hit when it is other-language-only or id-like, unless excluded. */
export function findTextHits(
  values: SweptValue[],
  input: TextSweepInput,
): TextSweepHit[] {
  const activeSet = new Set(input.active.map(normalize));
  const isOther = matcher(
    input.other.filter((value) => !activeSet.has(normalize(value))),
  );
  const isActive = matcher(input.active);
  const excluded = new Set(input.exclusions.map(normalize));
  const hits: TextSweepHit[] = [];
  for (const swept of values) {
    const value = normalize(swept.value);
    if (!value || excluded.has(value)) continue;
    if (ID_LIKE.test(value)) hits.push({ ...swept, value, reason: "id-like" });
    else if (isOther(value) && !isActive(value))
      hits.push({ ...swept, value, reason: "other-language" });
  }
  return hits;
}

export async function sweepText(
  page: Page,
  input: TextSweepInput,
): Promise<TextSweepHit[]> {
  return findTextHits(await collectText(page), input);
}
