// T25: responsive layout (plan T25; SC-014, FR-085, FR-088, FR-089, FR-095,
// FR-099). At 320/375/768/1024/1280/1440 px every authenticated screen has no
// horizontal scroll and shows the one nav surface of its tier (sidebar >= 1280,
// icon rail 768–1279, bottom bar < 768). At 320 px every visible interactive
// element is at least 44x44, 200% text keeps the layout intact, and a 64-char
// display name wraps. At 375 px the fixed bottom bar never covers a focused
// element. Evidence screenshots: .agents/evidence/ui-ux-design/T25/responsive/.
import path from "node:path";
import type { Page } from "@playwright/test";
import { expectOk, getJson, postJson } from "../support/api-client";
import {
  loginApi,
  loginAs,
  storageStatePath,
  type Role,
  type UserDto,
} from "../support/auth";
import { evidenceDir } from "../support/evidence";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { viewport, type ViewportWidth } from "../support/viewport";

const WIDTHS = [
  320, 375, 768, 1024, 1280, 1440,
] as const satisfies readonly ViewportWidth[];

const SURFACE_LABEL = (width: number) =>
  width >= 1280
    ? "Main navigation"
    : width >= 768
      ? "Compact navigation"
      : "Bottom navigation";
const ALL_SURFACES = [
  "Main navigation",
  "Compact navigation",
  "Bottom navigation",
];

type Screen = { key: string; path: string };

function screens(role: Role): Screen[] {
  const seed = readSeed();
  if (role === "admin") return [{ key: "admin", path: "/admin/" }];
  return [
    { key: "home", path: "/" },
    { key: "library", path: "/library/" },
    {
      key: "program",
      path: `/program/?id=${seed.programs.withCover.id}`,
    },
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

const shot = async (page: Page, width: number, name: string) => {
  const dir = path.join(evidenceDir("T25"), "responsive");
  await page.screenshot({
    path: path.join(dir, `${name}-${width}.png`),
    fullPage: true,
  });
};

/** Opens a screen and waits until the shell and its content have settled. */
async function openScreen(page: Page, screen: Screen) {
  await page.goto(screen.path, { waitUntil: "commit" });
  await expect(page.locator("nav:visible")).toHaveCount(1);
  await expect(page.locator("main h1, h1").first()).toBeVisible();
  await page.waitForLoadState("networkidle", { timeout: 8000 }).catch(() => {});
  await page.waitForTimeout(250);
}

const noHorizontalScroll = (page: Page) =>
  page.evaluate(() => ({
    doc:
      document.documentElement.scrollWidth -
      document.documentElement.clientWidth,
    body: document.body.scrollWidth - document.documentElement.clientWidth,
  }));

/** `tolerance` is in CSS px: scrollWidth/clientWidth are integer-rounded, so at
 *  200% text (fractional rem widths) a sub-pixel overhang reads as exactly 1. */
async function expectNoHorizontalScroll(
  page: Page,
  label: string,
  tolerance = 0,
) {
  const { doc, body } = await noHorizontalScroll(page);
  expect(
    doc,
    `${label}: documentElement scrollWidth > clientWidth`,
  ).toBeLessThanOrEqual(tolerance);
  expect(body, `${label}: body scrollWidth > clientWidth`).toBeLessThanOrEqual(
    tolerance,
  );
}

/** Interactive elements that are visible, not aria-hidden and not decorative,
 *  with a rect smaller than 44x44. Visually hidden form inputs defer to their
 *  wrapping label, which is measured instead. */
function undersizedTargets(page: Page) {
  return page.evaluate(() => {
    const selector = [
      "a[href]",
      "button",
      "select",
      "textarea",
      "summary",
      "input:not([type=hidden])",
      "label:has(input)",
      '[role="button"]',
      '[role="link"]',
      '[role="tab"]',
      '[role="checkbox"]',
      '[role="radio"]',
      '[role="switch"]',
      '[role="slider"]',
      '[role="menuitem"]',
      '[tabindex]:not([tabindex="-1"])',
    ].join(",");
    const bad: string[] = [];
    for (const el of document.querySelectorAll<HTMLElement>(selector)) {
      if (el.closest('[aria-hidden="true"], [inert]')) continue;
      if (
        !el.checkVisibility({ visibilityProperty: true, opacityProperty: true })
      )
        continue;
      const style = getComputedStyle(el);
      if (style.pointerEvents === "none") continue;
      const rect = el.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) continue;
      // Visually hidden (sr-only) controls, e.g. the skip link until focused.
      if (el.classList.contains("sr-only")) continue;
      // Visually hidden (sr-only) controls and skip links parked off-screen.
      if (rect.width <= 1 || rect.height <= 1) continue;
      if (
        el.closest("label")?.querySelector("input") === el &&
        el !== el.closest("label")
      ) {
        const labelRect = el.closest("label")!.getBoundingClientRect();
        if (labelRect.width >= 44 && labelRect.height >= 44) continue;
      }
      if (el.matches("input") && el.closest("label") && rect.width <= 1)
        continue;
      if (rect.width < 43.5 || rect.height < 43.5) {
        const name =
          el.getAttribute("aria-label") ||
          el.textContent?.trim().slice(0, 40) ||
          "";
        bad.push(
          `<${el.tagName.toLowerCase()} class="${el.className.toString().slice(0, 80)}"> "${name}" ${Math.round(rect.width)}x${Math.round(rect.height)}`,
        );
      }
    }
    return bad;
  });
}

for (const role of ["member", "admin"] as const) {
  test.describe(`${role} screens`, () => {
    test.use({ storageState: storageStatePath(role) });

    for (const width of WIDTHS) {
      test(`at ${width}px: no horizontal scroll and the ${SURFACE_LABEL(width)} surface only (SC-014, FR-085)`, async ({
        page,
      }) => {
        test.setTimeout(5 * 60_000);
        await page.setViewportSize(viewport(width));
        for (const screen of screens(role)) {
          await openScreen(page, screen);
          await expectNoHorizontalScroll(page, `${screen.key}@${width}`);
          const visible = page.getByRole("navigation", {
            name: SURFACE_LABEL(width),
          });
          await expect(visible).toBeVisible();
          for (const name of ALL_SURFACES.filter(
            (n) => n !== SURFACE_LABEL(width),
          ))
            await expect(
              page.getByRole("navigation", { name, includeHidden: true }),
            ).toBeHidden();
          await shot(page, width, screen.key);
        }
      });
    }

    test("at 320px every visible interactive element is >= 44x44 (FR-088)", async ({
      page,
    }) => {
      test.setTimeout(5 * 60_000);
      await page.setViewportSize(viewport(320));
      const failures: string[] = [];
      for (const screen of screens(role)) {
        await openScreen(page, screen);
        for (const bad of await undersizedTargets(page))
          failures.push(`${screen.key}: ${bad}`);
      }
      expect(failures).toEqual([]);
    });

    test("at 320px with 200% text: no horizontal scroll, nav and heading still visible (FR-089)", async ({
      page,
    }) => {
      test.setTimeout(5 * 60_000);
      await page.setViewportSize(viewport(320));
      for (const screen of screens(role)) {
        await openScreen(page, screen);
        await page.addStyleTag({ content: "html{font-size:200%}" });
        await page.waitForTimeout(250);
        // 1px: rounding artifact of fractional widths at 200% font size; a real
        // overflow (10px+ before the fix) still fails.
        await expectNoHorizontalScroll(page, `${screen.key}@320/200%`, 1);
        await expect(page.locator("h1").first()).toBeVisible();
        const bar = page.getByRole("navigation", { name: "Bottom navigation" });
        await expect(bar).toBeVisible();
        const items = bar.locator("a, button");
        const count = await items.count();
        expect(count).toBeGreaterThan(0);
        for (let i = 0; i < count; i++) {
          const box = await items.nth(i).boundingBox();
          expect(box, `${screen.key}: bar item ${i} has a box`).not.toBeNull();
          expect(box!.x).toBeGreaterThanOrEqual(0);
          expect(box!.x + box!.width).toBeLessThanOrEqual(320.5);
        }
        await shot(page, 320, `${screen.key}-font200`);
      }
    });
  });
}

test.describe("member chrome", () => {
  test.use({ storageState: storageStatePath("member") });

  for (const width of WIDTHS) {
    test(`at ${width}px: More sheet / Sign out dialog fit the viewport (FR-085)`, async ({
      page,
    }) => {
      await page.setViewportSize(viewport(width));
      await openScreen(page, { key: "stats", path: "/stats/" });
      if (width < 768) {
        await page
          .getByRole("navigation", { name: "Bottom navigation" })
          .getByRole("button", { name: "More" })
          .click();
        const sheet = page.getByRole("dialog", { name: "More" });
        await expect(sheet).toBeVisible();
        await page.waitForTimeout(300);
        await expectNoHorizontalScroll(page, `more@${width}`);
        await shot(page, width, "more-sheet");
        await sheet.getByRole("button", { name: "Sign out" }).click();
      } else {
        await page
          .getByRole("navigation", { name: SURFACE_LABEL(width) })
          .getByRole("button", { name: "Sign out" })
          .click();
      }
      const dialog = page.getByRole("alertdialog", { name: "Sign out?" });
      await expect(dialog).toBeVisible();
      await page.waitForTimeout(300);
      await expectNoHorizontalScroll(page, `signout@${width}`);
      const box = await dialog.boundingBox();
      expect(box!.x).toBeGreaterThanOrEqual(0);
      expect(box!.x + box!.width).toBeLessThanOrEqual(width + 0.5);
      await shot(page, width, "signout-dialog");
    });
  }

  test("at 375px focused elements are never hidden under the fixed bottom bar (FR-087)", async ({
    page,
  }) => {
    test.setTimeout(5 * 60_000);
    await page.setViewportSize(viewport(375));
    const hidden: string[] = [];
    for (const screen of screens("member")) {
      await openScreen(page, screen);
      await page.evaluate(() =>
        (document.activeElement as HTMLElement)?.blur(),
      );
      for (let i = 0; i < 45; i++) {
        await page.keyboard.press("Tab");
        const verdict = await page.evaluate(() => {
          const el = document.activeElement as HTMLElement | null;
          const bar = document.querySelector<HTMLElement>(
            'nav[aria-label="Bottom navigation"]',
          );
          if (!el || el === document.body || !bar) return null;
          if (bar.contains(el)) return "in-bar";
          const r = el.getBoundingClientRect();
          const b = bar.getBoundingClientRect();
          const covered = r.bottom > b.top && r.top < b.bottom;
          return covered
            ? `<${el.tagName.toLowerCase()}> "${(el.getAttribute("aria-label") || el.textContent || "").trim().slice(0, 30)}" rect ${Math.round(r.top)}-${Math.round(r.bottom)} bar top ${Math.round(b.top)}`
            : "ok";
        });
        if (verdict === "in-bar") break; // Tab order reached the bar: main is done
        if (verdict && verdict !== "ok")
          hidden.push(`${screen.key}: ${verdict}`);
      }
    }
    expect(hidden).toEqual([]);
  });
});

test.describe("64-character display name (FR-099)", () => {
  const email = `longname-${Date.now()}@example.test`;
  const password = "longname-e2e-pass-1";
  const spaced =
    "Aylin Nehir Zeynep Esra Melike Ozlem Cagla Deniz Hande Irem Sele"; // 64
  const unbroken = "W".repeat(64);
  let userId = 0;

  test.beforeAll(async () => {
    expect(spaced).toHaveLength(64);
    const admin = await loginAs("admin");
    const created = await postJson<UserDto>(admin, "/api/identity/users", {
      email,
      password: "longname-e2e-temp-1",
    });
    userId = created.id;
    await admin.dispose();
    const first = await loginApi(email, "longname-e2e-temp-1");
    await postJson(first, "/api/identity/me/password", { password });
    await first.dispose();
  });

  test.afterAll(async () => {
    const admin = await loginAs("admin");
    await admin.delete(`/api/identity/users/${userId}`);
    await admin.dispose();
  });

  test("wraps at 320px without horizontal scroll, on Home and Settings", async ({
    browser,
  }) => {
    test.setTimeout(3 * 60_000);
    const api = await loginApi(email, password);
    const state = await api.storageState();
    const context = await browser.newContext({
      storageState: state,
      viewport: viewport(320),
    });
    const page = await context.newPage();
    for (const name of [spaced, unbroken]) {
      await expectOk(
        await api.patch("/api/identity/me/profile", {
          data: { display_name: name },
        }),
        "set display name",
      );
      expect(
        (await getJson<UserDto>(api, "/api/identity/me")).display_name,
      ).toBe(name);

      await openScreen(page, { key: "home", path: "/" });
      const greeting = page.getByText(name).first();
      await expect(greeting).toBeVisible();
      await expectNoHorizontalScroll(
        page,
        `home long name ${name.slice(0, 6)}`,
      );
      const box = await greeting.boundingBox();
      expect(box!.x + box!.width).toBeLessThanOrEqual(320.5);
      if (name === spaced) {
        // Wrapped: taller than a single line of its own text.
        const lines = await greeting.evaluate((el) => {
          const lh = parseFloat(getComputedStyle(el).lineHeight) || 20;
          return el.getBoundingClientRect().height / lh;
        });
        expect(lines).toBeGreaterThan(1.5);
      }
      await shot(
        page,
        320,
        name === spaced ? "long-name-home" : "long-name-unbroken-home",
      );

      await openScreen(page, { key: "settings", path: "/settings/" });
      await expectNoHorizontalScroll(page, "settings long name");
    }
    await context.close();
    await api.dispose();
  });
});
