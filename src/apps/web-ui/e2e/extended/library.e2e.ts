// T17: the Library on an instance with no content (FR-037, US3, SC-002).
import type { BrowserContext } from "@playwright/test";
import { ensureUsers, loginApi, USERS } from "../support/auth";
import { expect, test } from "../support/test-fixture";

/** A session for this context only; independent of users.e2e.ts order. */
async function signIn(
  context: BrowserContext,
  role: "admin" | "member",
): Promise<void> {
  const api = await loginApi(USERS[role].email, USERS[role].password);
  await context.addCookies((await api.storageState()).cookies);
  await api.dispose();
}

test.use({ storageState: { cookies: [], origins: [] } });

test.beforeAll(async () => {
  await ensureUsers();
});

test("member with an empty library sees the host message and no Admin link anywhere", async ({
  page,
  context,
}) => {
  await signIn(context, "member");
  await page.goto("/library/");
  const programs = page.getByRole("region", { name: "Programs" });
  await expect(
    programs.getByRole("heading", { name: "No programs yet" }),
  ).toBeVisible();
  await expect(programs).toContainText(
    "The host has not added any programs yet.",
  );
  await expect(programs.getByRole("radio")).toHaveCount(0);
  await expect(page.locator('a[href^="/admin"]')).toHaveCount(0);
  await expect(page.getByRole("link", { name: /Admin/ })).toHaveCount(0);
});

test("admin with an empty library sees the host message with the Admin link", async ({
  page,
  context,
}) => {
  await signIn(context, "admin");
  await page.goto("/library/");
  const programs = page.getByRole("region", { name: "Programs" });
  await expect(
    programs.getByRole("heading", { name: "No programs yet" }),
  ).toBeVisible();
  await expect(
    programs.getByRole("link", { name: "Add programs in Admin" }),
  ).toHaveAttribute("href", "/admin/?section=meditations");
});
