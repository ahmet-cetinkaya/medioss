// T20: the Soundscape screen (US3, FR-049–FR-054, PB-6/PB-7/PB-8/PB-16,
// SC-008, SC-009 item 2) and the T10 dialog items it proves (FR-019/FR-020,
// focus fallback after a deleted row).
import type { Page } from "@playwright/test";
import { getJson, newApi, postJson, type Api } from "../support/api-client";
import { storageStatePath } from "../support/auth";
import { saveJson, saveScreenshot } from "../support/evidence";
import { localeOptions } from "../support/locale";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

type Mix = {
  id: number;
  name: string;
  items: { sound_id: number; volume: number; position: number }[];
};

const SOUNDS = (url: URL) => url.pathname === "/api/soundscape/sounds";
const MIXES = (url: URL) => url.pathname === "/api/soundscape/mixes";

const mixPanel = (page: Page) =>
  page.getByRole("region", { name: /^(Current mix|Geçerli karışım)$/ });
const savedPanel = (page: Page) =>
  page.getByRole("region", { name: /^(Saved mixes|Kayıtlı karışımlar)$/ });
const soundsPanel = (page: Page) =>
  page.getByRole("region", { name: /^(Sounds|Sesler)$/ });
// Seeded Rain and Wind live in "Weather"; built-in sound names (e.g. "wind")
// would otherwise match the same accessible names case-insensitively.
const weather = (page: Page) =>
  soundsPanel(page).getByRole("region", { name: "Weather", exact: true });
const audioOf = (page: Page, soundId: number) =>
  page.locator(`audio[data-sound-id="${soundId}"]`);
const paused = (page: Page, soundId: number) =>
  audioOf(page, soundId).evaluate((a: HTMLAudioElement) => a.paused);
const volume = (page: Page, soundId: number) =>
  audioOf(page, soundId).evaluate((a: HTMLAudioElement) => a.volume);

async function openSoundscape(page: Page): Promise<void> {
  await page.goto("/soundscape/");
  await expect(
    soundsPanel(page)
      .getByRole("button", { name: /^Add .+ to mix$/ })
      .first(),
  ).toBeVisible();
  await expect(
    savedPanel(page).locator('[data-region-state="loading"]'),
  ).toHaveCount(0);
}

async function memberApi(): Promise<Api> {
  return newApi(storageStatePath("member"));
}

async function createMix(name: string): Promise<Mix> {
  const seed = readSeed();
  const api = await memberApi();
  const mix = await postJson<Mix>(api, "/api/soundscape/mixes", {
    name,
    items: [{ sound_id: seed.sounds.rain.id, volume: 40, position: 0 }],
  });
  await api.dispose();
  return mix;
}

async function deleteMixQuietly(id: number): Promise<void> {
  const api = await memberApi();
  await api.delete(`/api/soundscape/mixes/${id}`);
  await api.dispose();
}

const unique = (prefix: string) =>
  `${prefix} ${Date.now().toString(36)}${Math.floor(Math.random() * 1e4)}`;

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  test("tiles: chips (All default), name, duration, add/remove label, download (FR-049, PB-16)", async ({
    page,
  }) => {
    const seed = readSeed();
    await openSoundscape(page);
    const sounds = soundsPanel(page);
    await expect(page.getByRole("radio", { name: "All" })).toBeChecked();
    await expect(
      sounds.getByRole("heading", { level: 3, name: "Weather" }),
    ).toBeVisible();
    // Built-in categories render too (many sounds, still grouped).
    expect(
      await sounds.getByRole("heading", { level: 3 }).count(),
    ).toBeGreaterThan(1);

    await page.locator("label").filter({ hasText: "Weather" }).click();
    await expect(page.getByRole("radio", { name: "Weather" })).toBeChecked();
    await expect(sounds.getByRole("heading", { level: 3 })).toHaveText([
      "Weather",
    ]);

    const tile = weather(page)
      .getByRole("listitem")
      .filter({ hasText: "Rain" });
    await expect(tile).toContainText(/\d+ (sec|second)/);
    const download = tile.getByRole("link", { name: "Download Rain" });
    await expect(download).toHaveAttribute(
      "href",
      `/api/soundscape/sounds/${seed.sounds.rain.id}/download`,
    );

    await tile.getByRole("button", { name: "Add Rain to mix" }).click();
    await expect(
      tile.getByRole("button", { name: "Remove Rain from mix" }),
    ).toBeVisible();
    await expect(tile).toContainText("In mix");
    await expect(
      mixPanel(page).getByRole("slider", { name: "Rain volume" }),
    ).toHaveValue("100");
    await tile.getByRole("button", { name: "Remove Rain from mix" }).click();
    await expect(
      tile.getByRole("button", { name: "Add Rain to mix" }),
    ).toBeVisible();
    // Only the overall volume slider remains.
    await expect(mixPanel(page).getByRole("slider")).toHaveCount(1);
  });

  test("load Evening: Rain 30, Wind 70; no autoplay; loading stops playback (SC-009, FR-051, PB-8)", async ({
    page,
  }) => {
    const seed = readSeed();
    await openSoundscape(page);
    const evening = savedPanel(page).getByRole("button", {
      name: "Load Evening",
    });
    await evening.click();
    const panel = mixPanel(page);
    await expect(
      panel.getByRole("slider", { name: "Rain volume" }),
    ).toHaveValue("30");
    await expect(
      panel.getByRole("slider", { name: "Wind volume" }),
    ).toHaveValue("70");
    await expect(
      panel.getByRole("slider", { name: "Rain volume" }),
    ).toHaveAttribute("aria-valuetext", "30 percent");
    await expect(panel.getByRole("button", { name: "Play mix" })).toBeVisible();
    expect(await paused(page, seed.sounds.rain.id)).toBe(true);
    expect(await paused(page, seed.sounds.wind.id)).toBe(true);
    expect(await volume(page, seed.sounds.rain.id)).toBeCloseTo(0.3);
    expect(await volume(page, seed.sounds.wind.id)).toBeCloseTo(0.7);
    await expect(panel).not.toContainText("no longer available");

    await panel.getByRole("button", { name: "Play mix" }).click();
    await expect(panel.getByRole("button", { name: "Stop" })).toBeVisible();
    expect(await paused(page, seed.sounds.rain.id)).toBe(false);

    await evening.click();
    await expect(panel.getByRole("button", { name: "Play mix" })).toBeVisible();
    expect(await paused(page, seed.sounds.rain.id)).toBe(true);
    expect(await paused(page, seed.sounds.wind.id)).toBe(true);
  });

  test("a mix with a deleted sound shows the unavailable warning (PB-8)", async ({
    page,
  }) => {
    const seed = readSeed();
    // The server drops a mix item when its sound is deleted, so the seeded
    // Broken Mix now holds only Rain. Mock the list so one item points at a
    // sound that no longer exists, as an older mix would.
    const missingId = 9_999_999;
    await page.route(MIXES, async (route) => {
      if (route.request().method() !== "GET") return route.continue();
      const response = await route.fetch();
      const mixes = (await response.json()) as Mix[];
      await route.fulfill({
        response,
        json: mixes.map((mix) =>
          mix.id === seed.mixes.broken.id
            ? {
                ...mix,
                items: [
                  ...mix.items,
                  { sound_id: missingId, volume: 50, position: 9 },
                ],
              }
            : mix,
        ),
      });
    });
    await openSoundscape(page);
    await savedPanel(page)
      .getByRole("button", { name: "Load Broken Mix" })
      .click();
    const panel = mixPanel(page);
    await expect(
      panel.getByText("Some sounds from this mix are no longer available."),
    ).toBeVisible();
    await expect(panel.getByRole("slider")).toHaveCount(2);
    await expect(
      panel.getByRole("slider", { name: "Rain volume" }),
    ).toHaveValue("50");
    // The remaining sound still plays (spec edge case).
    await panel.getByRole("button", { name: "Play mix" }).click();
    await expect(panel.getByRole("button", { name: "Stop" })).toBeVisible();

    // Loading a complete mix clears the warning.
    await savedPanel(page)
      .getByRole("button", { name: "Load Evening" })
      .click();
    await expect(panel).not.toContainText("no longer available");
  });

  test("the real Broken Mix loads its remaining sound (server dropped the deleted item)", async ({
    page,
  }) => {
    const seed = readSeed();
    expect(seed.mixes.broken.items.map((item) => item.sound_id)).toEqual([
      seed.sounds.rain.id,
    ]);
    await openSoundscape(page);
    await savedPanel(page)
      .getByRole("button", { name: "Load Broken Mix" })
      .click();
    await expect(
      mixPanel(page).getByRole("slider", { name: "Rain volume" }),
    ).toHaveValue("50");
  });

  test("play: disabled while empty; a sound added during play starts (PB-6)", async ({
    page,
  }) => {
    const seed = readSeed();
    await openSoundscape(page);
    const panel = mixPanel(page);
    const play = panel.getByRole("button", { name: "Play mix" });
    await expect(play).toBeDisabled();
    await expect(
      panel.getByRole("heading", { name: "Your mix is empty" }),
    ).toBeVisible();

    const sounds = weather(page);
    await sounds.getByRole("button", { name: "Add Rain to mix" }).click();
    await expect(play).toBeEnabled();
    await play.click();
    await expect(panel.getByRole("button", { name: "Stop" })).toBeVisible();
    expect(await paused(page, seed.sounds.rain.id)).toBe(false);

    await sounds.getByRole("button", { name: "Add Wind to mix" }).click();
    await expect.poll(() => paused(page, seed.sounds.wind.id)).toBe(false);
    expect(await audioOf(page, seed.sounds.wind.id).getAttribute("loop")).toBe(
      "",
    );

    // Removing every sound stops the mix and disables play again.
    await panel.getByRole("button", { name: "Remove Rain from mix" }).click();
    await panel.getByRole("button", { name: "Remove Wind from mix" }).click();
    await expect(
      panel.getByRole("button", { name: "Play mix" }),
    ).toBeDisabled();
  });

  test("a blocked autoplay shows the press-play message (PB-6)", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      HTMLMediaElement.prototype.play = () =>
        Promise.reject(new DOMException("blocked", "NotAllowedError"));
    });
    await openSoundscape(page);
    await weather(page)
      .getByRole("button", { name: "Add Rain to mix" })
      .click();
    const panel = mixPanel(page);
    await panel.getByRole("button", { name: "Play mix" }).click();
    await expect(panel.getByRole("alert")).toHaveText(
      /The browser blocked playback\. Press Play mix to start\./,
    );
    await expect(panel.getByRole("button", { name: "Play mix" })).toBeVisible();
  });

  test("sleep timer: only while playing, 5–120 whole minutes, m:ss live countdown, fade to 0 then stop, cancel restores (PB-7, D10)", async ({
    page,
  }) => {
    const seed = readSeed();
    const rain = seed.sounds.rain.id;
    await page.clock.install();
    await openSoundscape(page);
    await weather(page)
      .getByRole("button", { name: "Add Rain to mix" })
      .click();
    const panel = mixPanel(page);
    const timerButton = panel.getByRole("button", { name: "Sleep timer" });
    const cancel = panel.getByRole("button", { name: "Cancel timer" });
    const countdown = panel.locator('[aria-live="polite"][aria-atomic="true"]');
    const dialog = page.getByRole("dialog", { name: "Sleep timer" });
    const minutes = dialog.getByLabel("Minutes until the mix stops");
    const start = dialog.getByRole("button", { name: "Start timer" });
    await expect(timerButton).toBeDisabled();
    await expect(
      panel.getByText("The sleep timer is available while the mix plays."),
    ).toBeVisible();

    const slider = panel.getByRole("slider", { name: "Rain volume" });
    await slider.fill("80");
    await panel.getByRole("button", { name: "Play mix" }).click();
    await expect(panel.getByRole("button", { name: "Stop" })).toBeVisible();
    await expect(timerButton).toBeEnabled();
    // Freeze page time: the timer only moves when the test fast-forwards.
    await page.clock.pauseAt(new Date(Date.now() + 60_000));

    await timerButton.click();
    for (const bad of ["4", "121", "7.5"]) {
      await minutes.fill(bad);
      await expect(start).toBeDisabled();
      await expect(
        dialog.getByText("Enter whole minutes from 5 to 120."),
      ).toBeVisible();
      await expect(minutes).toHaveAttribute("aria-invalid", "true");
      await expect(countdown).toHaveText("");
    }

    await minutes.fill("5");
    await start.click();
    // The dialog's close cleanup runs on timers, which are frozen: advance
    // one second so it releases the page, then check the live countdown (m:ss).
    await page.clock.fastForward("00:01");
    await expect(dialog).toHaveCount(0);
    await expect(countdown).toHaveText(/^\s*Time left\s*4:59$/);

    // 30 s before the end: half the fade; the slider keeps its value.
    await page.clock.fastForward("04:29");
    await expect(countdown).toHaveText(/\s0:30$/);
    await expect.poll(() => volume(page, rain)).toBeCloseTo(0.4);
    await expect(slider).toHaveValue("80");

    // Saving during the fade stores the slider value, not the faded volume.
    const name = unique("Faded");
    await panel.getByRole("button", { name: "Save mix" }).click();
    const saveDialog = page.getByRole("dialog", { name: "Save mix" });
    await saveDialog.getByLabel("Mix name").fill(name);
    const posted = page.waitForRequest(
      (r) => MIXES(new URL(r.url())) && r.method() === "POST",
    );
    await saveDialog.getByRole("button", { name: "Save mix" }).click();
    const body = (await posted).postDataJSON() as Mix;
    expect(body.items).toEqual([{ sound_id: rain, volume: 80, position: 0 }]);
    await expect(
      savedPanel(page).getByRole("button", { name: `Load ${name}` }),
    ).toBeVisible();

    // Cancel restores the pre-fade volume. The frozen clock stalls the save
    // dialog's exit, so step it until the overlay is gone.
    await expect(async () => {
      await page.clock.fastForward(100);
      await expect(page.locator("[data-dialog-overlay]")).toHaveCount(0, {
        timeout: 200,
      });
      expect(
        await page.evaluate(
          () => getComputedStyle(document.body).pointerEvents,
        ),
      ).not.toBe("none");
    }).toPass();
    await cancel.click();
    await expect(countdown).toHaveText("");
    await expect.poll(() => volume(page, rain)).toBeCloseTo(0.8);
    await expect(timerButton).toBeFocused();
    expect(await paused(page, rain)).toBe(false);

    // Run a timer to the end: it fades to 0 and stops.
    await timerButton.click();
    await minutes.fill("5");
    await start.click();
    await expect(countdown).toHaveText(/\s5:00$/);
    await page.clock.fastForward("04:59");
    await expect(countdown).toHaveText(/\s0:01$/);
    await expect.poll(() => volume(page, rain)).toBeLessThan(0.02);
    await page.clock.fastForward("00:02");
    await expect(panel.getByRole("button", { name: "Play mix" })).toBeVisible();
    await expect(countdown).toHaveText("");
    expect(await paused(page, rain)).toBe(true);
    await expect(timerButton).toBeDisabled();

    const saved = await getJson<Mix[]>(
      await memberApi(),
      "/api/soundscape/mixes",
    );
    for (const mix of saved.filter((m) => m.name === name))
      await deleteMixQuietly(mix.id);
  });

  test("save requires a name and a sound and stores the slider values (PB-8)", async ({
    page,
  }) => {
    const seed = readSeed();
    await openSoundscape(page);
    const panel = mixPanel(page);
    const save = panel.getByRole("button", { name: "Save mix" });
    await expect(save).toBeDisabled();

    await weather(page)
      .getByRole("button", { name: "Add Rain to mix" })
      .click();
    await weather(page)
      .getByRole("button", { name: "Add Wind to mix" })
      .click();
    await panel.getByRole("slider", { name: "Rain volume" }).fill("25");
    await panel.getByRole("slider", { name: "Wind volume" }).fill("60");

    await save.click();
    const dialog = page.getByRole("dialog", { name: "Save mix" });
    const name = dialog.getByLabel("Mix name");
    const submit = dialog.getByRole("button", { name: "Save mix" });
    await expect(name).toHaveValue("");
    await expect(submit).toBeDisabled();
    await name.fill("   ");
    await expect(submit).toBeDisabled();

    const mixName = unique("Saved");
    await name.fill(mixName);
    await expect(submit).toBeEnabled();
    const posted = page.waitForResponse(
      (r) => MIXES(new URL(r.url())) && r.request().method() === "POST",
    );
    await submit.click();
    const created = (await (await posted).json()) as Mix;
    expect(created.items).toEqual([
      { sound_id: seed.sounds.rain.id, volume: 25, position: 0 },
      { sound_id: seed.sounds.wind.id, volume: 60, position: 1 },
    ]);
    await expect(dialog).toHaveCount(0);
    await expect(
      savedPanel(page).getByRole("button", { name: `Load ${mixName}` }),
    ).toBeVisible();
    expect(
      await page.evaluate(() => localStorage.getItem("medioss.lastMix")),
    ).toBe(String(created.id));
    await deleteMixQuietly(created.id);
  });

  test("rename prefills, refuses blank, and saves (FR-019, FR-052)", async ({
    page,
  }) => {
    const mix = await createMix(unique("Rename Me"));
    try {
      await openSoundscape(page);
      const opener = savedPanel(page).getByRole("button", {
        name: `Rename ${mix.name}`,
      });
      await opener.click();
      const dialog = page.getByRole("dialog", { name: "Rename mix" });
      const field = dialog.getByRole("textbox", { name: "Mix name" });
      await expect(field).toHaveValue(mix.name);
      const save = dialog.getByRole("button", { name: "Save" });

      await field.fill("   ");
      await expect(save).toBeDisabled();
      await field.press("Enter");
      await expect(dialog).toBeVisible();

      const renamed = unique("Renamed");
      await field.fill(`  ${renamed}  `);
      await expect(save).toBeEnabled();
      const put = page.waitForRequest((r) => r.method() === "PUT");
      await save.click();
      expect(((await put).postDataJSON() as Mix).name).toBe(renamed);
      await expect(dialog).toHaveCount(0);
      await expect(
        savedPanel(page).getByRole("button", { name: `Load ${renamed}` }),
      ).toBeVisible();
      await expect(
        savedPanel(page).getByRole("button", { name: `Rename ${renamed}` }),
      ).toBeFocused();
    } finally {
      await deleteMixQuietly(mix.id);
    }
  });

  test("delete: cancel keeps the mix; a double-pressed confirm sends one DELETE; focus falls back to the list heading (FR-018/FR-020/FR-053, SC-008)", async ({
    page,
  }) => {
    const mix = await createMix(unique("Delete Me"));
    const deletes: string[] = [];
    page.on("request", (r) => {
      if (r.method() === "DELETE") deletes.push(new URL(r.url()).pathname);
    });
    await openSoundscape(page);
    const saved = savedPanel(page);
    const opener = saved.getByRole("button", { name: `Delete ${mix.name}` });

    await opener.click();
    const dialog = page.getByRole("alertdialog", { name: "Delete this mix?" });
    await expect(dialog).toContainText(mix.name);
    await expect(dialog.getByRole("button", { name: "Cancel" })).toBeFocused();
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toHaveCount(0);
    await expect(opener).toBeFocused();
    await expect(
      saved.getByRole("button", { name: `Load ${mix.name}` }),
    ).toBeVisible();
    expect(deletes).toEqual([]);

    await opener.click();
    const confirmButton = dialog.getByRole("button", { name: "Delete mix" });
    await expect(confirmButton).toBeVisible();
    // Two activations in the same task, before any re-render can disable it.
    await confirmButton.evaluate((button: HTMLButtonElement) => {
      button.click();
      button.click();
    });
    await expect(dialog).toHaveCount(0);
    await expect(
      saved.getByRole("button", { name: `Load ${mix.name}` }),
    ).toHaveCount(0);
    await page.waitForTimeout(500);
    expect(deletes).toEqual([`/api/soundscape/mixes/${mix.id}`]);

    // The opener's row is gone: focus lands on the documented fallback.
    await expect(
      saved.getByRole("heading", { level: 2, name: "Saved mixes" }),
    ).toBeFocused();
    expect(await page.evaluate(() => document.activeElement?.id ?? null)).toBe(
      "soundscape-saved-heading",
    );

    saveJson("T20", "delete-request-count", {
      claim:
        "Cancel sends no DELETE; a double-activated confirm sends exactly one",
      deletesAfterCancel: 0,
      deletes,
    });
  });

  test("a mix deleted in another tab: the error shows and the list refreshes", async ({
    page,
  }) => {
    const mix = await createMix(unique("Gone"));
    await openSoundscape(page);
    await deleteMixQuietly(mix.id);

    const reloaded = page.waitForResponse(
      (r) => MIXES(new URL(r.url())) && r.request().method() === "GET",
    );
    await savedPanel(page)
      .getByRole("button", { name: `Delete ${mix.name}` })
      .click();
    const dialog = page.getByRole("alertdialog", { name: "Delete this mix?" });
    await dialog.getByRole("button", { name: "Delete mix" }).click();
    await expect(dialog.getByRole("alert")).toContainText(
      "This item no longer exists.",
    );
    await reloaded;
    await expect(
      savedPanel(page).getByRole("button", { name: `Load ${mix.name}` }),
    ).toHaveCount(0);
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toHaveCount(0);
    await expect(
      savedPanel(page).getByRole("heading", { name: "Saved mixes" }),
    ).toBeFocused();
  });

  test("no sounds: the host message without an Admin link; no mixes: how to save one (FR-054)", async ({
    page,
  }) => {
    await page.route(SOUNDS, (route) => route.fulfill({ json: [] }));
    await page.route(MIXES, (route) => route.fulfill({ json: [] }));
    await page.goto("/soundscape/");
    const sounds = soundsPanel(page);
    await expect(
      sounds.getByText("The host has not added any sounds yet."),
    ).toBeVisible();
    await expect(sounds.getByRole("link")).toHaveCount(0);
    await expect(
      savedPanel(page).getByRole("heading", { name: "No saved mixes" }),
    ).toBeVisible();
    await expect(savedPanel(page)).toContainText("press Save mix");
  });

  test("load failures show the error with Retry (FR-011)", async ({ page }) => {
    await page.route(SOUNDS, (route) =>
      route.fulfill({ status: 500, body: "" }),
    );
    await page.route(MIXES, (route) => route.abort("failed"));
    await page.goto("/soundscape/");
    await expect(soundsPanel(page)).toContainText(
      "Something went wrong on the server",
    );
    await expect(savedPanel(page)).toContainText("Could not reach the server");
    await page.unroute(SOUNDS);
    await page.unroute(MIXES);
    await soundsPanel(page).getByRole("button", { name: "Retry" }).click();
    await savedPanel(page).getByRole("button", { name: "Retry" }).click();
    await expect(
      savedPanel(page).getByRole("button", { name: "Load Evening" }),
    ).toBeVisible();
    await expect(
      weather(page).getByRole("button", { name: "Add Rain to mix" }),
    ).toBeVisible();
  });
});

test.describe("admin", () => {
  test.use({ storageState: storageStatePath("admin") });

  test("no sounds: the host message links to Admin for admins (FR-054)", async ({
    page,
  }) => {
    await page.route(SOUNDS, (route) => route.fulfill({ json: [] }));
    await page.goto("/soundscape/");
    const link = soundsPanel(page).getByRole("link", {
      name: "Add sounds in Admin",
    });
    await expect(link).toHaveAttribute("href", "/admin/?section=soundscape");
  });
});

for (const locale of ["en", "tr"] as const) {
  test.describe(`screenshots (${locale})`, () => {
    test.use({
      storageState: storageStatePath("member"),
      ...localeOptions(locale),
    });

    test("Evening loaded at 1440 and 375; no sideways page scroll", async ({
      page,
    }) => {
      await page.goto("/soundscape/");
      const load = savedPanel(page).getByRole("button", {
        name: locale === "en" ? "Load Evening" : "Evening karışımını yükle",
      });
      await load.click();
      await expect(
        mixPanel(page).getByRole("slider", {
          name: locale === "en" ? "Rain volume" : "Rain ses düzeyi",
        }),
      ).toHaveValue("30");
      for (const width of [1440, 375] as const) {
        await page.setViewportSize(viewport(width));
        expect(
          await page.evaluate(
            () =>
              document.documentElement.scrollWidth <=
              document.documentElement.clientWidth,
          ),
        ).toBe(true);
        await saveScreenshot(page, "T20", `soundscape-${locale}-${width}`);
      }
    });
  });
}
