// T18: the Program screen (US3, FR-012, FR-038–FR-042, PB-18, PB-19,
// SC-005, SC-009 item 4).
import type { Page } from "@playwright/test";
import { storageStatePath } from "../support/auth";
import { saveJson, saveScreenshot } from "../support/evidence";
import { localeOptions } from "../support/locale";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

const PROGRAM_API = "**/api/meditation/programs/**";

const hero = (page: Page) => page.locator('main [data-elevation="featured"]');
const sessions = (page: Page) =>
  page.getByRole("region", { name: /^(Sessions|Seanslar)$/ });

for (const locale of ["en", "tr"] as const) {
  test.describe(`member (${locale})`, () => {
    test.use({
      storageState: storageStatePath("member"),
      ...localeOptions(locale),
    });

    test("covered program: hero with cover, sessions in order; screenshots", async ({
      page,
    }) => {
      const program = readSeed().programs.withCover;
      await page.goto(`/program/?id=${program.id}`);

      // SC-009 item 4 / FR-038: the cover is in the single featured hero.
      await expect(hero(page)).toHaveCount(1);
      await expect(
        hero(page).locator(`img[src*="/programs/${program.id}/cover"]`),
      ).toHaveCount(1);
      await expect(
        hero(page).getByRole("heading", { level: 2, name: program.title }),
      ).toBeVisible();
      await expect(hero(page)).toContainText(
        locale === "en"
          ? /Completed|\d+% complete/
          : /Tamamlandı|%\d+ tamamlandı/,
      );
      await expect(page.locator('main [data-variant="primary"]')).toHaveCount(
        1,
      );
      await expect(
        hero(page).locator('a[data-variant="primary"]'),
      ).toHaveAttribute("href", /^\/player\/\?id=\d+$/);

      // FR-040: program order, each opening the Player for that meditation.
      const items = sessions(page).getByRole("listitem");
      await expect(items).toHaveCount(program.meditations.length);
      for (const [index, meditation] of program.meditations.entries()) {
        const item = items.nth(index);
        await expect(item).toContainText(meditation.name);
        // Name, duration and the meditation's own progress label (PB-19).
        await expect(item).toContainText(
          locale === "en" ? /Completed|\d+%/ : /Tamamlandı|%\d+/,
        );
        const link = item.getByRole("link", {
          name:
            locale === "en"
              ? `Play ${meditation.name}`
              : `Oynat: ${meditation.name}`,
        });
        await expect(link).toHaveAttribute(
          "href",
          `/player/?id=${meditation.id}`,
        );
      }

      for (const width of [1440, 375] as const) {
        await page.setViewportSize(viewport(width));
        await saveScreenshot(page, "T18", `program-${locale}-${width}`);
      }
    });
  });
}

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  test("a program without a cover shows artwork, no cover image", async ({
    page,
  }) => {
    const program = readSeed().programs.noCover;
    await page.goto(`/program/?id=${program.id}`);
    await expect(
      hero(page).getByRole("heading", { name: program.title }),
    ).toBeVisible();
    await expect(hero(page).locator("img")).toHaveCount(0);
  });

  test("sessions are a horizontal row with no page overflow at 320px", async ({
    page,
  }) => {
    const program = readSeed().programs.withCover;
    await page.setViewportSize(viewport(320));
    await page.goto(`/program/?id=${program.id}`);
    const list = sessions(page).getByRole("list");
    await expect(list).toBeVisible();
    const flex = await list.evaluate((el) => getComputedStyle(el).display);
    expect(flex).toBe("flex");
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth > window.innerWidth,
    );
    expect(overflow).toBe(false);
  });

  test("all completed → Play again on the first (FR-039)", async ({ page }) => {
    const program = readSeed().programs.withCover;
    await page.route(`**/api/meditation/programs/${program.id}/members`, (r) =>
      r.fulfill({
        json: program.meditations.map((m) => ({
          id: m.id,
          name: m.name,
          content_type: "audio/mpeg",
          size_bytes: 1,
          duration_ms: 60_000,
          created_at: "2026-01-01T00:00:00Z",
          completed: true,
          progress_percent: 100,
        })),
      }),
    );
    await page.goto(`/program/?id=${program.id}`);
    const primary = hero(page).getByRole("link", { name: "Play again" });
    await expect(primary).toHaveAttribute(
      "href",
      `/player/?id=${program.meditations[0].id}`,
    );
  });

  test("an empty program: empty state and the primary action disabled with its reason (FR-041)", async ({
    page,
  }) => {
    const program = readSeed().programs.empty;
    await page.route(`**/api/meditation/programs/${program.id}/members`, (r) =>
      r.fulfill({ json: [] }),
    );
    await page.goto(`/program/?id=${program.id}`);
    await expect(
      sessions(page).getByRole("heading", { name: "No sessions yet" }),
    ).toBeVisible();
    const primary = hero(page).getByRole("button", { name: "Start" });
    await expect(primary).toBeDisabled();
    await expect(primary).toHaveAccessibleDescription(
      "Nothing to play yet: this program has no sessions.",
    );
    await expect(hero(page).getByRole("link")).toHaveCount(0);
  });

  test("a delayed load shows the loading state and no progress value (FR-042)", async ({
    page,
  }) => {
    const program = readSeed().programs.withCover;
    let released = false;
    await page.route(`**/api/meditation/programs/${program.id}`, async (r) => {
      await new Promise((resolve) => setTimeout(resolve, 2_000));
      released = true;
      await r.continue();
    });
    await page.goto(`/program/?id=${program.id}`);
    const busy = page.locator('main [aria-busy="true"]');
    await expect(busy).toBeVisible();
    for (let sample = 0; sample < 3 && !released; sample++) {
      await expect(page.locator("main")).not.toContainText("% complete");
      await expect(page.locator("main")).not.toContainText("Completed");
      await expect(hero(page)).toHaveCount(0);
      await page.waitForTimeout(400);
    }
    await expect(hero(page)).toContainText(/Completed|\d+% complete/, {
      timeout: 10_000,
    });
    await expect(busy).toHaveCount(0);
  });

  test("a missing, non-numeric, or zero id opens Library without a program request (PB-18)", async ({
    page,
  }) => {
    const programRequests: string[] = [];
    page.on("request", (request) => {
      // Only requests the Program page itself sent (the referer is /program/).
      if (
        request.url().includes("/api/meditation/programs") &&
        (request.headers().referer ?? "").includes("/program/")
      )
        programRequests.push(request.url());
    });
    const results: { address: string; landed: string; requests: number }[] = [];
    for (const address of ["/program/?id=abc", "/program/?id=0", "/program/"]) {
      const before = programRequests.length;
      await page.goto(address);
      await page.waitForURL("**/library/**");
      const landed = new URL(page.url()).pathname;
      expect(landed).toBe("/library/");
      const own = programRequests.slice(before);
      results.push({ address, landed, requests: own.length });
      expect(own).toEqual([]);
    }
    saveJson("T18", "invalid-id-request-count", results);
  });

  test("an unknown id shows not found with a Library link, not login (FR-012, SC-005)", async ({
    page,
  }) => {
    await page.goto("/program/?id=99999");
    await expect(
      page.getByRole("heading", { level: 2, name: "Program not found" }),
    ).toBeVisible();
    await expect(
      page.locator("main").getByRole("link", { name: "Back to Library" }),
    ).toHaveAttribute("href", "/library/");
    expect(new URL(page.url()).pathname).toBe("/program/");
    await expect(hero(page)).toHaveCount(0);
  });

  for (const failure of ["500", "abort"] as const) {
    test(`program ${failure} shows the error with Retry, not login (SC-005)`, async ({
      page,
    }) => {
      const program = readSeed().programs.withCover;
      await page.route(PROGRAM_API, (route) =>
        failure === "abort"
          ? route.abort("failed")
          : route.fulfill({ status: 500, json: { error: "simulated" } }),
      );
      await page.goto(`/program/?id=${program.id}`);
      const retry = page.getByRole("button", { name: "Retry" });
      await expect(retry).toBeVisible();
      await expect(page.locator("main")).not.toContainText("simulated");
      expect(new URL(page.url()).pathname).toBe("/program/");

      await page.unroute(PROGRAM_API);
      await retry.click();
      await expect(
        hero(page).getByRole("heading", { name: program.title }),
      ).toBeVisible();
    });
  }
});
