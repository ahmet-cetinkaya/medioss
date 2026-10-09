// T25: reduced motion and reduced transparency (plan T25; SC-015, FR-093,
// FR-094). With prefers-reduced-motion and prefers-reduced-transparency both
// on: every element's transition and animation durations are zero (global.css
// pins them to 0.01ms so transitionend still fires), hovering sampled
// controls does not move them, and the glass surfaces (shell, panel, solid,
// dialogs, bottom bar, chips) are opaque with no backdrop-filter.
import type { Page } from "@playwright/test";
import { storageStatePath, type Role } from "../support/auth";
import { emulateReducedTransparency } from "../support/reduced-transparency";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

/** global.css writes 0.01ms (1e-5 s) so transitionend/animationend still fire. */
const MAX_DURATION_S = 0.00001 + 1e-9;
const HOVER_SAMPLE = 60;

type Screen = { key: string; path: string };

function screens(role: Role): Screen[] {
  const seed = readSeed();
  if (role === "admin") return [{ key: "admin", path: "/admin/" }];
  return [
    { key: "home", path: "/" },
    { key: "library", path: "/library/" },
    { key: "program", path: `/program/?id=${seed.programs.withCover.id}` },
    {
      key: "player",
      path: `/player/?id=${seed.programs.withCover.meditations[0].id}`,
    },
    { key: "soundscape", path: "/soundscape/" },
    { key: "mood", path: "/mood/" },
    { key: "history", path: "/history/" },
    { key: "stats", path: "/stats/" },
    { key: "settings", path: "/settings/" },
  ];
}

async function openScreen(page: Page, screen: Screen) {
  await page.goto(screen.path, { waitUntil: "commit" });
  await expect(page.locator("nav:visible")).toHaveCount(1);
  await expect(page.locator("h1").first()).toBeVisible();
  await page.waitForLoadState("networkidle", { timeout: 8000 }).catch(() => {});
  await page.waitForTimeout(250);
}

/** Every element whose computed transition or animation is not zero. */
function motionOffenders(page: Page) {
  return page.evaluate((max) => {
    const seconds = (list: string) =>
      list.split(",").map((part) => parseFloat(part));
    const bad: string[] = [];
    for (const el of document.querySelectorAll("*")) {
      const style = getComputedStyle(el);
      const durations = [
        ...seconds(style.transitionDuration),
        ...seconds(style.animationDuration),
      ];
      const fast = durations.every((d) => !(d > max));
      const iterations = style.animationIterationCount
        .split(",")
        .every((n) => parseFloat(n) <= 1);
      if (!fast || !iterations)
        bad.push(
          `<${el.tagName.toLowerCase()} class="${String(el.className).slice(0, 70)}"> transition=${style.transitionDuration} animation=${style.animationDuration}/${style.animationIterationCount}`,
        );
    }
    return bad;
  }, MAX_DURATION_S);
}

/** Hovers up to HOVER_SAMPLE visible interactive elements spread across the
 *  page and returns those whose transform or box moved. */
async function hoverMovers(page: Page): Promise<string[]> {
  const handles = await page
    .locator('a[href], button, label:has(input), [role="button"]')
    .elementHandles();
  const visible: typeof handles = [];
  for (const handle of handles) {
    if (
      await handle.evaluate((el) =>
        (el as HTMLElement).checkVisibility({ visibilityProperty: true }),
      )
    )
      visible.push(handle);
  }
  const step = Math.max(1, Math.ceil(visible.length / HOVER_SAMPLE));
  const sample = visible
    .filter((_, i) => i % step === 0)
    .slice(0, HOVER_SAMPLE);
  const moved: string[] = [];
  // Transform and size only: scrolling into view legitimately moves x/y.
  const probe = (el: Element) => {
    const r = el.getBoundingClientRect();
    return `${getComputedStyle(el).transform}|${r.width}|${r.height}`;
  };
  for (const handle of sample) {
    await handle.scrollIntoViewIfNeeded();
    const before = await handle.evaluate(probe);
    await handle.hover({ timeout: 2000 }).catch(() => {});
    await page.waitForTimeout(30);
    const after = await handle.evaluate(probe);
    if (before !== after)
      moved.push(
        `${await handle.evaluate((el) => `<${el.tagName.toLowerCase()}> ${(el.getAttribute("aria-label") || el.textContent || "").trim().slice(0, 30)}`)}: ${before} -> ${after}`,
      );
  }
  await page.mouse.move(0, 0);
  return moved;
}

type Surface = { kind: string; alpha: number; backdrop: string; label: string };

/** Alpha and backdrop-filter of the sampled surfaces. Chips are the radio
 *  labels of a ChipGroup; only a selected chip has a fill. */
function surfaces(page: Page): Promise<Surface[]> {
  return page.evaluate(() => {
    const alphaOf = (color: string) => {
      const slash = color.match(/\/\s*([\d.]+)(%?)\s*\)/);
      if (slash) return parseFloat(slash[1]) / (slash[2] ? 100 : 1);
      const rgba = color.match(/^rgba\((.+)\)$/);
      if (rgba) return parseFloat(rgba[1].split(",")[3] ?? "1");
      return color === "transparent" ? 0 : 1;
    };
    const groups: [string, string][] = [
      ["glass-shell", ".glass-shell"],
      ["glass-panel", ".glass-panel"],
      ["glass-solid", ".glass-solid"],
      [
        "dialog",
        '[data-slot="dialog-content"], [data-slot="alert-dialog-content"], [data-slot="sheet-content"]',
      ],
      ["bottom-bar", 'nav[aria-label="Bottom navigation"]'],
      ["chip", "label:has(input[type=radio])"],
    ];
    const out: {
      kind: string;
      alpha: number;
      backdrop: string;
      label: string;
    }[] = [];
    for (const [kind, selector] of groups) {
      for (const el of Array.from(document.querySelectorAll(selector)).slice(
        0,
        12,
      )) {
        if (!(el as HTMLElement).checkVisibility()) continue;
        const style = getComputedStyle(el);
        const filled = kind !== "chip" || alphaOf(style.backgroundColor) > 0;
        if (!filled) {
          out.push({
            kind,
            alpha: 1,
            backdrop: style.backdropFilter,
            label: "unselected chip",
          });
          continue;
        }
        out.push({
          kind,
          alpha: alphaOf(style.backgroundColor),
          backdrop: style.backdropFilter,
          label: `${el.tagName.toLowerCase()} ${String(el.className).slice(0, 50)} bg=${style.backgroundColor}`,
        });
      }
    }
    return out;
  });
}

function expectOpaque(found: Surface[], where: string) {
  for (const s of found) {
    expect.soft(s.alpha, `${where}: ${s.kind} alpha (${s.label})`).toBe(1);
    expect
      .soft(s.backdrop, `${where}: ${s.kind} backdrop-filter (${s.label})`)
      .toBe("none");
  }
}

for (const role of ["member", "admin"] as const) {
  test.describe(`${role}, reduced motion and transparency`, () => {
    test.use({ storageState: storageStatePath(role) });

    for (const width of [375, 1280] as const) {
      test(`at ${width}px: durations are zero, hover moves nothing, surfaces are opaque (SC-015)`, async ({
        page,
      }) => {
        test.setTimeout(5 * 60_000);
        await page.setViewportSize(viewport(width));
        await emulateReducedTransparency(page, {
          transparency: "reduce",
          reducedMotion: "reduce",
        });
        expect(
          await page.evaluate(
            () =>
              matchMedia("(prefers-reduced-motion: reduce)").matches &&
              matchMedia("(prefers-reduced-transparency: reduce)").matches,
          ),
        ).toBe(true);

        for (const screen of screens(role)) {
          await openScreen(page, screen);
          const where = `${screen.key}@${width}`;
          expect(await motionOffenders(page), `${where}: motion`).toEqual([]);
          expect(await hoverMovers(page), `${where}: hover`).toEqual([]);
          const found = await surfaces(page);
          expect(
            found.filter((s) => s.kind === "glass-shell").length,
            `${where}: glass-shell sampled`,
          ).toBeGreaterThan(0);
          if (width < 768)
            expect(
              found.filter((s) => s.kind === "bottom-bar").length,
              `${where}: bottom bar sampled`,
            ).toBe(1);
          expectOpaque(found, where);
        }
      });
    }

    if (role === "member")
      test("dialogs, the More sheet and chips are opaque too (FR-094)", async ({
        page,
      }) => {
        test.setTimeout(3 * 60_000);
        await page.setViewportSize(viewport(375));
        await emulateReducedTransparency(page, {
          transparency: "reduce",
          reducedMotion: "reduce",
        });
        // Chips: Mood score selector and Stats / History range chips.
        for (const screen of [
          { key: "mood", path: "/mood/" },
          { key: "history", path: "/history/" },
          { key: "stats", path: "/stats/" },
        ]) {
          await openScreen(page, screen);
          const found = await surfaces(page);
          expect(
            found.filter((s) => s.kind === "chip").length,
            `${screen.key}: chips sampled`,
          ).toBeGreaterThan(0);
          expectOpaque(found, `${screen.key} chips`);
        }

        await page
          .getByRole("navigation", { name: "Bottom navigation" })
          .getByRole("button", { name: "More" })
          .click();
        const sheet = page.getByRole("dialog", { name: "More" });
        await expect(sheet).toBeVisible();
        await page.waitForTimeout(150);
        expect(await motionOffenders(page), "more sheet motion").toEqual([]);
        let found = await surfaces(page);
        expect(found.filter((s) => s.kind === "dialog").length).toBeGreaterThan(
          0,
        );
        expectOpaque(found, "more sheet");

        await sheet.getByRole("button", { name: "Sign out" }).click();
        await expect(
          page.getByRole("alertdialog", { name: "Sign out?" }),
        ).toBeVisible();
        await page.waitForTimeout(150);
        expect(await motionOffenders(page), "sign-out motion").toEqual([]);
        found = await surfaces(page);
        expect(found.filter((s) => s.kind === "dialog").length).toBeGreaterThan(
          0,
        );
        expectOpaque(found, "sign-out dialog");
        // The scrim keeps its tint (it is not a surface) but never blurs.
        const overlays = await page
          .locator('[data-slot$="dialog-overlay"]')
          .evaluateAll((els) =>
            els.map((el) => getComputedStyle(el).backdropFilter),
          );
        expect(overlays.length).toBeGreaterThan(0);
        for (const backdrop of overlays) expect(backdrop).toBe("none");
      });
  });
}

test.describe("control: the checks can fail", () => {
  test.use({ storageState: storageStatePath("member") });

  test("without the preferences the shell is translucent and blurred", async ({
    page,
  }) => {
    await page.setViewportSize(viewport(1280));
    await openScreen(page, { key: "home", path: "/" });
    const shell = (await surfaces(page)).find((s) => s.kind === "glass-shell");
    expect(shell!.alpha).toBeLessThan(1);
    expect(shell!.backdrop).not.toBe("none");
  });
});
