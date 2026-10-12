// T25: loading under a 3 s delay (SC-006). Every authenticated screen is
// sampled every 250 ms while its API calls are held: at each sample the page
// shows a skeleton (the static shell skeleton or an aria-busy region), and
// every main data region is either busy or carries text. Never a blank frame.
import type { Page } from "@playwright/test";
import { storageStatePath, type Role } from "../support/auth";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";

const DELAY_MS = 3_000;
const SAMPLE_MS = 250;

type Screen = { key: string; role: Role; path: () => string };

const screens: readonly Screen[] = [
  { key: "home", role: "member", path: () => "/" },
  { key: "library", role: "member", path: () => "/library/" },
  {
    key: "program",
    role: "member",
    path: () => `/program/?id=${readSeed().programs.withCover.id}`,
  },
  {
    key: "player",
    role: "member",
    path: () =>
      `/player/?id=${readSeed().programs.withCover.meditations[0].id}`,
  },
  { key: "soundscape", role: "member", path: () => "/soundscape/" },
  { key: "mood", role: "member", path: () => "/mood/" },
  { key: "history", role: "member", path: () => "/history/" },
  { key: "stats", role: "member", path: () => "/stats/" },
  { key: "settings", role: "member", path: () => "/settings/" },
  { key: "admin", role: "admin", path: () => "/admin/" },
];

type Sample = {
  /** Static or hydrated shell skeleton present. */
  skeleton: boolean;
  busy: number;
  /** Main regions: a Panel or a region-state wrapper. */
  regions: { busy: boolean; text: number }[];
  mainText: number;
  hasMain: boolean;
};

/** Text-node character sum of an element, whitespace-trimmed. */
const snapshot = (page: Page): Promise<Sample> =>
  page.evaluate(() => {
    const textOf = (el: Element): number => {
      const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
      let sum = 0;
      for (let n = walker.nextNode(); n; n = walker.nextNode())
        sum += (n.textContent ?? "").trim().length;
      return sum;
    };
    const main = document.querySelector("main");
    const regions = main
      ? [
          ...main.querySelectorAll(
            "[data-region-state], [data-slot=panel], section[aria-label]",
          ),
        ].map((el) => ({
          busy:
            el.getAttribute("aria-busy") === "true" ||
            el.querySelector('[aria-busy="true"]') !== null ||
            el.getAttribute("data-region-state") === "loading",
          text: textOf(el),
        }))
      : [];
    return {
      skeleton:
        document.querySelector("[data-shell-skeleton]") !== null ||
        document.querySelector("[data-region-state=loading]") !== null,
      busy: document.querySelectorAll('[aria-busy="true"]').length,
      regions,
      mainText: main ? textOf(main) : 0,
      hasMain: main !== null,
    };
  });

for (const screen of screens) {
  test.describe(screen.key, () => {
    test.use({ storageState: storageStatePath(screen.role) });

    test(`${screen.key}: 3 s delay on /api/** is never blank (SC-006)`, async ({
      page,
    }) => {
      await page.route("**/api/**", async (route) => {
        await new Promise((resolve) => setTimeout(resolve, DELAY_MS));
        await route.continue().catch(() => {});
      });

      const started = Date.now();
      await page.goto(screen.path(), { waitUntil: "domcontentloaded" });
      const samples: Sample[] = [];
      while (Date.now() - started < DELAY_MS - SAMPLE_MS) {
        try {
          samples.push(await snapshot(page));
        } catch {
          // Mid-navigation: the execution context was replaced; resample.
        }
        await page.waitForTimeout(SAMPLE_MS);
      }
      expect(samples.length).toBeGreaterThanOrEqual(5);

      samples.forEach((s, i) => {
        const at = `sample ${i} (${i * SAMPLE_MS} ms)`;
        // Something signals loading: a skeleton or an aria-busy region.
        expect(
          s.skeleton || s.busy > 0,
          `${at}: no skeleton or aria-busy`,
        ).toBe(true);
        // Main regions: busy, or carrying text.
        s.regions.forEach((r, j) =>
          expect(
            r.busy || r.text > 0,
            `${at}: region ${j} blank and not busy`,
          ).toBe(true),
        );
        if (s.hasMain)
          expect(
            s.mainText > 0 || s.busy > 0,
            `${at}: main has no text and no aria-busy`,
          ).toBe(true);
      });

      // After release the screen resolves out of loading.
      await expect(page.locator("main")).toBeVisible({ timeout: 15_000 });
      await expect(page.locator('main [aria-busy="true"]')).toHaveCount(0, {
        timeout: 15_000,
      });
    });
  });
}
