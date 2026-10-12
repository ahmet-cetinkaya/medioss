// T25: the signed-out gate (SC-003): every authenticated address redirects to
// /login/ with `next`, and reopens at that address after sign-in.
import type { Page } from "@playwright/test";
import { USERS } from "../support/auth";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";

const signedOut = { cookies: [], origins: [] };

function addresses(): string[] {
  const seed = readSeed();
  const meditationId = seed.programs.withCover.meditations[0]?.id;
  expect(meditationId).toBeDefined();
  return [
    "/",
    "/library/",
    `/program/?id=${seed.programs.withCover.id}`,
    `/player/?id=${meditationId}`,
    "/soundscape/",
    "/mood/",
    "/history/",
    "/stats/",
    "/settings/",
    "/admin/",
  ];
}

async function signIn(page: Page, email: string, password: string) {
  await expect(page.getByLabel("Email")).toBeEditable();
  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Password").fill(password);
  await page.getByRole("button", { name: "Sign in" }).click();
}

test.describe("signed-out gate", () => {
  test.use({ storageState: signedOut });

  test("all 10 authenticated addresses redirect to /login/ with next and reopen after sign-in (SC-003)", async ({
    page,
    context,
  }) => {
    test.setTimeout(240_000);
    const list = addresses();
    expect(list).toHaveLength(10);
    for (const address of list) {
      await context.clearCookies();
      await page.goto(address);
      await page.waitForURL((url) => url.pathname === "/login/");
      expect(new URL(page.url()).searchParams.get("next"), address).toBe(
        address,
      );

      await signIn(page, USERS.member.email, USERS.member.password);
      await page.waitForURL((url) => url.pathname + url.search === address, {
        timeout: 15_000,
      });
      expect((await page.request.get("/api/identity/me")).status()).toBe(200);
    }
  });

  test("a crafted next=https://evil.com lands on / after sign-in (SC-003)", async ({
    page,
  }) => {
    await page.goto("/login/?next=https://evil.com");
    await signIn(page, USERS.member.email, USERS.member.password);
    await page.waitForURL((url) => url.pathname === "/");
    await expect(page).toHaveURL(/^http:\/\/localhost:\d+\/$/);
  });

  test("/admin/ after member sign-in shows the admins-only state, not a redirect", async ({
    page,
  }) => {
    await page.goto("/admin/");
    await page.waitForURL((url) => url.pathname === "/login/");
    await signIn(page, USERS.member.email, USERS.member.password);
    await page.waitForURL((url) => url.pathname === "/admin/");
    await expect(
      page.getByRole("heading", { name: "Admins only" }),
    ).toBeVisible();
    expect(new URL(page.url()).pathname).toBe("/admin/");
  });

  test("/admin/ after admin sign-in shows the Admin screen", async ({
    page,
  }) => {
    await page.goto("/admin/");
    await page.waitForURL((url) => url.pathname === "/login/");
    await signIn(page, USERS.admin.email, USERS.admin.password);
    await page.waitForURL((url) => url.pathname === "/admin/");
    await expect(
      page.getByRole("heading", { level: 1, name: "Admin" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Admins only" }),
    ).toHaveCount(0);
  });
});
