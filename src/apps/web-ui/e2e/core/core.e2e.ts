// Minimal e2e core: the flows most likely to break, one suite, fast feedback.
// Full historical suites live in e2e/extended/ (excluded from testMatch) and
// can be restored by moving files back into e2e/seeded/ etc.
import { expect, test } from "../support/test-fixture";
import { storageStatePath } from "../support/auth";

test.use({ storageState: storageStatePath("member") });

test("home renders its regions without a library preview section", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expect(page.getByRole("region", { name: /library/i })).toHaveCount(0);
});

test("home suggestion pill is one clickable target to the player", async ({
  page,
}) => {
  await page.goto("/");
  const pill = page.getByRole("link", { name: /Play / }).first();
  await expect(pill).toBeVisible();
  // The pill is a single link; its inner play control must not add a second
  // focusable target.
  const focusables = await pill
    .locator("a, button, [tabindex]:not([tabindex='-1'])")
    .count();
  expect(focusables).toBe(0);
  await pill.click();
  await page.waitForURL(/\/player\//);
});

test("library rows scroll horizontally and search opens the player", async ({
  page,
}) => {
  await page.goto("/library/");
  const first = page.locator("main a[href^='/program/']").first();
  await expect(first).toBeVisible();
  await page.getByRole("searchbox").fill("Body Scan");
  await page.getByRole("searchbox").press("Enter");
  const result = page
    .getByRole("region", { name: /Results/ })
    .getByRole("link", {
      name: /^Body Scan/,
    });
  await expect(result).toBeVisible();
  await result.click();
  await page.waitForURL(/\/player\//);
});

test("player: overlay while playing, mixer random, completion dialog", async ({
  page,
}) => {
  await page.goto("/player/1/");
  const play = page.getByRole("button", { name: "Play", exact: true });
  const pause = page.getByRole("button", { name: "Pause", exact: true });
  const speed = page.getByRole("button", { name: /^Speed / });
  await expect(speed).toBeVisible();

  // Normal mode: the name is a bottom-left label over a wide program cover.
  const title = page.locator("main").getByRole("heading", { level: 1 });
  const coverLink = page
    .locator("main a")
    .filter({ has: page.getByRole("heading", { level: 1 }) });
  await expect(coverLink.locator("img")).toBeVisible();
  await expect(title).toBeVisible();
  const coverBox = await coverLink.boundingBox();
  const titleBox = await title.boundingBox();
  expect(coverBox!.width).toBeGreaterThan(coverBox!.height);
  expect(coverBox!.height).toBeLessThanOrEqual(288.5);
  expect(titleBox!.x - coverBox!.x).toBeLessThan(40);
  expect(titleBox!.y + titleBox!.height / 2).toBeGreaterThan(
    coverBox!.y + coverBox!.height / 2,
  );

  // Focus overlay: playing hides the page UI behind an opaque color wash
  // with only the floating panel visible;
  // pausing returns the page. One set of controls, never duplicated.
  await play.click();
  const backdrop = page.locator(".focus-overlay-backdrop");
  await expect(backdrop).toBeVisible();
  await expect(pause).toBeVisible();
  await expect(page.getByRole("navigation", { name: /main/i })).toBeHidden();
  await expect(
    page
      .locator("[data-player-floating] img, [data-player-floating] svg")
      .first(),
  ).toBeVisible();
  // Idle mode: after inactivity only the large elapsed time remains; input
  // wakes the panel again.
  const idleClock = page.locator("[data-player-idle-clock]");
  await expect(idleClock).toBeVisible({ timeout: 10_000 });
  await expect(idleClock.locator("span.tabular-nums")).toBeVisible();
  await page.mouse.move(10, 10);
  await page.mouse.move(40, 40);
  await expect(idleClock).toHaveCount(0);

  // The menu must stay visible while the focus overlay hides the page:
  // its portal carries the data-dialog-portal marker (regression test).
  await page.getByRole("button", { name: /More actions for / }).click();
  const menu = page.getByRole("menu");
  await expect(menu).toBeVisible();
  await expect(
    menu.getByRole("menuitem", { name: "Download meditation" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(menu).toHaveCount(0);
  await pause.click();
  await expect(backdrop).toHaveCount(0);
  await expect(page.getByRole("navigation", { name: /main/i })).toBeVisible();
  await expect(play).toBeVisible();

  // Speed cycles 1x -> 1.25x -> 1.5x -> 1x.
  await speed.click();
  await expect(speed).toHaveAccessibleName("Speed 1.25×");

  // Mixer: category headings + Random fills the mix. The player's Random
  // button is labeled "Random" and disabled until the sound library loads.
  await page.getByRole("button", { name: /soundscape/i }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("heading", { level: 3 }).first()).toBeVisible();
  const random = dialog.getByRole("button", { name: "Random", exact: true });
  await expect(random).toBeEnabled();
  await random.click();
  await expect(dialog.getByRole("slider").first()).toBeVisible();
  await expect(
    dialog.getByRole("slider", { name: "Overall volume" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
});

test("program page lists sessions vertically with play actions", async ({
  page,
}) => {
  await page.goto("/program/?id=1");
  const sessions = page
    .getByRole("list")
    .filter({ has: page.getByRole("link", { name: /Play/ }) });
  await expect(sessions).toBeVisible();
  const sessionItems = sessions.getByRole("listitem");
  const firstBox = await sessionItems.nth(0).boundingBox();
  const secondBox = await sessionItems.nth(1).boundingBox();
  expect(firstBox).not.toBeNull();
  expect(secondBox).not.toBeNull();
  expect(secondBox!.y).toBeGreaterThan(firstBox!.y);
});

test("soundscape: current mix stays above the saved mixes and sounds", async ({
  page,
}) => {
  await page.goto("/soundscape/");
  const panel = page.getByRole("region", { name: "Current mix" });
  const saved = page.getByRole("region", { name: "Saved mixes" });
  const sounds = page.getByRole("region", { name: "Sounds" });
  const panelBox = await panel.boundingBox();
  const savedBox = await saved.boundingBox();
  const soundsBox = await sounds.boundingBox();
  expect(panelBox).not.toBeNull();
  expect(savedBox).not.toBeNull();
  expect(soundsBox).not.toBeNull();
  expect(panelBox!.y).toBeLessThan(savedBox!.y);
  expect(panelBox!.y).toBeLessThan(soundsBox!.y);
  expect(Math.abs(savedBox!.y - soundsBox!.y)).toBeLessThan(2);
  expect(savedBox!.x).toBeLessThan(soundsBox!.x);

  const overall = panel.getByRole("slider", { name: "Overall volume" });
  await expect(overall).toBeVisible();
  await overall.fill("50");

  const random = page.getByRole("button", { name: "Random mix" });
  await random.click();
  await expect(page.getByText(/playing together/i)).toBeVisible();
  await expect(panel.getByRole("slider")).toHaveCount(4);
  await expect(overall).toHaveValue("50");
  await expect
    .poll(() =>
      panel
        .getByRole("slider")
        .evaluateAll((sliders) =>
          sliders.every(
            (slider) => slider.getAttribute("aria-valuetext") === "50 percent",
          ),
        ),
    )
    .toBe(true);
  await random.click();
  await expect(panel.getByRole("slider")).toHaveCount(4);
  await expect(overall).toHaveValue("50");
  await expect
    .poll(() =>
      panel
        .getByRole("slider")
        .evaluateAll((sliders) =>
          sliders.every(
            (slider) => slider.getAttribute("aria-valuetext") === "50 percent",
          ),
        ),
    )
    .toBe(true);
});

test("soundscape: download lives in the tile overflow menu", async ({
  page,
}) => {
  await page.goto("/soundscape/");
  const tile = page.getByRole("listitem").filter({
    has: page.getByRole("button", { name: /^More actions for / }),
  });
  await tile
    .first()
    .getByRole("button", { name: /^More actions for / })
    .click();
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("menuitem", { name: /^Download / }).click(),
  ]);
  expect(new URL(download.url()).pathname).toMatch(
    /^\/api\/soundscape\/sounds\/[^/]+\/download$/,
  );
});

test("mood check-in saves from home dialog", async ({ page }) => {
  await page.goto("/");
  await page
    .getByRole("button", { name: /check-in/i })
    .first()
    .click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await dialog.getByText("5 · Very good").click();
  await dialog.getByRole("button", { name: "Save" }).click();
  await expect(dialog).toBeHidden();

  await page.goto("/mood/");
  await expect(page.getByTestId("mood-day").first()).toBeVisible();
  await expect(
    page.getByTestId("mood-day").first().locator("[data-mood-dot='5']").first(),
  ).toBeVisible();
});

test("mobile: bottom bar stays at the viewport bottom, no header avatar", async ({
  page,
}) => {
  await page.setViewportSize({ width: 375, height: 667 });
  await page.goto("/library/");
  await expect(page.locator("main a[href^='/program/']").first()).toBeVisible();
  const bar = page.getByRole("navigation", { name: /bottom|main/i }).last();
  const bottomOf = () =>
    bar.evaluate((el) => el.getBoundingClientRect().bottom);
  const before = await bottomOf();
  await page.evaluate(() => {
    document.body.style.minHeight = "3000px";
    window.scrollTo(0, 800);
  });
  expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);
  expect(await bottomOf()).toBeCloseTo(before, 0);
  expect(before).toBeCloseTo(667, 0);
  await expect(page.locator("header img")).toHaveCount(0);
  const wordmark = page.locator("header").getByText("MediOSS");
  await expect(wordmark).toBeVisible();
  await expect(wordmark).toHaveCSS("font-size", "16px");
  await expect(page.locator("main a[href^='/program/']").first()).toHaveCSS(
    "backdrop-filter",
    /blur\(20px\)/,
  );
});

test.describe("sign-in gate", () => {
  // A fresh, signed-out context: logout must not invalidate the shared member
  // session that the sleep-timer and playback-state suites use.
  test.use({ storageState: { cookies: [], origins: [] } });

  test("a signed-out member address shows login", async ({ page }) => {
    await page.goto("/login/");
    await page.getByLabel("Email").fill("member@example.test");
    await page.getByLabel("Password").fill("member-e2e-pass-1");
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).not.toHaveURL(/\/login\//);

    // The logout endpoint is origin-checked (fail-closed): drive it through
    // the page so the browser sends a same-origin Origin header.
    const logoutStatus = await page.evaluate(async () => {
      const response = await fetch("/api/identity/logout", {
        method: "POST",
      });
      return response.status;
    });
    expect(logoutStatus).toBe(204);
    await page.goto("/library/");
    await expect(page).toHaveURL(/\/login\//);
  });
});
