// T14: the Statistics screen (US4, FR-063–FR-065, PB-11, SC-003/005/006 subsets).
import type { Page } from "@playwright/test";
import { storageStatePath, USERS } from "../support/auth";
import { saveScreenshot } from "../support/evidence";
import { localeOptions, NEW_YORK, type AppLocale } from "../support/locale";
import { pickPreset, rangeTrigger } from "../support/range-popover";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

type Statistics = {
  range: string;
  total_minutes: number;
  total_days: number;
  current_streak: number;
  longest_streak: number;
  calendar: { date: string; minutes: number }[];
};

const STATISTICS = "**/api/meditation/statistics?*";

const COPY = {
  en: {
    labels: [
      "Total minutes",
      "Total practice days",
      "Current streak",
      "Longest streak",
    ],
    minutes: (n: number) => `${n} minute${n === 1 ? "" : "s"}`,
    day: /^\d+ days?$/,
    minute: /^\d+ minutes?$/,
  },
  tr: {
    labels: [
      "Toplam dakika",
      "Toplam pratik günü",
      "Güncel seri",
      "En uzun seri",
    ],
    minutes: (n: number) => `${n} dakika`,
    day: /^\d+ gün$/,
    minute: /^\d+ dakika$/,
  },
} as const;

/** What the screen must show for a server calendar day: the same Y-M-D, in UTC. */
function calendarLabel(day: string, locale: AppLocale): string {
  const [year, month, date] = day.split("-").map(Number);
  return new Intl.DateTimeFormat(locale, {
    dateStyle: "medium",
    timeZone: "UTC",
  }).format(new Date(Date.UTC(year, month - 1, date)));
}

const daysList = (page: Page) =>
  page.getByRole("region", { name: /Practiced days|Pratik yapılan günler/ });

async function openStats(page: Page): Promise<Statistics> {
  const response = page.waitForResponse(
    (r) => r.url().includes("/api/meditation/statistics?") && r.ok(),
  );
  await page.goto("/stats/");
  return (await (await response).json()) as Statistics;
}

const EMPTY: Statistics = {
  range: "today",
  total_minutes: 0,
  total_days: 0,
  current_streak: 0,
  longest_streak: 0,
  calendar: [],
};

for (const locale of ["en", "tr"] as const) {
  test.describe(`member in New York (${locale})`, () => {
    test.use({
      storageState: storageStatePath("member"),
      ...localeOptions(locale, NEW_YORK),
    });

    test("the 23:30 New York completion shows under its local date (PB-11)", async ({
      page,
    }) => {
      const seed = readSeed();
      const served = await openStats(page);
      const days = daysList(page);

      const local = days.getByRole("listitem").filter({
        hasText: calendarLabel(seed.completion.localDate, locale),
      });
      await expect(local).toHaveCount(1);
      await expect(local).toContainText(
        COPY[locale].minutes(seed.completion.minutes),
      );
      // Never shifted to the UTC calendar day of the same instant.
      await expect(
        days.getByText(calendarLabel(seed.completion.utcDate, locale)),
      ).toHaveCount(0);

      // Every day the server reported is shown with the same Y-M-D and minutes.
      expect(served.calendar).toContainEqual({
        date: seed.completion.localDate,
        minutes: seed.completion.minutes,
      });
      await expect(days.getByRole("listitem")).toHaveCount(
        served.calendar.length,
      );
      for (const day of served.calendar) {
        await expect(
          days
            .getByRole("listitem")
            .filter({ hasText: calendarLabel(day.date, locale) }),
        ).toContainText(COPY[locale].minutes(day.minutes));
      }
    });

    test("figures show labels and units; screenshots", async ({ page }) => {
      const served = await openStats(page);
      await expect(page.locator("main dl dt")).toHaveText([
        ...COPY[locale].labels,
      ]);
      const values = page.locator("main dl dd");
      await expect(values.nth(0)).toHaveText(COPY[locale].minute);
      for (const index of [1, 2, 3])
        await expect(values.nth(index)).toHaveText(COPY[locale].day);
      await expect(values.nth(0)).toContainText(String(served.total_minutes));
      await expect(values.nth(1)).toContainText(String(served.total_days));

      for (const width of [1440, 375] as const) {
        await page.setViewportSize(viewport(width));
        await saveScreenshot(page, "T14", `stats-${locale}-${width}`);
      }
    });
  });
}

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  test("ranges switch; All time is the default", async ({ page }) => {
    await openStats(page);
    await expect(rangeTrigger(page)).toHaveAccessibleName(
      "Date range: All time",
    );

    for (const [label, range] of [
      ["Last 30 days", "last30"],
      ["Today", "today"],
      ["All time", "all"],
    ] as const) {
      const response = page.waitForResponse(
        (r) =>
          r.url().includes("/api/meditation/statistics?") &&
          new URL(r.url()).searchParams.get("range") === range,
      );
      await pickPreset(page, label);
      expect((await response).status()).toBe(200);
      await expect(rangeTrigger(page)).toHaveAccessibleName(
        `Date range: ${label}`,
      );
      await expect(page.locator("main dl dt")).toHaveCount(4);
      await expect(page.locator("main dl dd").first()).toHaveText(
        COPY.en.minute,
      );
    }
  });

  test("a range without practice shows zeros, the message, and a Library link (FR-065)", async ({
    page,
  }) => {
    await page.route(STATISTICS, (route) =>
      new URL(route.request().url()).searchParams.get("range") === "today"
        ? route.fulfill({ json: EMPTY })
        : route.continue(),
    );
    await openStats(page);
    await pickPreset(page, "Today");

    await expect(page.locator("main dl dd")).toHaveText([
      "0 minutes",
      "0 days",
      "0 days",
      "0 days",
    ]);
    const days = daysList(page);
    await expect(
      days.getByRole("heading", { name: "No practice in this range" }),
    ).toBeVisible();
    await expect(days.getByRole("listitem")).toHaveCount(0);
    const library = days.getByRole("link", { name: "Go to Library" });
    await expect(library).toHaveAttribute("href", "/library/");
  });

  for (const failure of ["404", "500", "abort"] as const) {
    test(`statistics ${failure} shows the error with Retry, not login (SC-005)`, async ({
      page,
    }) => {
      await page.route(STATISTICS, (route) =>
        failure === "abort"
          ? route.abort("failed")
          : route.fulfill({
              status: Number(failure),
              json: { error: "simulated" },
            }),
      );
      await page.goto("/stats/");
      const retry = page.getByRole("button", { name: "Retry" });
      await expect(retry).toBeVisible();
      await expect(page.locator("main")).toContainText(
        failure === "abort"
          ? "Could not reach the server"
          : failure === "500"
            ? "Something went wrong on the server"
            : "Could not load statistics",
      );
      await expect(page.locator("main")).not.toContainText("simulated");
      expect(new URL(page.url()).pathname).toBe("/stats/");

      await page.unroute(STATISTICS);
      await retry.click();
      await expect(page.locator("main dl dt")).toHaveCount(4);
      expect(new URL(page.url()).pathname).toBe("/stats/");
    });
  }

  test("a 3 s delay shows the loading state, never blank (SC-006)", async ({
    page,
  }) => {
    let release = 0;
    await page.route(STATISTICS, async (route) => {
      await new Promise((resolve) => setTimeout(resolve, 3_000));
      release = Date.now();
      await route.continue();
    });
    await page.goto("/stats/");
    const busy = page.locator('main [aria-busy="true"]');
    await expect(busy).toBeVisible();
    // Sample during the delay: the busy region stays visible with its space.
    for (let sample = 0; sample < 4; sample++) {
      if (release) break;
      await expect(busy).toBeVisible();
      const box = await busy.boundingBox();
      expect(box?.height ?? 0).toBeGreaterThan(100);
      await page.waitForTimeout(500);
    }
    await expect(page.locator("main dl dt")).toHaveCount(4, {
      timeout: 10_000,
    });
    await expect(busy).toHaveCount(0);
  });
});

test.describe("signed out", () => {
  test.use({ storageState: { cookies: [], origins: [] } });

  test("opening /stats/ shows login with next=/stats/ (SC-003)", async ({
    page,
  }) => {
    await page.goto("/stats/");
    await page.waitForURL("**/login/**");
    const url = new URL(page.url());
    expect(url.pathname).toBe("/login/");
    expect(url.search).toBe("?next=%2Fstats%2F");
    await expect(page.getByLabel("Password")).toBeVisible();
  });

  test("after signing in, /stats/ opens (SC-003)", async ({ page }) => {
    await page.goto("/stats/");
    await page.waitForURL("**/login/?next=%2Fstats%2F");
    await expect(page.getByLabel("Email")).toBeEditable();
    await page.getByLabel("Email").fill(USERS.member.email);
    await page.getByLabel("Password").fill(USERS.member.password);
    await page.getByRole("button", { name: "Sign in" }).click();
    await page.waitForURL("**/stats/");
    await expect(page.locator("main dl dt")).toHaveCount(4);
  });
});
