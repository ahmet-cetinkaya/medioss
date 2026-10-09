// T25: navigation reachability (plan T25; FR-002, FR-003, FR-004, FR-087,
// SC-001, SC-002). At each of the four layout tiers (sidebar >= 1280, icon
// rail 768–1279, bottom bar + More sheet < 768) every nav destination is
// reachable from every authenticated screen in at most two activations (a
// mobile sheet destination costs two: More, then the link), exactly one
// current item shows per visible nav surface (Library for Program and
// Player; More stands in on mobile for sheet-hosted screens), the member
// sees no Admin entry anywhere, and the admin sees Admin on every screen.
import type { Page } from "@playwright/test";
import { storageStatePath, type Role } from "../support/auth";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { viewport, type ViewportWidth } from "../support/viewport";

const WIDTHS = [
  1440, 1100, 800, 375,
] as const satisfies readonly ViewportWidth[];

/** The one nav surface AppShell leaves visible at each tier. */
const SURFACE_LABEL: Record<number, string> = {
  1440: "Main navigation",
  1100: "Compact navigation",
  800: "Compact navigation",
  375: "Bottom navigation",
};

type Destination = {
  name: string;
  path: string;
  /** Shown in the mobile bottom bar; the rest sit behind More (FR-087). */
  primary: boolean;
  adminOnly?: boolean;
};

const DESTINATIONS: readonly Destination[] = [
  { name: "Home", path: "/", primary: true },
  { name: "Library", path: "/library/", primary: true },
  { name: "Soundscape", path: "/soundscape/", primary: true },
  { name: "Mood", path: "/mood/", primary: true },
  { name: "History", path: "/history/", primary: false },
  { name: "Statistics", path: "/stats/", primary: false },
  { name: "Settings", path: "/settings/", primary: false },
  { name: "Admin", path: "/admin/", primary: false, adminOnly: true },
];

const destinations = (role: Role): readonly Destination[] =>
  DESTINATIONS.filter((dest) => role === "admin" || !dest.adminOnly);

type Screen = { key: string; path: string; current: string };

/** The ten authenticated screens; /admin/ only for the admin role. */
function screens(role: Role): Screen[] {
  const seed = readSeed();
  const list: Screen[] = [
    { key: "home", path: "/", current: "Home" },
    { key: "library", path: "/library/", current: "Library" },
    {
      key: "program",
      path: `/program/?id=${seed.programs.withCover.id}`,
      // Program and Player are not nav items: Library is current (SC-001).
      current: "Library",
    },
    {
      key: "player",
      path: `/player/?id=${seed.programs.withCover.meditations[0].id}`,
      current: "Library",
    },
    { key: "soundscape", path: "/soundscape/", current: "Soundscape" },
    { key: "mood", path: "/mood/", current: "Mood" },
    { key: "history", path: "/history/", current: "History" },
    { key: "stats", path: "/stats/", current: "Statistics" },
    { key: "settings", path: "/settings/", current: "Settings" },
  ];
  if (role === "admin")
    list.push({ key: "admin", path: "/admin/", current: "Admin" });
  return list;
}

/** Bottom-bar items; every other current item lives behind More (FR-087). */
const PRIMARY_CURRENT = new Set(["Home", "Library", "Soundscape", "Mood"]);

const isMobile = (width: number) => width < 768;
const surface = (page: Page, width: number) =>
  page.getByRole("navigation", { name: SURFACE_LABEL[width] });
const moreSheet = (page: Page) =>
  page
    .getByRole("dialog", { name: "More" })
    .getByRole("navigation", { name: "More pages" });

/** Opens a screen and asserts the tier's one visible surface carries exactly
 *  one current item with the expected accessible name (SC-001). */
async function openScreen(page: Page, screen: Screen, width: number) {
  await page.goto(screen.path, { waitUntil: "commit" });
  await expect(page.locator("nav:visible")).toHaveCount(1);
  const current = surface(page, width).locator('[aria-current="page"]');
  await expect(current).toHaveCount(1);
  const expected =
    isMobile(width) && !PRIMARY_CURRENT.has(screen.current)
      ? "More"
      : screen.current;
  await expect(current).toHaveAccessibleName(expected);
}

/** Reaches `dest` from the current screen with at most two activations and
 *  lands on its path: one click in the visible surface, or More then the
 *  link on mobile for sheet-hosted destinations (SC-001). */
async function activate(
  page: Page,
  width: number,
  dest: Destination,
  screen: Screen,
  role: Role,
) {
  if (isMobile(width) && !dest.primary) {
    await surface(page, width).getByRole("button", { name: "More" }).click();
    const sheet = moreSheet(page);
    // SC-002 inside the sheet: Admin appears for the admin only.
    await expect(sheet.getByRole("link", { name: "Admin" })).toHaveCount(
      role === "admin" ? 1 : 0,
    );
    // The sheet marks the current page only when it hosts it (FR-087).
    await expect(sheet.locator('[aria-current="page"]')).toHaveCount(
      PRIMARY_CURRENT.has(screen.current) ? 0 : 1,
    );
    await sheet.getByRole("link", { name: dest.name }).click();
  } else {
    await surface(page, width).getByRole("link", { name: dest.name }).click();
  }
  await page.waitForURL((url) => url.pathname === dest.path, {
    waitUntil: "commit",
  });
}

for (const role of ["member", "admin"] as const) {
  test.describe(role, () => {
    test.use({ storageState: storageStatePath(role) });

    for (const width of WIDTHS) {
      test(`at ${width}px: one current item per surface and every destination in ≤2 activations (SC-001), ${role === "member" ? "no Admin entry" : "Admin on every screen"} (SC-002)`, async ({
        page,
      }) => {
        test.setTimeout(5 * 60_000);
        await page.setViewportSize(viewport(width));

        for (const screen of screens(role)) {
          await openScreen(page, screen, width);

          // SC-002 on every screen: the member has no Admin entry anywhere,
          // the admin has exactly one in the visible surface (the mobile
          // sheet is asserted in `activate` on every More open).
          if (role === "member") {
            await expect(page.getByRole("link", { name: "Admin" })).toHaveCount(
              0,
            );
          } else if (!isMobile(width)) {
            await expect(
              surface(page, width).getByRole("link", { name: "Admin" }),
            ).toHaveCount(1);
          }

          const here = new URL(screen.path, "http://localhost/");
          for (const dest of destinations(role)) {
            if (dest.path === here.pathname) {
              // Already there (0 activations): the control must still exist
              // with the right href in the surface that hosts it.
              if (isMobile(width) && !dest.primary) {
                await surface(page, width)
                  .getByRole("button", { name: "More" })
                  .click();
                await expect(
                  moreSheet(page).getByRole("link", { name: dest.name }),
                ).toHaveAttribute("href", dest.path);
                await page.keyboard.press("Escape");
              } else {
                await expect(
                  surface(page, width).getByRole("link", { name: dest.name }),
                ).toHaveAttribute("href", dest.path);
              }
              continue;
            }
            await activate(page, width, dest, screen, role);
            expect(new URL(page.url()).pathname).toBe(dest.path);
            // Back to the same screen so the next destination is again
            // measured from it. Admin canonicalizes its address to
            // `?section=users` through replaceState (FR-072), so its history
            // entry may carry that canonical search.
            await page.goBack({ waitUntil: "commit" });
            await expect(page).toHaveURL(
              (url) =>
                url.pathname === here.pathname &&
                (here.pathname === "/admin/" || url.search === here.search),
            );
          }
        }
      });
    }
  });
}
