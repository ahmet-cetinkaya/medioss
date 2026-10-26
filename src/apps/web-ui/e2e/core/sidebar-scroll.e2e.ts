import { storageStatePath } from "../support/auth";
import { expect, test } from "../support/test-fixture";

// A short window cannot fit every sidebar link; the shell clips its overflow,
// so the sidebar must scroll on its own or the bottom links are unreachable.
const SHORT_DESKTOP = { width: 1280, height: 500 };
const MAX_TABS = 40;

test.describe("sidebar in a short window", () => {
  test.use({
    storageState: storageStatePath("member"),
    viewport: SHORT_DESKTOP,
  });

  test("scrolls on its own and keeps the last link reachable", async ({
    page,
  }) => {
    await page.goto("/");
    const sidebar = page.getByRole("navigation", { name: "Main navigation" });
    const lastLink = sidebar.getByRole("button", { name: "Support" });

    await expect(lastLink).not.toBeInViewport();

    await sidebar.evaluate((nav) => nav.scrollTo({ top: nav.scrollHeight }));
    await expect(lastLink).toBeInViewport();
  });

  test("Tab reaches the last link and scrolls it into view", async ({
    page,
  }) => {
    await page.goto("/");
    const sidebar = page.getByRole("navigation", { name: "Main navigation" });
    const lastLink = sidebar.getByRole("button", { name: "Support" });

    await sidebar.getByRole("link").first().focus();
    for (let presses = 0; presses < MAX_TABS; presses++) {
      if (await lastLink.evaluate((el) => el === document.activeElement)) break;
      await page.keyboard.press("Tab");
    }

    await expect(lastLink).toBeFocused();
    // Without its own overflow the clipped shell scrolls instead, and the
    // whole window frame slides; the sidebar itself must do the scrolling.
    expect(await sidebar.evaluate((nav) => nav.scrollTop)).toBeGreaterThan(0);
    await expect(lastLink).toBeInViewport();
  });
});

// The icon rail (768-1279 px) sits in the same clipped shell and had the same
// bug, so it gets the same guarantee.
const SHORT_NARROW_DESKTOP = { width: 1000, height: 400 };

test.describe("icon rail in a short window", () => {
  test.use({
    storageState: storageStatePath("member"),
    viewport: SHORT_NARROW_DESKTOP,
  });

  test("scrolls on its own and keeps the last item reachable", async ({
    page,
  }) => {
    await page.goto("/");
    const rail = page.getByRole("navigation", { name: "Compact navigation" });
    const lastItem = rail.getByRole("button", { name: "Sign out" });

    await expect(lastItem).not.toBeInViewport();

    await rail.evaluate((nav) => nav.scrollTo({ top: nav.scrollHeight }));
    await expect(lastItem).toBeInViewport();
  });
});
