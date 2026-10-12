// T15: Login (FR-080–FR-082, FR-113, PB-14, SC-003, SC-022).
// Throttling uses a throwaway email so no shared user is ever locked out; the
// only failed attempt on a seeded user is cleared by its successful sign-in.
import type { Page } from "@playwright/test";
import { newApi } from "../support/api-client";
import { storageStatePath, USERS } from "../support/auth";
import { saveScreenshot } from "../support/evidence";
import { localeOptions } from "../support/locale";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

const LOGIN = "/api/identity/login";
const signedOut = { cookies: [], origins: [] };

const emailField = (page: Page) => page.getByLabel("Email");
const passwordField = (page: Page) => page.getByLabel("Password");
const signIn = (page: Page) => page.getByRole("button", { name: "Sign in" });

async function signInAsMember(page: Page): Promise<void> {
  await expect(emailField(page)).toBeEditable();
  await emailField(page).fill(USERS.member.email);
  await passwordField(page).fill(USERS.member.password);
  await signIn(page).click();
}

test.describe("signed out", () => {
  test.use({ storageState: signedOut });

  test("a wrong password shows one generic message and keeps the email (FR-080, PB-14)", async ({
    page,
  }) => {
    await page.goto("/login/");
    await emailField(page).fill(USERS.member.email);
    await passwordField(page).fill("not-the-password");
    const refused = page.waitForResponse(
      (r) => new URL(r.url()).pathname === LOGIN,
    );
    await signIn(page).click();
    expect((await refused).status()).toBe(401);

    const alert = page.getByRole("alert");
    await expect(alert).toContainText("The email or password is not correct.");
    await expect(alert).not.toContainText("invalid credentials");
    await expect(emailField(page)).toHaveValue(USERS.member.email);
    await expect(passwordField(page)).toHaveValue("");
    await expect(passwordField(page)).toBeFocused();
    expect(new URL(page.url()).pathname).toBe("/login/");

    // An unknown email reads exactly the same.
    await page.waitForTimeout(1_100);
    await emailField(page).fill(`nobody-${Date.now()}@example.test`);
    await passwordField(page).fill("not-the-password");
    await signIn(page).click();
    await expect(alert).toContainText("The email or password is not correct.");

    // The member's one-second penalty is over; signing in clears it.
    await page.waitForTimeout(1_100);
    await page.goto("/login/");
    await signInAsMember(page);
    await page.waitForURL((url) => url.pathname === "/");
  });

  test("throttling shows a decrementing countdown and keeps Sign in disabled until 0 (PB-14)", async ({
    page,
    networkGuard,
  }) => {
    test.setTimeout(60_000);
    const email = `throttle-${Date.now()}@example.test`;
    // Server penalty: 2^(failures-1) s. Three spaced failures leave 4 s.
    const api = await newApi();
    for (const pause of [1_100, 2_100, 0]) {
      const response = await api.post(LOGIN, {
        data: { email, password: "wrong" },
      });
      expect(response.status()).toBe(401);
      if (pause) await page.waitForTimeout(pause);
    }
    await api.dispose();

    await page.goto("/login/");
    await emailField(page).fill(email);
    await passwordField(page).fill("wrong");
    const throttled = page.waitForResponse(
      (r) => new URL(r.url()).pathname === LOGIN,
    );
    await signIn(page).click();
    expect((await throttled).status()).toBe(429);

    const countdown = page
      .getByRole("main")
      .getByText(/Too many sign-in attempts\. Try again in \d+:\d\d\./);
    await expect(countdown).toBeVisible();
    await expect(signIn(page)).toBeDisabled();
    await expect(page.getByRole("status")).toContainText(
      "Too many sign-in attempts. You can try again in",
    );

    const seconds = async () =>
      Number(/in \d+:(\d\d)/.exec((await countdown.textContent()) ?? "")?.[1]);
    const first = await seconds();
    expect(first).toBeGreaterThanOrEqual(2);
    await expect.poll(seconds, { timeout: 3_000 }).toBeLessThan(first);
    // Still disabled while any time remains; Enter sends nothing either.
    await expect(signIn(page)).toBeDisabled();
    const loginPosts = () =>
      networkGuard.requests.filter((r) => new URL(r.url).pathname === LOGIN)
        .length;
    const before = loginPosts();
    await passwordField(page).press("Enter");
    await page.waitForTimeout(300);
    expect(loginPosts()).toBe(before);

    await expect(countdown).toHaveCount(0, { timeout: 10_000 });
    await expect(signIn(page)).toBeEnabled();
    await expect(page.getByRole("status")).toHaveText("You can sign in again.");
    await expect(emailField(page)).toHaveValue(email);
  });

  test("next=//evil.com lands on / after sign-in (SC-003)", async ({
    page,
  }) => {
    await page.goto("/login/?next=//evil.com");
    await signInAsMember(page);
    await page.waitForURL((url) => url.pathname === "/");
    await expect(page).toHaveURL(/^http:\/\/localhost:\d+\/$/);
  });

  test("next=/player/?id=N lands on that player (SC-003)", async ({ page }) => {
    const bodyScan = readSeed().programs.withCover.meditations.find(
      (meditation) => meditation.name === "Body Scan",
    );
    expect(bodyScan).toBeDefined();
    await page.goto(`/login/?next=/player/?id=${bodyScan?.id}`);
    await signInAsMember(page);
    await page.waitForURL(
      (url) =>
        url.pathname === "/player/" &&
        url.searchParams.get("id") === String(bodyScan?.id),
    );
  });

  test("the Setup link shows only when setup is required (FR-082)", async ({
    page,
  }) => {
    await page.goto("/login/");
    await expect(emailField(page)).toBeEditable();
    await expect(page.getByRole("link", { name: "Go to Setup" })).toHaveCount(
      0,
    );

    await page.route("**/api/identity/setup-status", (route) =>
      route.fulfill({ json: { setup_required: true } }),
    );
    await page.reload();
    await expect(
      page.getByRole("link", { name: "Go to Setup" }),
    ).toHaveAttribute("href", "/setup/");
  });

  test("help text names the reset-password command (PB-14)", async ({
    page,
  }) => {
    await page.goto("/login/");
    const help = page.getByText("Forgot your password?");
    await expect(help).toContainText(
      "Ask the host operator to run medioss reset-password with your email.",
    );
    await expect(help.locator("code")).toHaveText("medioss reset-password");
  });

  test("a /me network error still shows the form", async ({ page }) => {
    await page.route("**/api/identity/me", (route) => route.abort("failed"));
    await page.goto("/login/");
    await expect(emailField(page)).toBeEditable();
    expect(new URL(page.url()).pathname).toBe("/login/");
  });

  for (const locale of ["en", "tr"] as const) {
    test(`screenshots (${locale})`, async ({ browser }) => {
      const context = await browser.newContext({
        ...localeOptions(locale),
        storageState: signedOut,
      });
      const page = await context.newPage();
      await page.goto("/login/");
      await expect(page.locator("main input")).toHaveCount(2);
      for (const width of [1440, 375] as const) {
        await page.setViewportSize(viewport(width));
        await saveScreenshot(page, "T15", `login-${locale}-${width}`);
      }
      await context.close();
    });
  }
});

test.describe("Turkish browser", () => {
  test.use({ storageState: signedOut, ...localeOptions("tr") });

  test("shows Turkish; the switcher changes text instantly and keeps the typed email (SC-022, FR-113)", async ({
    page,
  }) => {
    await page.goto("/login/");
    await expect(page.locator("h1")).toHaveText("Giriş yap");
    await expect(page.getByLabel("E-posta")).toBeEditable();
    await expect(page.getByLabel("Parola")).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("lang", "tr");

    await page.getByLabel("E-posta").fill("typed@example.test");
    await page.locator('label[lang="en"]').click();
    await expect(page.locator("h1")).toHaveText("Sign in");
    await expect(emailField(page)).toHaveValue("typed@example.test");
    await expect(signIn(page)).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("lang", "en");
  });
});

test.describe("signed in", () => {
  test.use({ storageState: storageStatePath("member") });

  test("a visit to /login/ lands on / (FR-081)", async ({ page }) => {
    await page.goto("/login/");
    await page.waitForURL((url) => url.pathname === "/");
    expect(new URL(page.url()).search).toBe("");
  });

  test("a visit to /login/?next=/stats/ lands on /stats/", async ({ page }) => {
    await page.goto("/login/?next=%2Fstats%2F");
    await page.waitForURL((url) => url.pathname === "/stats/");
  });
});
