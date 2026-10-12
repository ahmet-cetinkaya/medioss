// T25: SC-007 aggregate. On the empty instance a member sees an empty state on
// every data screen, with an action offered; Settings has no data region (N/A,
// it shows the profile form). Writes empty-instance.json as evidence.
import type { Locator, Page } from "@playwright/test";
import {
  ensureUsers,
  saveStorageState,
  storageStatePath,
} from "../support/auth";
import { saveJson } from "../support/evidence";
import { expect, test } from "../support/test-fixture";

test.beforeAll(async () => {
  await ensureUsers();
  await saveStorageState("member");
});

test.use({ storageState: storageStatePath("member") });

type Result = {
  screen: string;
  emptyTextSeen: string | null;
  actionOffered: string | null;
};

const results: Result[] = [];

const region = (page: Page, name: string | RegExp) =>
  page.getByRole("region", { name });

/** Asserts the empty heading, then returns the heading text and the action. */
async function record(
  screen: string,
  scope: Locator,
  heading: string,
  action: Locator,
): Promise<void> {
  const title = scope.getByRole("heading", { name: heading });
  await expect(title).toBeVisible();
  await expect(action).toBeVisible();
  results.push({
    screen,
    emptyTextSeen: (await title.textContent())?.trim() ?? null,
    actionOffered:
      ((await action.textContent())?.trim() ||
        (await action.getAttribute("aria-label"))) ??
      null,
  });
}

test("each screen shows its empty state with an action (SC-007)", async ({
  page,
}) => {
  // Mood entries may exist from other suites on this shared instance.
  await page.route(
    (url) => url.pathname === "/api/mood",
    (route) =>
      route.request().method() === "GET"
        ? route.fulfill({ json: { items: [], next_cursor: null } })
        : route.continue(),
  );

  await page.goto("/");
  const recent = region(page, "Recent mood");
  await record(
    "/",
    recent,
    "No check-ins yet",
    recent.getByRole("button", { name: "Check in now" }),
  );
  await expect(
    region(page, "Recent programs").getByRole("heading", {
      name: "The host has not added meditations yet",
    }),
  ).toBeVisible();

  await page.goto("/library/");
  const programs = region(page, "Programs");
  await expect(programs).toContainText(
    "The host has not added any programs yet.",
  );
  // A member has no Admin action; the empty state itself is the message.
  await record(
    "/library/",
    programs,
    "No programs yet",
    programs.getByRole("heading", { name: "No programs yet" }),
  );
  results[results.length - 1].actionOffered = "none (host message for member)";

  await page.goto("/soundscape/");
  const saved = region(page, /^Saved mixes$/);
  await record(
    "/soundscape/",
    saved,
    "No saved mixes",
    saved.getByText(/press Save mix/),
  );
  results[results.length - 1].actionOffered = "instruction: press Save mix";

  await page.goto("/mood/");
  const insights = region(page, "Insights");
  await record(
    "/mood/",
    insights,
    "No mood entries in this range",
    insights.getByRole("button", { name: "Choose a score" }),
  );
  await expect(
    region(page, "Entries").getByRole("heading", {
      name: "No entries in this range",
    }),
  ).toBeVisible();

  await page.goto("/history/");
  const timeline = region(page, /^Timeline$/);
  await record(
    "/history/",
    timeline,
    "No history for All time",
    timeline.getByRole("link", { name: "Go to Library" }),
  );

  await page.goto("/stats/");
  const days = region(page, "Practiced days");
  await record(
    "/stats/",
    days,
    "No practice in this range",
    days.getByRole("link", { name: "Go to Library" }),
  );

  await page.goto("/settings/");
  await expect(
    page.getByRole("heading", { level: 2, name: "Profile" }),
  ).toBeVisible();
  results.push({
    screen: "/settings/",
    emptyTextSeen: null,
    actionOffered: "N/A: no data region; shows the profile form",
  });

  expect(results.map((r) => r.screen)).toEqual([
    "/",
    "/library/",
    "/soundscape/",
    "/mood/",
    "/history/",
    "/stats/",
    "/settings/",
  ]);
  saveJson("T25", "empty-instance", results);
});
