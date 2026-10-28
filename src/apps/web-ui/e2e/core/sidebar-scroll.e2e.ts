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
    // About is the last group after Account (Sign out); a member sees no
    // Admin item.
    const lastItem = rail.getByRole("link", { name: "About" });

    await expect(lastItem).not.toBeInViewport();

    await rail
      .locator("[data-rail-scroll]")
      .evaluate((box) => box.scrollTo({ top: box.scrollHeight }));
    await expect(lastItem).toBeInViewport();
  });
});

// The rail's own scroll box must not clip the labels: the tooltip lives
// outside it, positioned against the nav.
const TALL_NARROW_DESKTOP = { width: 1000, height: 800 };

test.describe("icon rail tooltips", () => {
  test.use({
    storageState: storageStatePath("member"),
    viewport: TALL_NARROW_DESKTOP,
  });

  test.describe("in a short window", () => {
    test.use({ viewport: SHORT_NARROW_DESKTOP });

    test("rail never overflows horizontally", async ({ page }) => {
      await page.goto("/");
      const rail = page.getByRole("navigation", { name: "Compact navigation" });
      await expect(
        rail.evaluate((nav) => nav.scrollWidth <= nav.clientWidth),
      ).resolves.toBe(true);
    });

    test("a focused item's tooltip is fully visible and unclipped", async ({
      page,
    }) => {
      await page.goto("/");
      const rail = page.getByRole("navigation", { name: "Compact navigation" });
      await rail.locator("[data-tip]").first().focus();

      const tooltip = rail.locator("[data-rail-tooltip]");
      await expect(tooltip).toBeVisible();

      const tipBox = await tooltip.boundingBox();
      expect(tipBox).not.toBeNull();
      expect(tipBox!.width).toBeGreaterThan(0);
      // Fully inside the viewport (nothing there clips it)...
      expect(tipBox!.x).toBeGreaterThanOrEqual(0);
      expect(tipBox!.x + tipBox!.width).toBeLessThanOrEqual(
        page.viewportSize()!.width,
      );
      expect(tipBox!.y).toBeGreaterThanOrEqual(0);
      expect(tipBox!.y + tipBox!.height).toBeLessThanOrEqual(
        page.viewportSize()!.height,
      );
      // ...reaching past the rail's right edge (not hidden inside it)...
      const railBox = await rail.boundingBox();
      expect(tipBox!.x + tipBox!.width).toBeGreaterThan(
        railBox!.x + railBox!.width,
      );
      // ...and not clipped by any ancestor between the tooltip and the document.
      const clipped = await tooltip.evaluate((el) => {
        for (let a = el.parentElement; a; a = a.parentElement) {
          const style = getComputedStyle(a);
          if (style.overflow !== "visible") {
            const box = a.getBoundingClientRect();
            const r = el.getBoundingClientRect();
            if (r.right > box.right + 1 || r.left < box.left - 1) return true;
          }
        }
        return false;
      });
      expect(clipped).toBe(false);
    });

    test("Tab-focusing an out-of-view item shows its tooltip", async ({
      page,
    }) => {
      await page.goto("/");
      const rail = page.getByRole("navigation", { name: "Compact navigation" });
      // Focusing an out-of-view item: focusin sets the tip, then the browser's
      // focus scroll fires onscroll, which used to clear it (tip = null).
      const items = rail.locator("[data-tip]");
      const beforeLast = items.nth((await items.count()) - 2);
      await beforeLast.focus();
      await expect(beforeLast).toBeFocused();

      const tooltip = rail.locator("[data-rail-tooltip]");
      // Wait until the focus scroll has actually happened; clearing the tip is
      // exactly its job, so the assertion below is only meaningful after it.
      const box = rail.locator("[data-rail-scroll]");
      await expect(box.evaluate((el) => el.scrollTop > 0)).resolves.toBe(true);
      // The tip must have survived that scroll, re-anchored to the item.
      await expect(tooltip).toHaveText(
        await beforeLast.getAttribute("aria-label"),
      );

      await page.keyboard.press("Tab");
      const lastItem = items.last();
      await expect(lastItem).toBeFocused();
      await expect(tooltip).toHaveText(
        await lastItem.getAttribute("aria-label"),
      );
      const tipBox = await tooltip.boundingBox();
      expect(tipBox!.y).toBeGreaterThanOrEqual(0);
      expect(tipBox!.y + tipBox!.height).toBeLessThanOrEqual(
        page.viewportSize()!.height,
      );
    });
  });
});
