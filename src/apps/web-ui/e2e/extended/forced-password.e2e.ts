// T25: the forced password change (SC-016, FR-105–FR-107).
import { newApi, postJson } from "../support/api-client";
import { storageStatePath } from "../support/auth";
import { expect, test } from "../support/test-fixture";

const signedOut = { cookies: [], origins: [] };

test.use({ storageState: signedOut });

test("a flagged user is held on the change-password view at every address, then released (SC-016)", async ({
  page,
  networkGuard,
}) => {
  test.setTimeout(120_000);
  const email = `t25-${Date.now()}@example.test`;
  const temporary = "t25-e2e-temp-1";
  const next = "t25-e2e-new-pass-1";
  const admin = await newApi(storageStatePath("admin"));
  await postJson(admin, "/api/identity/users", { email, password: temporary });
  await admin.dispose();

  const heading = page.getByRole("heading", {
    level: 1,
    name: "Change password",
  });
  await page.goto("/login/");
  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Password").fill(temporary);
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(heading).toBeVisible();
  await expect(page.locator("nav")).toHaveCount(0);

  // Three other addresses: the view appears in place, no redirect loop.
  for (const address of ["/library/", "/mood/", "/settings/"]) {
    const start = networkGuard.requests.length;
    await page.goto(address);
    await expect(heading).toBeVisible();
    await expect(page.locator("nav")).toHaveCount(0);
    expect(new URL(page.url()).pathname).toBe(address);
    await page.waitForTimeout(500);
    const apiCalls = networkGuard.requests
      .slice(start)
      .filter((r) => new URL(r.url).pathname.startsWith("/api/"));
    expect(
      apiCalls.length,
      `${address}: ${apiCalls.map((r) => r.url).join(", ")}`,
    ).toBeLessThanOrEqual(2);
  }

  await page.getByLabel("New password", { exact: true }).fill(next);
  await page.getByLabel("Confirm new password").fill(next);
  const changed = page.waitForResponse("**/api/identity/me/password");
  await page.getByRole("button", { name: "Change password" }).click();
  expect((await changed).status()).toBe(200);

  // Released: the requested screen renders at the same URL, nav is back.
  await expect(
    page.getByRole("heading", { level: 1, name: "Settings" }),
  ).toBeVisible();
  expect(new URL(page.url()).pathname).toBe("/settings/");
  await expect(page.locator("nav:visible")).toHaveCount(1);
  const me = await page.request.get("/api/identity/me");
  expect(me.status()).toBe(200);
  expect((await me.json()).must_change_password).toBe(false);
});
