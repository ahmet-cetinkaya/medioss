// T16: Home on an instance with admin and member and no content (FR-028,
// FR-110, SC-007): every region shows an empty state, with its action.
import type { BrowserContext, Page } from "@playwright/test";
import { ensureUsers, loginApi, USERS } from "../support/auth";
import { saveScreenshot } from "../support/evidence";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

/** A session for this context only; independent of users.e2e.ts order. */
async function signIn(
  context: BrowserContext,
  role: "admin" | "member",
): Promise<void> {
  const api = await loginApi(USERS[role].email, USERS[role].password);
  await context.addCookies((await api.storageState()).cookies);
  await api.dispose();
}

const HOST_MESSAGE = "The host has not added meditations yet";
const CONTENT_REGIONS = ["Continue", "Today's suggestion", "Recent programs"];

const region = (page: Page, name: string) =>
  page.getByRole("region", { name, exact: true });

/** Mood entries may exist from other suites on this shared instance. */
async function emptyMood(page: Page): Promise<void> {
  await page.route(
    (url) => url.pathname === "/api/mood",
    (route) =>
      route.request().method() === "GET"
        ? route.fulfill({ json: { items: [], next_cursor: null } })
        : route.continue(),
  );
}

test.use({ storageState: { cookies: [], origins: [] } });

test.beforeAll(async () => {
  await ensureUsers();
});

async function expectMoodEmpty(page: Page): Promise<void> {
  const mood = region(page, "Recent mood");
  await expect(
    mood.getByRole("heading", { name: "No check-ins yet" }),
  ).toBeVisible();
  await mood.getByRole("button", { name: "Check in now" }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByRole("radio", { name: /3/ })).toBeChecked();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(mood.getByRole("link", { name: "Open Mood" })).toHaveAttribute(
    "href",
    "/mood/",
  );
}

test("admin: every region is empty with an action; the Admin link is offered", async ({
  page,
  context,
}) => {
  await signIn(context, "admin");
  await emptyMood(page);
  await page.goto("/");
  for (const name of CONTENT_REGIONS) {
    const section = region(page, name);
    await expect(
      section.getByRole("heading", { name: HOST_MESSAGE }),
    ).toBeVisible();
    await expect(
      section.getByRole("link", { name: "Go to Admin" }),
    ).toHaveAttribute("href", "/admin/");
    await expect(
      section.getByRole("link", { name: "Go to Library" }),
    ).toHaveCount(0);
  }
  await expectMoodEmpty(page);
  await expect(page.locator("[data-elevation=featured]")).toHaveCount(1);
  await expect(page.getByRole("button", { name: "Retry" })).toHaveCount(0);

  // The Library preview also shows the host message with the Admin link.
  const library = region(page, "Library");
  await expect(
    library.getByRole("heading", { name: HOST_MESSAGE }),
  ).toBeVisible();
  await expect(
    library.getByRole("link", { name: "Go to Admin" }),
  ).toHaveAttribute("href", "/admin/");
});

test("member: the host message without any Admin link; mood invites a first check-in", async ({
  page,
  context,
}) => {
  await signIn(context, "member");
  await emptyMood(page);
  await page.goto("/");
  for (const name of [...CONTENT_REGIONS, "Library"]) {
    const section = region(page, name);
    await expect(
      section.getByRole("heading", { name: HOST_MESSAGE }),
    ).toBeVisible();
    await expect(section.getByRole("link")).toHaveCount(
      // "See all" only on the panels that carry one.
      name === "Recent programs" || name === "Library" ? 1 : 0,
    );
  }
  await expect(page.locator('a[href^="/admin"]')).toHaveCount(0);
  await expectMoodEmpty(page);

  for (const width of [1440, 375] as const) {
    await page.setViewportSize(viewport(width));
    await saveScreenshot(page, "T16", `home-empty-member-en-${width}`);
  }
});
