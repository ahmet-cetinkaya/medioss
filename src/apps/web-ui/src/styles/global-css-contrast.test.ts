// SC-013 / FR-090: every text and indicator token in global.css meets WCAG AA
// over the composited glass it actually sits on, sampled across the canvas
// gradient (worst case: the bottom-right lavender stop #c9aaf2).
//
// The parser reads the top-level `:root` blocks of global.css (never `.dark`
// or the indented fallback blocks) and understands these value forms:
// `#rgb`, `#rrggbb`, `rgb(r g b / a)` (a as 0–1 or %), `transparent`, and
// `var(--token)` references. Gradient tokens contribute every color they list.

import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

type Rgb = [number, number, number];
type Rgba = { rgb: Rgb; a: number };

const css = readFileSync(new URL("./global.css", import.meta.url), "utf8");

function rootTokens(source: string): Map<string, string> {
  const tokens = new Map<string, string>();
  const text = source.replace(/\/\*[\s\S]*?\*\//g, "");
  for (const block of text.matchAll(/^:root\s*\{([^}]*)\}/gm)) {
    for (const decl of block[1].split(";")) {
      const match = /^\s*(--[\w-]+)\s*:\s*([\s\S]+?)\s*$/.exec(decl);
      if (match) tokens.set(match[1], match[2].replace(/\s+/g, " "));
    }
  }
  return tokens;
}

const tokens = rootTokens(css);

function resolveToken(name: string, depth = 0): string {
  const value = tokens.get(name);
  if (value === undefined) throw new Error(`token ${name} not found in :root`);
  const ref = /^var\((--[\w-]+)\)$/.exec(value);
  if (ref && depth < 10) return resolveToken(ref[1], depth + 1);
  return value;
}

const COLOR = /#[0-9a-f]{3,6}\b|rgb\([^)]*\)|\btransparent\b/gi;

function parseColor(value: string): Rgba {
  const v = value.trim().toLowerCase();
  if (v === "transparent") return { rgb: [0, 0, 0], a: 0 };
  const hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/.exec(v);
  if (hex) {
    const h =
      hex[1].length === 3 ? [...hex[1]].map((c) => c + c).join("") : hex[1];
    return {
      rgb: [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16)) as Rgb,
      a: 1,
    };
  }
  const rgb =
    /^rgb\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)\s*(?:\/\s*([\d.]+)(%?))?\s*\)$/.exec(
      v,
    );
  if (rgb) {
    const alpha = rgb[4] === undefined ? 1 : Number(rgb[4]);
    return {
      rgb: [Number(rgb[1]), Number(rgb[2]), Number(rgb[3])],
      a: rgb[5] ? alpha / 100 : alpha,
    };
  }
  throw new Error(`unsupported color syntax: ${value}`);
}

const color = (name: string) => parseColor(resolveToken(name));
const gradientStops = (name: string) =>
  (resolveToken(name).match(COLOR) ?? []).map(parseColor);

// Source-over compositing in sRGB, as browsers paint translucent fills.
function over(top: Rgba, bottom: Rgb): Rgb {
  return top.rgb.map((c, i) => c * top.a + bottom[i] * (1 - top.a)) as Rgb;
}

function luminance([r, g, b]: Rgb): number {
  const lin = (c: number) => {
    const s = c / 255;
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

function contrast(a: Rgb, b: Rgb): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const opaque = (name: string): Rgb => {
  const c = color(name);
  if (c.a !== 1) throw new Error(`${name} must be opaque`);
  return c.rgb;
};

// Canvas samples: every stop plus 10 % steps between every pair of stops, so
// blends anywhere in the layered gradient are covered.
const canvasStops = gradientStops("--gradient-canvas").filter((c) => c.a > 0);
const canvasSamples: Rgb[] = [];
for (let i = 0; i < canvasStops.length; i++) {
  for (let j = i; j < canvasStops.length; j++) {
    for (let t = 0; t <= 10; t++) {
      canvasSamples.push(
        over({ rgb: canvasStops[j].rgb, a: t / 10 }, canvasStops[i].rgb),
      );
    }
  }
}

type Surface = { name: string; rgb: Rgb };

function surfaces(): Surface[] {
  const out: Surface[] = [];
  const shell = color("--surface-shell");
  const panel = color("--surface-panel");
  const tint = gradientStops("--gradient-panel-tint");
  for (const canvas of canvasSamples) {
    const tag = `canvas rgb(${canvas.map(Math.round).join(" ")})`;
    const onShell = over(shell, canvas);
    out.push({ name: `shell over ${tag}`, rgb: onShell });
    out.push({
      name: `panel over shell over ${tag}`,
      rgb: over(panel, onShell),
    });
    out.push({ name: `panel over ${tag}`, rgb: over(panel, canvas) });
    for (const stop of tint) {
      out.push({
        name: `panel tint over shell over ${tag}`,
        rgb: over(stop, onShell),
      });
    }
  }
  const solid = opaque("--surface-solid");
  out.push({ name: "surface-solid", rgb: solid });
  out.push({
    name: "surface-shell-opaque",
    rgb: opaque("--surface-shell-opaque"),
  });
  out.push({
    name: "surface-panel-opaque",
    rgb: opaque("--surface-panel-opaque"),
  });
  out.push({
    name: "accent-soft over surface-solid",
    rgb: over(color("--accent-soft"), solid),
  });
  // Image wash end over black: the darkest possible artwork under a title.
  const wash = gradientStops("--gradient-image-wash").at(-1)!;
  out.push({
    name: "image wash over black artwork",
    rgb: over(wash, [0, 0, 0]),
  });
  return out;
}

const allSurfaces = surfaces();

function worst(token: string): { ratio: number; on: string } {
  const fg = opaque(token);
  let result = { ratio: Infinity, on: "" };
  for (const s of allSurfaces) {
    const ratio = contrast(fg, s.rgb);
    if (ratio < result.ratio) result = { ratio, on: s.name };
  }
  return result;
}

function expectAtLeast(token: string, min: number) {
  const { ratio, on } = worst(token);
  if (process.env.CONTRAST_REPORT)
    console.log(`${token} ${ratio.toFixed(2)} on ${on}`);
  if (ratio < min)
    throw new Error(
      `${token} is ${ratio.toFixed(2)}:1 on ${on}, needs ${min}:1`,
    );
  expect(ratio).toBeGreaterThanOrEqual(min);
}

describe("global.css parser guard", () => {
  test("finds the tokens and the canvas stops", () => {
    for (const name of [
      "--text-strong",
      "--text-body",
      "--text-muted",
      "--text-faint",
      "--accent-strong",
      "--accent-fill",
      "--control-border",
      "--chart-mark",
      "--surface-shell",
      "--surface-panel",
      "--surface-solid",
    ]) {
      expect(() => color(name)).not.toThrow();
    }
    const hexes = canvasStops.map((c) => c.rgb.join(","));
    for (const stop of ["#ebc0a6", "#eaa8b8", "#c9aaf2", "#aacdf2"]) {
      expect(hexes).toContain(parseColor(stop).rgb.join(","));
    }
    expect(color("--surface-shell").a).toBeLessThan(1);
    expect(allSurfaces.length).toBeGreaterThan(100);
  });

  test("ignores the .dark block", () => {
    expect(resolveToken("--background")).not.toContain("oklch");
  });

  test("would reject the §3.1 reference --text-muted #7c7c7c", () => {
    const ratios = allSurfaces.map((s) =>
      contrast(parseColor("#7c7c7c").rgb, s.rgb),
    );
    expect(Math.min(...ratios)).toBeLessThan(4.5);
  });
});

describe("text contrast over composited glass", () => {
  for (const token of [
    "--text-strong",
    "--text-body",
    "--text-muted",
    "--accent-strong",
    "--destructive",
    "--success",
  ]) {
    test(`${token} is body text, >= 4.5:1`, () => expectAtLeast(token, 4.5));
  }

  test("--text-faint is large text and icons only, >= 3:1", () =>
    expectAtLeast("--text-faint", 3));
});

describe("non-text indicators, >= 3:1 against adjacent colors", () => {
  test("focus ring (--ring is --accent-strong)", () => {
    expect(resolveToken("--ring")).toBe(resolveToken("--accent-strong"));
    expectAtLeast("--accent-strong", 3);
  });

  test("control boundary (--control-border, shadcn --input)", () => {
    expect(resolveToken("--input")).toBe(resolveToken("--control-border"));
    expectAtLeast("--control-border", 3);
  });

  test("chart marks (--chart-mark)", () => expectAtLeast("--chart-mark", 3));
});

describe("mood scale (--mood-1..5, the score selector)", () => {
  const scores = [1, 2, 3, 4, 5] as const;
  /** The white ink the selected option flips to (score-selector.svelte). */
  const ink = opaque("--primary-foreground");

  test("finds all five tokens", () => {
    for (const score of scores) {
      expect(() => color(`--mood-${score}`)).not.toThrow();
    }
  });

  for (const score of scores) {
    test(`--mood-${score} is unselected score text, >= 4.5:1`, () =>
      expectAtLeast(`--mood-${score}`, 4.5));
  }

  // The selected option fills with the score color and flips its text to the
  // white ink: that pair must hold on its own, independent of the surface.
  for (const score of scores) {
    test(`white ink on the --mood-${score} fill, >= 4.5:1`, () => {
      const ratio = contrast(ink, opaque(`--mood-${score}`));
      if (process.env.CONTRAST_REPORT)
        console.log(
          `--primary-foreground on --mood-${score} ${ratio.toFixed(2)}`,
        );
      expect(ratio).toBeGreaterThanOrEqual(4.5);
    });
  }
});

describe("labels on fills, >= 4.5:1", () => {
  const pairs: [string, string][] = [
    ["--primary-foreground", "--primary"],
    ["--sidebar-primary-foreground", "--sidebar-primary"],
    ["--accent-foreground", "--accent"],
    ["--primary-foreground", "--destructive"],
    ["--primary-foreground", "--success"],
    ["--foreground", "--background"],
    ["--muted-foreground", "--background"],
  ];
  for (const [fg, bg] of pairs) {
    test(`${fg} on ${bg}`, () => {
      const ratio = contrast(opaque(fg), opaque(bg));
      if (process.env.CONTRAST_REPORT)
        console.log(`${fg} on ${bg} ${ratio.toFixed(2)}`);
      expect(ratio).toBeGreaterThanOrEqual(4.5);
    });
  }
});

describe("fallbacks (FR-093, FR-094)", () => {
  const body = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const opaqueOverride =
    /--surface-shell:\s*var\(--surface-shell-opaque\);\s*--surface-panel:\s*var\(--surface-panel-opaque\);/;

  test("reduced transparency makes glass opaque", () => {
    const block =
      /@media \(prefers-reduced-transparency: reduce\) \{([\s\S]*?)\n\}/.exec(
        body,
      );
    expect(block?.[1]).toMatch(opaqueOverride);
  });

  test("missing backdrop-filter makes glass opaque", () => {
    const block =
      /@supports not \(\s*\(backdrop-filter: blur\(1px\)\) or \(-webkit-backdrop-filter: blur\(1px\)\)\s*\) \{([\s\S]*?)\n\}/.exec(
        body,
      );
    expect(block?.[1]).toMatch(opaqueOverride);
  });

  test("reduced motion zeros transitions and animations", () => {
    const block =
      /@media \(prefers-reduced-motion: reduce\) \{([\s\S]*?)\n {2}\}/.exec(
        body,
      );
    expect(block?.[1]).toContain("transition-duration: 0.01ms !important");
    expect(block?.[1]).toContain("animation-duration: 0.01ms !important");
  });
});
