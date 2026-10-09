// T14: the app shell on the pilot screen (plan T12 acceptance (c) and (d);
// FR-003–FR-008, FR-096, FR-105–FR-107, SC-004).
import type { BrowserContext, Page } from "@playwright/test";
import { newApi, postJson } from "../support/api-client";
import { loginApi, storageStatePath, USERS } from "../support/auth";
import { expect, test } from "../support/test-fixture";
import { viewport, type ViewportWidth } from "../support/viewport";

const signedOut = { cookies: [], origins: [] };

/** A new session for this context only, so the shared storage states stay valid. */
async function signInFresh(
  context: BrowserContext,
  email: string,
  password: string,
): Promise<void> {
  const api = await loginApi(email, password);
  await context.addCookies((await api.storageState()).cookies);
  await api.dispose();
}

async function openStats(page: Page, path = "/stats/"): Promise<void> {
  await page.goto(path);
  await expect(page.locator("main dl dt")).toHaveCount(4);
}

const sidebar = (page: Page) =>
  page.getByRole("navigation", { name: "Main navigation" });

async function confirmSignOut(page: Page): Promise<void> {
  await sidebar(page).getByRole("button", { name: "Sign out" }).click();
  await page
    .getByRole("alertdialog")
    .getByRole("button", { name: "Sign out" })
    .click();
}

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  for (const width of [1440, 1100, 800, 375] as ViewportWidth[]) {
    test(`exactly one current item per visible nav surface at ${width}px`, async ({
      page,
    }) => {
      await page.setViewportSize(viewport(width));
      await openStats(page);
      const navs = page.locator("nav:visible");
      await expect(navs).toHaveCount(1);
      await expect(navs.locator('[aria-current="page"]')).toHaveCount(1);
      if (width >= 768)
        await expect(
          navs.locator('[aria-current="page"]'),
        ).toHaveAccessibleName("Statistics");
      await expect(navs.getByRole("link", { name: "Admin" })).toHaveCount(0);
    });
  }

  test("the More sheet has one current item and no Admin link", async ({
    page,
  }) => {
    await page.setViewportSize(viewport(375));
    await openStats(page);
    await page.getByRole("button", { name: "More" }).click();
    const more = page
      .getByRole("dialog", { name: "More" })
      .getByRole("navigation", { name: "More pages" });
    await expect(more.locator('[aria-current="page"]')).toHaveCount(1);
    await expect(more.locator('[aria-current="page"]')).toHaveAccessibleName(
      "Statistics",
    );
    await expect(more.getByRole("link", { name: "Admin" })).toHaveCount(0);
  });

  test("the skip link is the first Tab stop and moves focus to main", async ({
    page,
  }) => {
    await openStats(page);
    await page.keyboard.press("Tab");
    const skip = page.getByRole("link", { name: "Skip to main content" });
    await expect(skip).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page.locator("main#main")).toBeFocused();
  });

  test("a failed logout (500) stays on the screen and shows the message", async ({
    page,
  }) => {
    await page.route("**/api/identity/logout", (route) =>
      route.fulfill({ status: 500, json: { error: "simulated" } }),
    );
    await openStats(page);
    await confirmSignOut(page);
    const dialog = page.getByRole("alertdialog");
    await expect(dialog.getByRole("alert")).toHaveText(
      /Sign-out did not complete/,
    );
    await expect(
      page.locator('[role="status"][aria-live="polite"]'),
    ).toContainText("Sign-out did not complete");
    expect(new URL(page.url()).pathname).toBe("/stats/");
    await expect(page.locator("main")).not.toContainText("simulated");
  });
});

test.describe("admin", () => {
  test.use({ storageState: storageStatePath("admin") });

  test("the admin sees Admin in the sidebar and the More sheet", async ({
    page,
  }) => {
    await openStats(page);
    await expect(
      sidebar(page).getByRole("link", { name: "Admin" }),
    ).toHaveCount(1);
    await page.setViewportSize(viewport(375));
    await page.getByRole("button", { name: "More" }).click();
    await expect(
      page
        .getByRole("navigation", { name: "More pages" })
        .getByRole("link", { name: "Admin" }),
    ).toHaveCount(1);
  });
});

test.describe("sign-out (fresh sessions)", () => {
  test.use({ storageState: signedOut });

  test("cancel stays on the screen, still signed in (SC-004)", async ({
    page,
    context,
  }) => {
    await signInFresh(context, USERS.member.email, USERS.member.password);
    await openStats(page);
    await sidebar(page).getByRole("button", { name: "Sign out" }).click();
    const dialog = page.getByRole("alertdialog");
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toHaveCount(0);
    expect(new URL(page.url()).pathname).toBe("/stats/");
    expect((await page.request.get("/api/identity/me")).status()).toBe(200);
  });

  test("sign-in then sign-out: login, reopen and Back show login; storage clean (SC-004, T12d)", async ({
    page,
    context,
  }) => {
    // Sign in through the (legacy, until T15) login screen.
    await page.goto("/login/");
    await page.getByLabel("Email").fill(USERS.member.email);
    await page.getByLabel("Password").fill(USERS.member.password);
    await page.getByRole("button", { name: "Sign in" }).click();
    await page.waitForURL((url) => url.pathname === "/");

    await openStats(page, "/stats/?from=back");
    await openStats(page);
    const sessionCookies = (await context.cookies()).map((c) => c.name);
    expect(sessionCookies.length).toBeGreaterThan(0);

    const logout = page.waitForResponse("**/api/identity/logout");
    await confirmSignOut(page);
    expect((await logout).status()).toBe(204);
    await page.waitForURL((url) => url.pathname === "/login/");
    expect(new URL(page.url()).search).toBe("");

    const storage = await page.evaluate(() => ({
      local: Object.keys(localStorage),
      session: sessionStorage.length,
      cookie: document.cookie,
    }));
    for (const key of storage.local) expect(["medioss.locale"]).toContain(key);
    expect(storage.session).toBe(0);
    for (const name of sessionCookies)
      expect(storage.cookie).not.toContain(name);

    // Back (bfcache or reload) must not show the signed-in page.
    await page.goBack({ waitUntil: "commit" });
    await page.waitForURL((url) => url.pathname === "/login/");
    await expect(page.locator("nav")).toHaveCount(0);

    await page.goto("/stats/");
    await page.waitForURL("**/login/?next=%2Fstats%2F");
  });

  test("logout 401 (session already ended) goes to login without an error", async ({
    page,
    context,
  }) => {
    await signInFresh(context, USERS.member.email, USERS.member.password);
    await openStats(page);
    // End the same session server-side (shared cookie jar) before confirming.
    const ended = await page.request.post("/api/identity/logout", {
      headers: { Origin: new URL(page.url()).origin },
    });
    expect(ended.status()).toBe(204);

    const logout = page.waitForResponse("**/api/identity/logout");
    await confirmSignOut(page);
    expect((await logout).status()).toBe(401);
    await page.waitForURL((url) => url.pathname === "/login/");
    expect(new URL(page.url()).search).toBe("");
    await expect(page.getByText("Sign-out did not complete")).toHaveCount(0);
  });

  test("the seeded flagged user sees the change-password view at /stats/, without nav", async ({
    page,
    context,
  }) => {
    await signInFresh(
      context,
      USERS.flagged.email,
      USERS.flagged.temporaryPassword,
    );
    await page.goto("/stats/");
    await expect(
      page.getByRole("heading", { level: 1, name: "Change password" }),
    ).toBeVisible();
    expect(new URL(page.url()).pathname).toBe("/stats/");
    await expect(page.locator("nav")).toHaveCount(0);
    await expect(page.locator("main dl")).toHaveCount(0);
  });

  test("a required password change returns 200 and /stats/ renders at the same URL (FR-107)", async ({
    page,
    context,
  }) => {
    const email = `t14-${Date.now()}@example.test`;
    const temporary = "t14-e2e-temp-1";
    const admin = await newApi(storageStatePath("admin"));
    await postJson(admin, "/api/identity/users", {
      email,
      password: temporary,
    });
    await admin.dispose();

    await signInFresh(context, email, temporary);
    await page.goto("/stats/");
    await expect(
      page.getByRole("heading", { level: 1, name: "Change password" }),
    ).toBeVisible();
    await expect(page.locator("nav")).toHaveCount(0);

    await page
      .getByLabel("New password", { exact: true })
      .fill("t14-e2e-new-pass-1");
    await page.getByLabel("Confirm new password").fill("t14-e2e-new-pass-1");
    const changed = page.waitForResponse("**/api/identity/me/password");
    await page.getByRole("button", { name: "Change password" }).click();
    expect((await changed).status()).toBe(200);

    await expect(page.locator("main dl dt")).toHaveCount(4);
    expect(new URL(page.url()).pathname).toBe("/stats/");
    await expect(
      page.getByRole("heading", { level: 1, name: "Statistics" }),
    ).toBeVisible();
    await expect(page.locator("nav:visible")).toHaveCount(1);
  });
});
