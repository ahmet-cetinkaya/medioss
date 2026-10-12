import type { Page, Request } from "@playwright/test";
import { storageStatePath } from "../support/auth";
import { readSeed, type Named } from "../support/seed-content";
import { baseURL } from "../support/server-env";
import { expect, test } from "../support/test-fixture";

const ORIGIN = new URL(baseURL()).origin;

function meditation(name: string): Named {
  const seed = readSeed();
  const meditations = [
    ...seed.programs.withCover.meditations,
    ...seed.programs.noCover.meditations,
  ];
  const found = meditations.find((item) => item.name === name);
  if (!found) throw new Error(`seed has no meditation "${name}"`);
  return found;
}

const playButton = (page: Page) =>
  page.getByRole("button", { name: "Play", exact: true });
const pauseButton = (page: Page) =>
  page.getByRole("button", { name: "Pause", exact: true });
const seekSlider = (page: Page) =>
  page.getByRole("slider", { name: "Playback position" });

async function openPlayer(page: Page, id: number): Promise<void> {
  await page.goto(`/player/?id=${id}`);
  await expect(seekSlider(page)).toBeVisible();
  await page.waitForFunction(
    () => (document.querySelector("audio")?.readyState ?? 0) >= 1,
  );
}

async function startPlaying(page: Page): Promise<void> {
  await playButton(page).click();
  await expect(pauseButton(page)).toBeVisible();
  await page.waitForFunction(() => {
    const audio = document.querySelector("audio");
    return !!audio && !audio.paused && audio.currentTime > 0.3;
  });
}

const isPlaybackState = (request: Request) =>
  request.method() === "POST" &&
  /\/api\/meditation\/\d+\/playback-state$/.test(request.url());

test.describe("stateful playback", () => {
  test.use({ storageState: storageStatePath("admin") });

  test("pause saves the playback position", async ({ page }) => {
    await openPlayer(page, meditation("Quiet Mind").id);
    await startPlaying(page);
    const saved = page.waitForResponse(
      (response) => isPlaybackState(response.request()) && response.ok(),
    );
    await pauseButton(page).click();
    const position = (await saved).request().postDataJSON() as {
      position_ms: number;
    };
    expect(position.position_ms).toBeGreaterThan(0);
    await expect(playButton(page)).toBeVisible();
  });

  test("closing the player sends a beacon that saves playback position", async ({
    page,
  }) => {
    const quietMind = meditation("Quiet Mind");
    const url = `/api/meditation/${quietMind.id}/playback-state`;
    const api = page.context().request;
    expect(
      (
        await api.post(url, {
          data: { position_ms: 1_000 },
          headers: { Origin: ORIGIN },
        })
      ).ok(),
    ).toBe(true);
    await page.route(`**${url}`, (route) =>
      route.request().method() === "POST" &&
      route.request().resourceType() === "fetch"
        ? route.abort()
        : route.continue(),
    );
    await openPlayer(page, quietMind.id);
    await startPlaying(page);
    await page.waitForFunction(
      () => (document.querySelector("audio")?.currentTime ?? 0) > 2,
    );
    await page.close({ runBeforeUnload: true });
    await expect
      .poll(async () => {
        const body = (await (await api.get(url)).json()) as {
          position_ms: number;
        };
        return body.position_ms;
      })
      .toBeGreaterThan(2_000);
  });

  test("seeking to the end records one completion", async ({ page }) => {
    const slowBreathing = meditation("Slow Breathing");
    await page.request.put(`/api/meditation/${slowBreathing.id}/completed`, {
      data: { completed: false },
      headers: { Origin: ORIGIN },
    });
    const completions: Request[] = [];
    page.on("request", (request) => {
      if (
        request.method() === "POST" &&
        request.url().endsWith(`/api/meditation/${slowBreathing.id}/completion`)
      )
        completions.push(request);
    });
    const completion = page.waitForResponse(
      (response) =>
        response.request().method() === "POST" &&
        response
          .url()
          .endsWith(`/api/meditation/${slowBreathing.id}/completion`) &&
        response.ok(),
    );
    await openPlayer(page, slowBreathing.id);
    await startPlaying(page);
    await seekSlider(page).focus();
    await page.keyboard.press("End");
    await completion;
    await expect.poll(() => completions.length).toBe(1);
    await expect(page.getByRole("dialog", { name: "Well done" })).toBeVisible();
  });
});
