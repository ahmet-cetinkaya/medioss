// T22: History on an instance with no content (FR-062): the empty state names
// the active range and notes-only filter and offers Library and Mood.
import {
  ensureUsers,
  saveStorageState,
  storageStatePath,
} from "../support/auth";
import { localeOptions } from "../support/locale";
import { pickPreset } from "../support/range-popover";
import { expect, test } from "../support/test-fixture";

// This file sorts before users.e2e.ts: make sure the member and its storage
// state exist (idempotent; creates no content).
test.beforeAll(async () => {
  await ensureUsers();
  await saveStorageState("member");
});

const region = (page: import("@playwright/test").Page) =>
  page.getByRole("region", { name: /^(Timeline|Zaman çizelgesi)$/ });

test.describe("member, no events", () => {
  test.use({ storageState: storageStatePath("member") });

  test("empty state names range and notes-only, with Library and Mood links", async ({
    page,
  }) => {
    const served = page.waitForResponse(
      (r) => new URL(r.url()).pathname === "/api/timeline",
    );
    await page.goto("/history/");
    const body = await (await served).json();
    expect(body).toEqual({ items: [], next_cursor: null });

    const timeline = region(page);
    await expect(
      timeline.getByRole("heading", { name: "No history for All time" }),
    ).toBeVisible();
    await expect(timeline).toContainText("Filter: All time, all events.");
    await expect(
      timeline.getByRole("link", { name: "Go to Library" }),
    ).toHaveAttribute("href", "/library/");
    await expect(
      timeline.getByRole("link", { name: "Go to Mood" }),
    ).toHaveAttribute("href", "/mood/");
    await expect(page.getByRole("button", { name: "Load more" })).toHaveCount(
      0,
    );

    await page.locator("main label").filter({ hasText: "Notes only" }).click();
    await expect(
      page.getByRole("checkbox", { name: "Notes only" }),
    ).toBeChecked();
    await expect(timeline).toContainText("Filter: All time, notes only.");

    await pickPreset(page, "Today");
    await expect(
      timeline.getByRole("heading", { name: "No history for Today" }),
    ).toBeVisible();
    await expect(timeline).toContainText("Filter: Today, notes only.");
  });
});

test.describe("member, no events (tr)", () => {
  test.use({
    storageState: storageStatePath("member"),
    ...localeOptions("tr"),
  });

  test("the Turkish empty state names the filter", async ({ page }) => {
    await page.goto("/history/");
    const timeline = region(page);
    await expect(
      timeline.getByRole("heading", { name: "Tüm zamanlar için geçmiş yok" }),
    ).toBeVisible();
    await expect(timeline).toContainText("Filtre: Tüm zamanlar, tüm olaylar.");
    await expect(
      timeline.getByRole("link", { name: "Kütüphaneye git" }),
    ).toHaveAttribute("href", "/library/");
    await expect(
      timeline.getByRole("link", { name: "Ruh hâline git" }),
    ).toHaveAttribute("href", "/mood/");
  });
});
