import type { Page } from "@playwright/test";
import { storageStatePath } from "../support/auth";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";

const mixPanel = (page: Page) =>
  page.getByRole("region", { name: "Current mix" });
const soundsPanel = (page: Page) =>
  page.getByRole("region", { name: "Sounds" });
const weather = (page: Page) =>
  soundsPanel(page).getByRole("region", { name: "Weather", exact: true });
const volume = (page: Page, soundId: number) =>
  page
    .locator(`audio[data-sound-id="${soundId}"]`)
    .evaluate((audio: HTMLAudioElement) => audio.volume);

test.describe("sleep timer", () => {
  test.use({ storageState: storageStatePath("member") });

  test("cancelling a timer restores the mix volume", async ({ page }) => {
    const rain = readSeed().sounds.rain.id;
    await page.clock.install();
    await page.goto("/soundscape/");
    await expect(
      weather(page).getByRole("button", { name: "Add Rain to mix" }),
    ).toBeVisible();
    await weather(page)
      .getByRole("button", { name: "Add Rain to mix" })
      .click();

    const panel = mixPanel(page);
    const slider = panel.getByRole("slider", { name: "Rain volume" });
    const timer = panel.getByRole("button", { name: "Sleep timer" });
    const cancel = panel.getByRole("button", { name: "Cancel timer" });
    await slider.fill("80");
    await panel.getByRole("button", { name: "Play mix" }).click();
    await expect(timer).toBeEnabled();
    await page.clock.pauseAt(new Date(Date.now() + 60_000));

    await timer.click();
    const dialog = page.getByRole("dialog", { name: "Sleep timer" });
    await dialog.getByLabel("Minutes until the mix stops").fill("5");
    await dialog.getByRole("button", { name: "Start timer" }).click();
    await page.clock.fastForward("00:01");
    await expect(dialog).toHaveCount(0);
    await page.clock.fastForward("04:29");
    await expect.poll(() => volume(page, rain)).toBeCloseTo(0.4);

    await cancel.click();
    await expect.poll(() => volume(page, rain)).toBeCloseTo(0.8);
    await expect(timer).toBeFocused();
  });
});
