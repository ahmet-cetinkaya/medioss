// T25: one featured element and one primary action (FR-101). Per screen, at
// most one [data-elevation=featured]; within each Panel (data-slot=panel),
// section[aria-label], and open dialog, at most one [data-variant=primary].
import { storageStatePath, type Role } from "../support/auth";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";

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

const CONTAINERS = '[data-slot=panel], section[aria-label], [role="dialog"]';

for (const screen of screens) {
  test.describe(screen.key, () => {
    test.use({ storageState: storageStatePath(screen.role) });

    test(`${screen.key}: at most one featured element and one primary action per panel (FR-101)`, async ({
      page,
    }) => {
      await page.goto(screen.path());
      await expect(page.locator("main")).toBeVisible();
      await expect(page.locator('main [aria-busy="true"]')).toHaveCount(0);

      expect(
        await page.locator("[data-elevation=featured]").count(),
      ).toBeLessThanOrEqual(1);

      const counts = await page.locator(CONTAINERS).evaluateAll((els) =>
        els.map((el) => ({
          label:
            el.getAttribute("aria-label") ??
            el.getAttribute("aria-labelledby") ??
            el.tagName,
          primary: el.querySelectorAll("[data-variant=primary]").length,
        })),
      );
      for (const c of counts)
        expect(c.primary, `panel "${c.label}"`).toBeLessThanOrEqual(1);
    });
  });
}
