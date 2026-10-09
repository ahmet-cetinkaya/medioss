// T15: Setup on an empty instance (FR-083, FR-084, PB-15, SC-009 item 5).
// Runs first in the chain and completes setup through the UI with
// USERS.admin, so the `empty` project signs in with the same credentials.
// The setup secret comes from sessionSecret(); it is never logged, never
// asserted with a value-printing matcher, and only typed into a password field.
import type { Page } from "@playwright/test";
import { USERS } from "../support/auth";
import { saveJson, saveScreenshot } from "../support/evidence";
import { localeOptions } from "../support/locale";
import { sessionSecret } from "../support/server-env";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

const STATUS = "**/api/identity/setup-status";
const SETUP = "/api/identity/setup";

const secretField = (page: Page) => page.getByLabel("Setup secret");
const emailField = (page: Page) => page.getByLabel("Administrator email");
const passwordField = (page: Page) => page.getByLabel("Administrator password");
const submit = (page: Page) =>
  page.getByRole("button", { name: "Create administrator" });
const loginLink = (page: Page) =>
  page.getByRole("main").getByRole("link", { name: "Go to sign in" });

/** Main-area controls a user could operate (the language switcher is outside main). */
async function operableControls(page: Page): Promise<number> {
  return page
    .locator("main")
    .locator("input, button, textarea, select")
    .evaluateAll(
      (nodes) =>
        nodes.filter((node) => !(node as HTMLInputElement).disabled).length,
    );
}

/** Storage keys and whether any stored value contains `needle`. */
async function storageSnapshot(page: Page, needle: string) {
  return page.evaluate((secret) => {
    const entries = (storage: Storage) =>
      Array.from({ length: storage.length }, (_, index) => {
        const key = storage.key(index) ?? "";
        return [key, storage.getItem(key) ?? ""] as const;
      });
    const all = [...entries(localStorage), ...entries(sessionStorage)];
    return {
      keys: all.map(([key]) => key),
      leaked: all.some(
        ([key, value]) => key.includes(secret) || value.includes(secret),
      ),
      cookieLeaked: document.cookie.includes(secret),
    };
  }, needle);
}

async function expectSecretNowhere(page: Page): Promise<void> {
  const secret = sessionSecret();
  // Booleans only: a failing matcher must not print the secret.
  expect(await secretField(page).inputValue()).toBe("");
  expect((await page.content()).includes(secret)).toBe(false);
  const storage = await storageSnapshot(page, secret);
  expect(storage.leaked).toBe(false);
  expect(storage.cookieLeaked).toBe(false);
  for (const key of storage.keys) expect(["medioss.locale"]).toContain(key);
}

test("a 3 s delayed status shows loading with no operable form (FR-083, SC-009)", async ({
  page,
}) => {
  let released = false;
  await page.route(STATUS, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 3_000));
    released = true;
    await route.continue();
  });
  await page.goto("/setup/");
  await expect(page.locator('main [aria-busy="true"]')).toBeVisible();
  let samples = 0;
  while (!released) {
    expect(await operableControls(page)).toBe(0);
    await expect(secretField(page)).toHaveCount(0);
    samples++;
    await page.waitForTimeout(400);
  }
  expect(samples).toBeGreaterThanOrEqual(3);
  await expect(secretField(page)).toBeEditable();
  await expect(secretField(page)).toHaveAttribute("type", "password");
  await expect(secretField(page)).toHaveAttribute("autocomplete", "off");
});

test("a status failure shows the error with Retry and no form (FR-084)", async ({
  page,
}) => {
  await page.route(STATUS, (route) =>
    route.fulfill({ status: 500, json: { error: "simulated" } }),
  );
  await page.goto("/setup/");
  const retry = page.getByRole("button", { name: "Retry" });
  await expect(retry).toBeVisible();
  await expect(page.locator("main")).toContainText(
    "Something went wrong on the server",
  );
  await expect(page.locator("main")).not.toContainText("simulated");
  await expect(page.locator("main input")).toHaveCount(0);
  await expect(submit(page)).toHaveCount(0);

  await page.unroute(STATUS);
  await retry.click();
  await expect(secretField(page)).toBeEditable();
});

for (const locale of ["en", "tr"] as const) {
  test.describe(`screenshots (${locale})`, () => {
    test.use(localeOptions(locale));

    test("empty setup form", async ({ page }) => {
      await page.goto("/setup/");
      await expect(page.locator("main input")).toHaveCount(3);
      await expect(page.locator("h1")).toHaveText(
        locale === "en" ? "Setup" : "Kurulum",
      );
      for (const width of [1440, 375] as const) {
        await page.setViewportSize(viewport(width));
        await saveScreenshot(page, "T15", `setup-${locale}-${width}`);
      }
    });
  });
}

test("a too-short password is refused before any request; the secret is cleared", async ({
  page,
  networkGuard,
}) => {
  await page.goto("/setup/");
  await secretField(page).fill("not-the-secret");
  await emailField(page).fill(USERS.admin.email);
  await passwordField(page).fill("short");
  await submit(page).click();
  await expect(page.getByRole("alert")).toContainText("at least 8 characters");
  await expect(passwordField(page)).toBeFocused();
  await expect(secretField(page)).toHaveValue("");
  const posts = networkGuard.requests.filter(
    (r) => r.method === "POST" && new URL(r.url).pathname === SETUP,
  );
  saveJson("T15", "setup-too-short-requests", {
    claim: "a too-short admin password sends no setup request",
    setupPosts: posts.length,
    requests: networkGuard.requests.map((r) => `${r.method} ${r.url}`),
  });
  expect(posts).toHaveLength(0);
});

test("a wrong secret shows the forbidden reason and empties the secret field (PB-15)", async ({
  page,
}) => {
  await page.goto("/setup/");
  await secretField(page).fill("wrong-setup-secret");
  await emailField(page).fill(USERS.admin.email);
  await passwordField(page).fill(USERS.admin.password);
  const response = page.waitForResponse(
    (r) => new URL(r.url()).pathname === SETUP,
  );
  await submit(page).click();
  expect((await response).status()).toBe(403);
  await expect(page.getByRole("alert")).toContainText(
    "The setup secret is not correct, or this address is not the instance's configured origin.",
  );
  await expect(secretField(page)).toHaveValue("");
  await expect(emailField(page)).toHaveValue(USERS.admin.email);
  await expectSecretNowhere(page);
});

test("success shows the Login link on /setup/; a reload shows already set up (PB-15)", async ({
  page,
}) => {
  await page.goto("/setup/");
  await expect(secretField(page)).toBeEditable();
  await secretField(page).fill(sessionSecret());
  await emailField(page).fill(USERS.admin.email);
  await passwordField(page).fill(USERS.admin.password);
  const response = page.waitForResponse(
    (r) => new URL(r.url()).pathname === SETUP,
  );
  await submit(page).click();
  expect((await response).status()).toBe(201);

  await expect(
    page.getByText("The administrator account was created."),
  ).toBeVisible();
  await expect(loginLink(page)).toHaveAttribute("href", "/login/");
  await expect(loginLink(page)).toBeFocused();
  await expect(page.locator("main input")).toHaveCount(0);
  expect(new URL(page.url()).pathname).toBe("/setup/");
  // Nothing navigates on its own.
  await page.waitForTimeout(1_000);
  expect(new URL(page.url()).pathname).toBe("/setup/");

  const storage = await storageSnapshot(page, sessionSecret());
  expect(storage.leaked).toBe(false);
  expect(storage.cookieLeaked).toBe(false);
  expect((await page.content()).includes(sessionSecret())).toBe(false);
  for (const key of storage.keys) expect(["medioss.locale"]).toContain(key);

  await page.reload();
  await expect(
    page.getByText("This instance is already set up."),
  ).toBeVisible();
  await expect(loginLink(page)).toHaveAttribute("href", "/login/");
  await expect(page.locator("main input")).toHaveCount(0);
  await saveScreenshot(page, "T15", "setup-already-set-up-en-1440");
});
