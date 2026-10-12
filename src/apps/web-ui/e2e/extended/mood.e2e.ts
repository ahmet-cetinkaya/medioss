// T21: the Mood screen for a member without entries, signed out, and on
// server errors (US4, FR-059, SC-005). Runs before the seed.
import type { Page } from "@playwright/test";
import {
  ensureUsers,
  saveStorageState,
  storageStatePath,
} from "../support/auth";
import { expect, test } from "../support/test-fixture";

const insightsRegion = (page: Page) =>
  page.getByRole("region", { name: "Insights" });
const entriesRegion = (page: Page) =>
  page.getByRole("region", { name: "Entries" });

// This file sorts before users.e2e.ts: make sure the member and its storage
// state exist (idempotent).
test.beforeAll(async () => {
  await ensureUsers();
  await saveStorageState("member");
});

test.describe("member without entries", () => {
  test.use({ storageState: storageStatePath("member") });

  test("no data: no chart, empty entries, and the action focuses the score selector (FR-059)", async ({
    page,
  }) => {
    const statistics = page.waitForResponse(
      (r) => new URL(r.url()).pathname === "/api/mood/statistics" && r.ok(),
    );
    await page.goto("/mood/");
    expect(await (await statistics).json()).toMatchObject({ entry_count: 0 });

    const insights = insightsRegion(page);
    await expect(
      insights.getByRole("heading", { name: "No mood entries in this range" }),
    ).toBeVisible();
    await expect(insights.locator("svg")).toHaveCount(0);
    await expect(page.locator("main svg circle")).toHaveCount(0);
    await expect(
      entriesRegion(page).getByRole("heading", {
        name: "No entries in this range",
      }),
    ).toBeVisible();
    await expect(page.getByRole("radio", { name: "All time" })).toBeChecked();

    await insights.getByRole("button", { name: "Choose a score" }).click();
    await expect(
      page.getByRole("group", { name: "Mood score" }).getByRole("radio", {
        name: /^3/,
      }),
    ).toBeFocused();
  });

  for (const target of ["statistics", "list"] as const) {
    test(`${target} 500 shows the error with Retry, not login (SC-005)`, async ({
      page,
    }) => {
      const pattern =
        target === "statistics" ? "**/api/mood/statistics?*" : "**/api/mood?*";
      await page.route(pattern, (route) =>
        route.fulfill({ status: 500, json: { error: "simulated" } }),
      );
      await page.goto("/mood/");
      const region =
        target === "statistics" ? insightsRegion(page) : entriesRegion(page);
      const retry = region.getByRole("button", { name: "Retry" });
      await expect(retry).toBeVisible();
      await expect(region).toContainText("Something went wrong on the server");
      await expect(page.locator("main")).not.toContainText("simulated");
      expect(new URL(page.url()).pathname).toBe("/mood/");

      await page.unroute(pattern);
      await retry.click();
      await expect(
        insightsRegion(page).getByRole("heading", {
          name: "No mood entries in this range",
        }),
      ).toBeVisible();
      await expect(
        entriesRegion(page).getByRole("heading", {
          name: "No entries in this range",
        }),
      ).toBeVisible();
      expect(new URL(page.url()).pathname).toBe("/mood/");
    });
  }
});

test.describe("signed out", () => {
  test.use({ storageState: { cookies: [], origins: [] } });

  test("opening /mood/ shows login with next=/mood/ (US4)", async ({
    page,
  }) => {
    await page.goto("/mood/");
    await page.waitForURL("**/login/**");
    const url = new URL(page.url());
    expect(url.pathname).toBe("/login/");
    expect(url.search).toBe("?next=%2Fmood%2F");
    await expect(page.getByLabel("Password")).toBeVisible();
  });
});
