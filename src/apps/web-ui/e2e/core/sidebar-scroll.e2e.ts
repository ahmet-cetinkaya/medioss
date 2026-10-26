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
