// T25: privacy walkthrough (plan T25; SC-018, FR-100). Sign in, visit the ten
// screens, open and cancel dialogs, play briefly, load a mix, and switch the
// language: every request stays on the app's own origin, the only
// localStorage key is the language choice, sessionStorage is empty, and no
// session token is readable from JavaScript. Only origins are saved as
// evidence (never headers, URLs with queries, or bodies).
import { USERS } from "../support/auth";
import { saveJson } from "../support/evidence";
import { localeOptions } from "../support/locale";
import { readSeed } from "../support/seed-content";
import { baseURL } from "../support/server-env";
import { expect, test } from "../support/test-fixture";

test.use({
  storageState: { cookies: [], origins: [] },
  ...localeOptions("en"),
});

test("sign-in through ten screens, dialogs, play, mix, and language switch contact only the app origin (SC-018)", async ({
  page,
  context,
  networkGuard,
}) => {
  test.setTimeout(5 * 60_000);
  const seed = readSeed();

  // Playback writes are answered locally so the seeded member state is untouched.
  await page.route(/\/api\/meditation\/\d+\/playback-state$/, (route) =>
    route.request().method() === "GET"
      ? route.continue()
      : route.fulfill({ status: 204 }),
  );

  // Sign in through the real form (the member owns the seeded mixes).
  await page.goto("/login/");
  await page.getByLabel("Email").fill(USERS.member.email);
  await page.getByLabel("Password").fill(USERS.member.password);
  await page.getByRole("button", { name: "Sign in" }).click();
  await page.waitForURL((url) => url.pathname === "/");

  const meditationId = seed.programs.withCover.meditations[0].id;
  const screens = [
    "/",
    "/library/",
    `/program/?id=${seed.programs.withCover.id}`,
    `/player/?id=${meditationId}`,
    "/soundscape/",
    "/mood/",
    "/history/",
    "/stats/",
    "/settings/",
    "/admin/",
  ];
  for (const url of screens) {
    await page.goto(url);
    await expect(page.locator("main")).toBeVisible();
    await page.waitForLoadState("networkidle");
  }

  // Sign-out dialog: open, then cancel (the session stays).
  await page
    .getByRole("navigation", { name: "Main navigation" })
    .getByRole("button", { name: "Sign out" })
    .click();
  const signOut = page.getByRole("alertdialog", { name: "Sign out?" });
  await expect(signOut).toBeVisible();
  await signOut.getByRole("button", { name: "Cancel" }).click();
  await expect(signOut).toHaveCount(0);

  // A confirm dialog on data: delete mix, cancelled.
  await page.goto("/soundscape/");
  await page
    .getByRole("button", { name: `Delete ${seed.mixes.evening.name}` })
    .click();
  const confirm = page.getByRole("alertdialog", { name: "Delete this mix?" });
  await expect(confirm).toBeVisible();
  await confirm.getByRole("button", { name: "Cancel" }).click();
  await expect(confirm).toHaveCount(0);

  // Load a mix (no autoplay).
  await page
    .getByRole("button", { name: `Load ${seed.mixes.evening.name}` })
    .click();
  await expect(page.getByRole("slider", { name: "Rain volume" })).toHaveValue(
    "30",
  );

  // Brief playback.
  await page.goto(`/player/?id=${meditationId}`);
  await page.waitForFunction(
    () => (document.querySelector("audio")?.readyState ?? 0) >= 1,
  );
  await page.getByRole("button", { name: "Play", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Pause", exact: true }),
  ).toBeVisible();
  await page.waitForFunction(() => {
    const audio = document.querySelector("audio");
    return !!audio && !audio.paused && audio.currentTime > 0.3;
  });
  await page.getByRole("button", { name: "Pause", exact: true }).click();

  // Language: Turkish and back.
  await page.goto("/settings/");
  await page.getByRole("radio", { name: "Türkçe" }).check({ force: true });
  await expect(page.locator("html")).toHaveAttribute("lang", "tr");
  await page.getByRole("radio", { name: "English" }).check({ force: true });
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await page.waitForLoadState("networkidle");

  // Browser storage: only the language choice; the session is not script-readable.
  const storage = await page.evaluate(() => ({
    local: Object.keys(localStorage),
    sessionCount: sessionStorage.length,
    jsCookie: document.cookie,
  }));
  expect(
    storage.local.every((key) =>
      ["medioss.locale", "medioss.lastMix"].includes(key),
    ),
  ).toBe(true);
  expect(storage.sessionCount).toBe(0);
  const session = (await context.cookies()).filter((c) =>
    /session/i.test(c.name),
  );
  expect(session.length).toBeGreaterThan(0);
  for (const cookie of session) {
    expect(cookie.httpOnly).toBe(true);
    expect(storage.jsCookie).not.toContain(cookie.value);
  }

  // Final summary: every recorded request is on the app origin.
  const origin = new URL(baseURL()).origin;
  const origins = [...new Set(networkGuard.requests.map((r) => r.origin))];
  expect(networkGuard.requests.length).toBeGreaterThan(50);
  expect(networkGuard.foreign()).toEqual([]);
  expect(origins).toEqual([origin]);
  saveJson("T25", "network-origins", {
    appOrigin: origin,
    requestCount: networkGuard.requests.length,
    origins,
    foreignOrigins: [],
    localStorageKeys: storage.local,
    sessionStorageKeys: 0,
  });
});
