// T19: the Player screen (US3, FR-012, FR-043–FR-048, PB-1–PB-5, PB-16,
// PB-18, SC-005, SC-008, SC-009 item 3). Member tests only read; tests that
// write playback state, completions, or notes run as the admin so the
// member's seeded progress stays intact for other suites.
import type { Page, Request } from "@playwright/test";
import { storageStatePath } from "../support/auth";
import { saveJson, saveScreenshot } from "../support/evidence";
import { localeOptions } from "../support/locale";
import { readSeed, type Named } from "../support/seed-content";
import { baseURL } from "../support/server-env";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

const AUDIO = "**/api/meditation/*/audio";
const ORIGIN = new URL(baseURL()).origin;

function meditation(name: string): Named {
  const seed = readSeed();
  const all = [
    ...seed.programs.withCover.meditations,
    ...seed.programs.noCover.meditations,
  ];
  const found = all.find((item) => item.name === name);
  if (!found) throw new Error(`seed has no meditation "${name}"`);
  return found;
}

type AudioState = {
  currentTime: number;
  paused: boolean;
  rate: number;
  readyState: number;
};

const audioState = (page: Page): Promise<AudioState> =>
  page.evaluate(() => {
    const audio = document.querySelector("audio");
    if (!audio) throw new Error("no audio element");
    return {
      currentTime: audio.currentTime,
      paused: audio.paused,
      rate: audio.playbackRate,
      readyState: audio.readyState,
    };
  });

const playButton = (page: Page) =>
  page.getByRole("button", { name: "Play", exact: true });
const pauseButton = (page: Page) =>
  page.getByRole("button", { name: "Pause", exact: true });
const seekSlider = (page: Page) =>
  page.getByRole("slider", { name: "Playback position" });
const speedButton = (page: Page) =>
  page.getByRole("button", { name: /^Speed / });
const mixAudio = (page: Page) => page.locator("audio[data-sound-id]");
const mixPaused = (page: Page, paused: boolean) =>
  page.evaluate(
    (want) =>
      [
        ...document.querySelectorAll<HTMLAudioElement>("audio[data-sound-id]"),
      ].every((a) => a.paused === want),
    paused,
  );
const notesRegion = (page: Page) =>
  page.getByRole("region", { name: /^(Notes|Notlar)$/ });

async function openPlayer(page: Page, id: number): Promise<void> {
  await page.goto(`/player/?id=${id}`);
  await expect(page.locator('main input[type="range"]')).toBeVisible();
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

const isPlaybackPost = (request: Request) =>
  request.method() === "POST" &&
  /\/api\/meditation\/\d+\/playback-state$/.test(request.url());

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  test("name is the in-content h1; program link on the cover, download (FR-043, SC-009)", async ({
    page,
  }) => {
    const seed = readSeed();
    const bodyScan = meditation("Body Scan");
    await openPlayer(page, bodyScan.id);

    await expect(page.getByRole("heading", { level: 1 })).toHaveCount(1);
    await expect(page.locator("main h1")).toContainText("Body Scan");
    await expect(page.getByText("From the program")).toHaveCount(0);
    const program = page.getByRole("link", {
      name: "Open program Morning Calm",
    });
    await expect(program).toHaveAttribute(
      "href",
      `/program/?id=${seed.programs.withCover.id}`,
    );
    await expect(
      page.locator(
        `main img[src="/api/meditation/programs/${seed.programs.withCover.id}/cover"]`,
      ),
    ).toBeVisible();
    await page
      .getByRole("button", { name: "More actions for Body Scan" })
      .click();
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.getByRole("menuitem", { name: "Download meditation" }).click(),
    ]);
    expect(new URL(download.url()).pathname).toBe(
      `/api/meditation/${bodyScan.id}/download`,
    );
    await expect(page.getByRole("switch")).toHaveCount(0);
    await expect(page.locator("audio").first()).toBeHidden();
    await expect(page.locator("audio").first()).toHaveAttribute(
      "src",
      `/api/meditation/${bodyScan.id}/audio`,
    );
  });

  test("a completed meditation starts at 0 even with a saved position (PB-1)", async ({
    page,
  }) => {
    const bodyScan = meditation("Body Scan");
    await page.route(`**/api/meditation/${bodyScan.id}/playback-state`, (r) =>
      r.request().method() === "GET"
        ? r.fulfill({ json: { position_ms: 6_000, completed: true } })
        : r.continue(),
    );
    await openPlayer(page, bodyScan.id);
    expect((await audioState(page)).currentTime).toBe(0);
  });

  test("Media Session carries the meditation name (PB-5)", async ({ page }) => {
    await openPlayer(page, meditation("Body Scan").id);
    await expect
      .poll(() =>
        page.evaluate(() => navigator.mediaSession.metadata?.title ?? null),
      )
      .toBe("Body Scan");
  });

  test("Media Session play, pause, seekbackward, seekforward, seekto, and stop act on the audio (PB-5)", async ({
    page,
  }) => {
    // The browser keeps registered handlers private; record them as the page
    // registers them, then invoke each the way an OS media control would.
    await page.addInitScript(() => {
      const handlers: Record<string, MediaSessionActionHandler | null> = {};
      (
        window as unknown as { __mediaHandlers: typeof handlers }
      ).__mediaHandlers = handlers;
      const original = MediaSession.prototype.setActionHandler;
      MediaSession.prototype.setActionHandler = function (action, handler) {
        handlers[action] = handler;
        return original.call(this, action, handler);
      };
    });
    await openPlayer(page, meditation("Body Scan").id);
    await expect
      .poll(() =>
        page.evaluate(() =>
          [
            "play",
            "pause",
            "seekbackward",
            "seekforward",
            "seekto",
            "stop",
          ].filter(
            (name) =>
              typeof (
                window as unknown as {
                  __mediaHandlers: Record<string, unknown>;
                }
              ).__mediaHandlers[name] === "function",
          ),
        ),
      )
      .toEqual([
        "play",
        "pause",
        "seekbackward",
        "seekforward",
        "seekto",
        "stop",
      ]);

    const invoke = (action: string, details: object = {}) =>
      page.evaluate(
        ([name, extra]) =>
          (
            window as unknown as {
              __mediaHandlers: Record<string, (details: object) => void>;
            }
          ).__mediaHandlers[name]({ action: name, ...(extra as object) }),
        [action, details] as const,
      );
    const sessionState = () =>
      page.evaluate(() => navigator.mediaSession.playbackState);
    const duration = await page.evaluate(
      () => document.querySelector("audio")!.duration,
    );
    expect(duration).toBeGreaterThanOrEqual(20);

    // play starts the audio and the session reports it.
    await invoke("play");
    await expect.poll(async () => !(await audioState(page)).paused).toBe(true);
    await expect.poll(sessionState).toBe("playing");

    // pause stops it.
    await invoke("pause");
    await expect.poll(async () => (await audioState(page)).paused).toBe(true);
    await expect.poll(sessionState).toBe("paused");

    // seekto jumps to the requested second.
    await invoke("seekto", { seekTime: 15 });
    expect((await audioState(page)).currentTime).toBeCloseTo(15, 1);
    // seekbackward and seekforward move by 10 seconds.
    await invoke("seekbackward");
    expect((await audioState(page)).currentTime).toBeCloseTo(5, 1);
    await invoke("seekforward");
    expect((await audioState(page)).currentTime).toBeCloseTo(15, 1);
    // seekto without a time changes nothing.
    await invoke("seekto");
    expect((await audioState(page)).currentTime).toBeCloseTo(15, 1);

    // stop pauses and rewinds to 0.
    await invoke("play");
    await expect.poll(async () => !(await audioState(page)).paused).toBe(true);
    await invoke("stop");
    await expect.poll(async () => (await audioState(page)).paused).toBe(true);
    expect((await audioState(page)).currentTime).toBe(0);
    await expect.poll(sessionState).toBe("paused");
  });

  test("notes newest first; Add needs text; delete cancel keeps the note (FR-046/047, SC-008)", async ({
    page,
  }) => {
    const seed = readSeed();
    const bodyScan = meditation("Body Scan");
    await openPlayer(page, bodyScan.id);
    const notes = notesRegion(page);
    const items = notes.getByRole("listitem");
    const texts = seed.meditationNotes
      .filter((note) => note.meditationId === bodyScan.id)
      .map((note) => note.text);
    await expect(items).toHaveCount(texts.length);
    for (const [index, text] of [...texts].reverse().entries())
      await expect(items.nth(index)).toContainText(text);
    // Local date and time beside each note.
    await expect(items.first()).toContainText(/\d{1,2}:\d{2}/);

    const add = notes.getByRole("button", { name: "Add note" });
    await expect(add).toBeDisabled();
    await notes.getByLabel("New note").fill("   ");
    await expect(add).toBeDisabled();
    await notes.getByLabel("New note").fill("x");
    await expect(add).toBeEnabled();
    await notes.getByLabel("New note").fill("");

    const remove = items.first().getByRole("button", { name: "Delete note" });
    await remove.click();
    const dialog = page.getByRole("alertdialog");
    await expect(dialog).toContainText(texts[texts.length - 1]);
    await expect(dialog.getByRole("button", { name: "Cancel" })).toBeFocused();
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toBeHidden();
    await expect(items).toHaveCount(texts.length);
    await expect(remove).toBeFocused();

    // Escape cancels too.
    await remove.click();
    await expect(dialog).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();
    await expect(items).toHaveCount(texts.length);
  });

  test("a 404 id shows not found with a Library link, not login (FR-012, SC-005)", async ({
    page,
  }) => {
    await page.goto("/player/?id=999999");
    await expect(
      page.getByRole("heading", { name: "Meditation not found" }),
    ).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Back to Library" }),
    ).toHaveAttribute("href", "/library/");
    const url = new URL(page.url());
    expect(url.pathname).toBe("/player/");
    expect(url.search).toBe("?id=999999");
    await expect(page.getByRole("heading", { level: 1 })).toHaveText("Player");
  });

  test("an invalid id opens Library without a meditation request (PB-18)", async ({
    page,
  }) => {
    const log: Record<string, string[]> = {};
    for (const search of ["", "?id=abc", "?id=0", "?id=-2"]) {
      const calls: string[] = [];
      const listener = (request: Request) => {
        const path = new URL(request.url()).pathname;
        if (/^\/api\/meditation\/-?\d+/.test(path) || path.includes("abc"))
          calls.push(`${request.method()} ${path}`);
      };
      page.on("request", listener);
      await page.goto(`/player/${search}`);
      await page.waitForURL("**/library/");
      page.off("request", listener);
      expect(calls).toEqual([]);
      log[search || "(none)"] = calls;
    }
    saveJson("T19", "invalid-id-request-count", log);
  });

  test("320 px: no horizontal scroll and every control is reachable (US3)", async ({
    page,
  }) => {
    await page.setViewportSize(viewport(320));
    await openPlayer(page, meditation("Body Scan").id);
    await expect(notesRegion(page).getByRole("listitem")).not.toHaveCount(0);
    const overflow = await page.evaluate(
      () =>
        document.documentElement.scrollWidth -
        document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);

    const controls = [
      page.getByRole("link", { name: "Open program Morning Calm" }),
      seekSlider(page),
      page.getByRole("button", { name: "Restart" }),
      page.getByRole("button", { name: "Back 10 seconds" }),
      playButton(page),
      page.getByRole("button", { name: "Forward 10 seconds" }),
      speedButton(page),
      page.getByRole("button", { name: "Edit sounds" }),
      page.getByRole("button", { name: /More actions for/ }),
      notesRegion(page).getByLabel("New note"),
      notesRegion(page).getByRole("button", { name: "Add note" }),
      notesRegion(page).getByRole("button", { name: "Delete note" }).first(),
    ];
    for (const control of controls) {
      await control.scrollIntoViewIfNeeded();
      await expect(control).toBeInViewport();
      const box = await control.boundingBox();
      expect(box).not.toBeNull();
      expect(box!.x).toBeGreaterThanOrEqual(0);
      expect(box!.x + box!.width).toBeLessThanOrEqual(320);
      expect(box!.height).toBeGreaterThanOrEqual(44);
    }
  });
});

for (const locale of ["en", "tr"] as const) {
  test.describe(`member screenshots (${locale})`, () => {
    test.use({
      storageState: storageStatePath("member"),
      ...localeOptions(locale),
    });

    test("player at 1440 and 375", async ({ page }) => {
      await openPlayer(page, meditation("Body Scan").id);
      await expect(notesRegion(page).getByRole("listitem")).not.toHaveCount(0);
      await expect(page.getByRole("heading", { level: 1 })).toContainText(
        "Body Scan",
      );
      await expect(
        page.getByRole("button", {
          name: locale === "tr" ? "Oynat" : "Play",
          exact: true,
        }),
      ).toBeVisible();
      for (const width of [1440, 375] as const) {
        await page.setViewportSize(viewport(width));
        // Let the glass layers repaint at the new width before a full-page capture.
        await page.waitForTimeout(300);
        await saveScreenshot(page, "T19", `player-${locale}-${width}`);
      }
    });
  });
}

test.describe("admin (writes)", () => {
  test.use({ storageState: storageStatePath("admin") });

  test("resumes from a saved position of 6 s (PB-1)", async ({ page }) => {
    const quietMind = meditation("Quiet Mind");
    const saved = await page.request.post(
      `/api/meditation/${quietMind.id}/playback-state`,
      { data: { position_ms: 6_000 }, headers: { Origin: ORIGIN } },
    );
    expect(saved.ok()).toBe(true);
    await openPlayer(page, quietMind.id);
    await expect
      .poll(async () => (await audioState(page)).currentTime)
      .toBeCloseTo(6, 1);
    await expect(seekSlider(page)).toHaveAttribute(
      "aria-valuetext",
      /^6 seconds of \d+ seconds$/,
    );
  });

  test("pause reports the position (PB-2)", async ({ page }) => {
    await openPlayer(page, meditation("Quiet Mind").id);
    await startPlaying(page);
    const posted = page.waitForRequest(isPlaybackPost);
    await pauseButton(page).click();
    const request = await posted;
    const body = request.postDataJSON() as { position_ms: number };
    expect(body.position_ms).toBeGreaterThan(0);
    expect(request.resourceType()).toBe("fetch");
    await expect(playButton(page)).toBeVisible();
  });

  test("each hide while playing sends one beacon, captured by the route (PB-2)", async ({
    page,
  }) => {
    const quietMind = meditation("Quiet Mind");
    const captured: { type: string; position: number }[] = [];
    await page.route(
      `**/api/meditation/${quietMind.id}/playback-state`,
      async (route) => {
        const request = route.request();
        if (request.method() === "POST")
          captured.push({
            type: request.resourceType(),
            position: (request.postDataJSON() as { position_ms: number })
              .position_ms,
          });
        await route.continue();
      },
    );
    await openPlayer(page, quietMind.id);
    await startPlaying(page);
    const beacons = () => captured.filter((c) => c.type !== "fetch");
    // A tab switch: visibilitychange (hidden), then pagehide of the same hide.
    const hide = () =>
      page.evaluate(() => {
        Object.defineProperty(document, "visibilityState", {
          configurable: true,
          get: () => "hidden",
        });
        document.dispatchEvent(new Event("visibilitychange"));
        window.dispatchEvent(new PageTransitionEvent("pagehide"));
      });
    const show = () =>
      page.evaluate(() => {
        Object.defineProperty(document, "visibilityState", {
          configurable: true,
          get: () => "visible",
        });
        document.dispatchEvent(new Event("visibilitychange"));
      });
    await hide();
    await expect.poll(() => beacons().length).toBe(1);
    await page.waitForTimeout(500);
    expect(beacons()).toHaveLength(1);
    expect(beacons()[0].position).toBeGreaterThan(0);

    await show();
    await hide();
    await expect.poll(() => beacons().length).toBe(2);

    // Paused: a hide sends nothing.
    await show();
    await pauseButton(page).click();
    await expect(playButton(page)).toBeVisible();
    await hide();
    await page.waitForTimeout(500);
    expect(beacons()).toHaveLength(2);
    saveJson("T19", "beacon-route-capture", { captured });
  });

  test("page.close() while playing reports the position by beacon (PB-2)", async ({
    page,
  }) => {
    // Playwright stops observing a page's requests once it closes, so the
    // beacon is verified on the server: fetch reports are aborted, so only
    // the beacon can move the saved position off 1 s.
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
    let saved = 0;
    await expect
      .poll(async () => {
        saved = ((await (await api.get(url)).json()) as { position_ms: number })
          .position_ms;
        return saved;
      })
      .toBeGreaterThan(2_000);
    saveJson("T19", "beacon-close-server-state", { before: 1_000, saved });
  });

  test("seeking to the end records exactly one completion (PB-3)", async ({
    page,
  }) => {
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
    await openPlayer(page, slowBreathing.id);
    await startPlaying(page);
    await seekSlider(page).focus();
    await page.keyboard.press("End");
    await expect.poll(() => completions.length).toBe(1);
    // The congratulations dialog opens: an optional note, one Continue that
    // saves the note (no Save button) and returns to the program page.
    const done = page.getByRole("dialog", { name: "Well done" });
    await expect(done).toBeVisible();
    const text = `Completion note ${Date.now()}`;
    await done.getByLabel(/optional/i).fill(text);
    await done.getByRole("button", { name: "Continue" }).click();
    await page.waitForURL(`**/program/?id=*`);
    await expect(
      notesRegion(page).getByRole("listitem").filter({ hasText: text }),
    ).toHaveCount(1);
    await expect(playButton(page)).toBeVisible();
    await page.waitForTimeout(1_500);
    expect(completions).toHaveLength(1);
    const body = completions[0].postDataJSON() as {
      started_at: string;
      finished_at: string;
    };
    expect(Date.parse(body.started_at)).toBeLessThan(
      Date.parse(body.finished_at),
    );
  });

  test("completion dialog: Escape saves the prefilled-note edit; no program Continue goes to Library", async ({
    page,
  }) => {
    // Slow Breathing belongs to "Evening Rest"; Body Scan to "Morning Calm".
    const slowBreathing = meditation("Slow Breathing");
    const bodyScan = meditation("Body Scan");
    const seed = readSeed();
    const notes = await (
      await page.request.get(`/api/meditation/${bodyScan.id}/notes`, {
        headers: { Origin: ORIGIN },
      })
    ).json();
    const prefill = notes[0].text as string;
    const edited = `Edited note ${Date.now()}`;

    // Escape closes and saves the edit of the prefilled newest note.
    await page.request.put(`/api/meditation/${bodyScan.id}/completed`, {
      data: { completed: false },
      headers: { Origin: ORIGIN },
    });
    await openPlayer(page, bodyScan.id);
    await startPlaying(page);
    await seekSlider(page).focus();
    await page.keyboard.press("End");
    const done = page.getByRole("dialog", { name: "Well done" });
    await expect(done).toBeVisible();
    await expect(done.getByLabel(/optional/i)).toHaveValue(
      expect.stringContaining(prefill.slice(0, 10)),
    );
    await done.getByLabel(/optional/i).fill(edited);
    await page.keyboard.press("Escape");
    await expect(done).toBeHidden();
    await expect
      .poll(() =>
        page
          .getByRole("region", { name: /^(Notes|Notlar)$/ })
          .getByRole("listitem")
          .filter({ hasText: edited })
          .count(),
      )
      .toBe(1);

    // A meditation in no program would Continue to Library; with one it goes
    // to the program page (verified above). Slow Breathing's own note field
    // stays empty: closing saves nothing.
    await openPlayer(page, slowBreathing.id);
    await startPlaying(page);
    await seekSlider(page).focus();
    await page.keyboard.press("End");
    await expect(done).toBeVisible();
    const notePosts: Request[] = [];
    page.on("request", (request) => {
      if (
        request.method() === "POST" &&
        request.url().endsWith(`/api/meditation/${slowBreathing.id}/notes`)
      )
        notePosts.push(request);
    });
    await done.getByRole("button", { name: "Continue" }).click();
    await page.waitForURL(`**/program/?id=${seed.programs.noCover.id}`);
    expect(notePosts).toHaveLength(0);
  });

  test("one speed button cycles 1x, 1.25x, 1.5x, 1x; resets after a reload (PB-4)", async ({
    page,
  }) => {
    await openPlayer(page, meditation("Quiet Mind").id);
    await expect(page.getByRole("radio")).toHaveCount(0);
    await expect(speedButton(page)).toHaveCount(1);
    await expect(speedButton(page)).toHaveAccessibleName("Speed 1×");
    expect((await audioState(page)).rate).toBe(1);
    for (const [label, rate] of [
      ["Speed 1.25×", 1.25],
      ["Speed 1.5×", 1.5],
      ["Speed 1×", 1],
    ] as const) {
      await speedButton(page).click();
      await expect(speedButton(page)).toHaveAccessibleName(label);
      expect((await audioState(page)).rate).toBe(rate);
    }
    await speedButton(page).click();
    expect((await audioState(page)).rate).toBe(1.25);

    await page.reload();
    await page.waitForFunction(
      () => (document.querySelector("audio")?.readyState ?? 0) >= 1,
    );
    await expect(speedButton(page)).toHaveAccessibleName("Speed 1×");
    expect((await audioState(page)).rate).toBe(1);
  });

  test("Restart seeks to the start (US3)", async ({ page }) => {
    await openPlayer(page, meditation("Quiet Mind").id);
    await seekSlider(page).focus();
    await page.keyboard.press("End");
    await expect
      .poll(async () => (await audioState(page)).currentTime)
      .toBeGreaterThan(5);
    await page.getByRole("button", { name: "Restart" }).click();
    await expect.poll(async () => (await audioState(page)).currentTime).toBe(0);
  });

  test("playing starts a background mix that follows pause; the dialog edits it, groups by category, and Random refills", async ({
    page,
  }) => {
    await openPlayer(page, meditation("Quiet Mind").id);
    await expect(mixAudio(page)).toHaveCount(0);
    await playButton(page).click();
    // Headless Chromium may refuse audio: assert the engine's elements.
    await expect.poll(() => mixAudio(page).count()).toBeGreaterThan(0);
    const count = await mixAudio(page).count();
    expect(count).toBeLessThanOrEqual(3);
    await expect.poll(() => mixPaused(page, false)).toBe(true);
    await pauseButton(page).click();
    await expect.poll(() => mixPaused(page, true)).toBe(true);

    // The Soundscape toggle in the main controls opens the mixer dialog.
    await page.getByRole("button", { name: "Soundscape" }).click();
    const dialog = page.getByRole("dialog", { name: "Background mix" });
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole("slider")).toHaveCount(count);
    await dialog
      .getByRole("button", { name: /^Remove / })
      .first()
      .click();
    await expect(mixAudio(page)).toHaveCount(count - 1);

    // The addable sounds are grouped by category, Uncategorized last (the
    // Soundscape screen's library pattern; h4 groups under the Add heading).
    const addSection = dialog.getByRole("heading", {
      name: "Add a sound",
      level: 3,
    });
    await expect(addSection).toBeVisible();
    const groups = dialog.locator("h4");
    await expect(groups.first()).toBeVisible();
    const names = (await groups.allInnerTexts()).map((name) => name.trim());
    if (names.includes("Uncategorized"))
      expect(names[names.length - 1]).toBe("Uncategorized");
    expect(new Set(names).size).toBe(names.length);

    // Random replaces the mix with 3 distinct sounds at volume 50.
    await dialog.getByRole("button", { name: "Random" }).click();
    await expect(dialog.getByRole("slider")).toHaveCount(3);
    for (const volume of await dialog
      .getByRole("slider")
      .evaluateAll((els) =>
        els.map((el) => Number((el as HTMLInputElement).value)),
      ))
      expect(volume).toBe(50);

    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();
    // Leaving the screen drops the mix audio.
    await page.goto("/library/");
    await expect(mixAudio(page)).toHaveCount(0);
  });

  test("audio 500 shows the playback error; notes, completion, and download still work (FR-048)", async ({
    page,
  }) => {
    const breathFocus = meditation("Breath Focus");
    await page.route(AUDIO, (route) => route.fulfill({ status: 500 }));
    await page.goto(`/player/?id=${breathFocus.id}`);
    await expect(page.getByRole("alert")).toContainText(
      "audio could not be played",
    );
    expect(new URL(page.url()).pathname).toBe("/player/");
    await expect(playButton(page)).toBeDisabled();

    await page
      .getByRole("button", { name: "More actions for Breath Focus" })
      .click();
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.getByRole("menuitem", { name: "Download meditation" }).click(),
    ]);
    expect(new URL(download.url()).pathname).toBe(
      `/api/meditation/${breathFocus.id}/download`,
    );

    const notes = notesRegion(page);
    const text = `T19 note ${Date.now()}`;
    await notes.getByLabel("New note").fill(text);
    await notes.getByRole("button", { name: "Add note" }).click();
    const item = notes.getByRole("listitem").filter({ hasText: text });
    await expect(item).toHaveCount(1);
    await expect(notes.getByRole("listitem").first()).toContainText(text);
    await item.getByRole("button", { name: "Delete note" }).click();
    const dialog = page.getByRole("alertdialog");
    await dialog.getByRole("button", { name: "Delete note" }).click();
    await expect(dialog).toBeHidden();
    await expect(item).toHaveCount(0);
  });

  test("audio error with an ended session goes to Login (plan D2)", async ({
    page,
  }) => {
    let release!: () => void;
    const gate = new Promise<void>((resolve) => (release = resolve));
    await page.route(AUDIO, async (route) => {
      await gate;
      await route.fulfill({ status: 500 }).catch(() => {});
    });
    await page.goto(`/player/?id=${meditation("Breath Focus").id}`);
    await expect(seekSlider(page)).toBeVisible();
    await page.route("**/api/identity/me", (route) =>
      route.fulfill({ status: 401, json: { error: "unauthorized" } }),
    );
    release();
    await page.waitForURL("**/login/**");
    const url = new URL(page.url());
    expect(url.pathname).toBe("/login/");
    expect(url.searchParams.get("next")).toBe(
      `/player/?id=${meditation("Breath Focus").id}`,
    );
  });

  test("keyboard runs play, seek with arrows, skip, and speed (FR-044)", async ({
    page,
  }) => {
    const quietMind = meditation("Quiet Mind");
    await openPlayer(page, quietMind.id);

    await playButton(page).focus();
    await page.keyboard.press("Enter");
    await expect(pauseButton(page)).toBeFocused();
    await expect.poll(async () => (await audioState(page)).paused).toBe(false);
    await page.keyboard.press("Space");
    await expect(playButton(page)).toBeFocused();
    await expect.poll(async () => (await audioState(page)).paused).toBe(true);

    await seekSlider(page).focus();
    await page.keyboard.press("Home");
    await expect.poll(async () => (await audioState(page)).currentTime).toBe(0);
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowRight");
    await expect.poll(async () => (await audioState(page)).currentTime).toBe(2);
    await page.keyboard.press("ArrowLeft");
    await expect.poll(async () => (await audioState(page)).currentTime).toBe(1);
    await expect(seekSlider(page)).toHaveAttribute(
      "aria-valuetext",
      /^1 second of \d+ seconds$/,
    );

    await page.getByRole("button", { name: "Forward 10 seconds" }).focus();
    await page.keyboard.press("Enter");
    await expect
      .poll(async () => (await audioState(page)).currentTime)
      .toBe(11);
    await page.getByRole("button", { name: "Back 10 seconds" }).focus();
    await page.keyboard.press("Enter");
    await expect.poll(async () => (await audioState(page)).currentTime).toBe(1);

    await speedButton(page).focus();
    await page.keyboard.press("Enter");
    await expect(speedButton(page)).toHaveAccessibleName("Speed 1.25×");
    expect((await audioState(page)).rate).toBe(1.25);
  });
});
