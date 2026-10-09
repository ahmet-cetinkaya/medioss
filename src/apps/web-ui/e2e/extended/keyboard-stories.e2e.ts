// T25: keyboard-only walkthroughs of user stories US1-US9 (SC-012, FR-098).
// Every interaction goes through page.keyboard (Tab, Enter, Escape, arrows,
// Space, typing); no pointer API is used anywhere in this file. `tabTo` walks
// the real Tab order until the target holds focus, so an unreachable control
// fails the test.
import type { BrowserContext, Locator, Page } from "@playwright/test";
import { newApi, postJson } from "../support/api-client";
import { loginApi, storageStatePath, USERS } from "../support/auth";
import { localeOptions } from "../support/locale";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

const signedOut = { cookies: [], origins: [] };

/** Presses Tab until `target` is the focused element; fails if it never is. */
async function tabTo(page: Page, target: Locator, max = 150): Promise<void> {
  const focused = () =>
    target
      .first()
      .evaluate((element) => element === document.activeElement)
      .catch(() => false);
  for (let step = 0; step <= max; step++) {
    if (await focused()) return;
    await page.keyboard.press("Tab");
  }
  throw new Error(`Tab never reached ${target.toString()} in ${max} presses`);
}

/** Tab to a text field and type into it with real key presses. */
async function typeInto(
  page: Page,
  field: Locator,
  text: string,
): Promise<void> {
  await tabTo(page, field);
  await field.pressSequentially(text);
  await expect(field).toHaveValue(text);
}

const path = (page: Page) => new URL(page.url()).pathname;

/** A new session for this context only, so shared storage states stay valid. */
async function signInFresh(
  context: BrowserContext,
  email: string,
  password: string,
): Promise<void> {
  const api = await loginApi(email, password);
  await context.addCookies((await api.storageState()).cookies);
  await api.dispose();
}

async function createFreshUser(
  tag: string,
): Promise<{ email: string; password: string; id: number }> {
  const email = `t25-${tag}-${Date.now()}@example.test`;
  const temporary = "t25-e2e-temp-1";
  const password = "t25-e2e-pass-1";
  const admin = await newApi(storageStatePath("admin"));
  const { id } = await postJson<{ id: number }>(admin, "/api/identity/users", {
    email,
    password: temporary,
  });
  await admin.dispose();
  const first = await loginApi(email, temporary);
  await postJson(first, "/api/identity/me/password", { password });
  await first.dispose();
  return { email, password, id };
}

async function deleteUser(id: number): Promise<void> {
  const admin = await newApi(storageStatePath("admin"));
  await admin.delete(`/api/identity/users/${id}`);
  await admin.dispose();
}

const nav = (page: Page, name: string) =>
  page.getByRole("navigation", { name, exact: true });

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  test("US1: the nav surface at every tier is keyboard-operable; Escape cancels sign-out", async ({
    page,
  }) => {
    const surfaces = [
      {
        width: 1440,
        label: "Main navigation",
        link: "Library",
        to: "/library/",
      },
      {
        width: 800,
        label: "Compact navigation",
        link: "Soundscape",
        to: "/soundscape/",
      },
    ];
    for (const { width, label, link, to } of surfaces) {
      await page.setViewportSize(viewport(width as 1440 | 800));
      await page.goto("/stats/");
      await expect(page.locator("main dl dt")).toHaveCount(4);
      await tabTo(page, nav(page, label).getByRole("link", { name: link }));
      await page.keyboard.press("Enter");
      await page.waitForURL((url) => url.pathname === to);
    }

    // Escape cancels the confirmed sign-out and returns focus to its opener.
    await page.setViewportSize(viewport(1440));
    await page.goto("/stats/");
    await expect(page.locator("main dl dt")).toHaveCount(4);
    const opener = nav(page, "Main navigation").getByRole("button", {
      name: "Sign out",
    });
    await tabTo(page, opener);
    await page.keyboard.press("Enter");
    const dialog = page.getByRole("alertdialog", { name: "Sign out?" });
    await expect(dialog.getByRole("button", { name: "Cancel" })).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0);
    await expect(opener).toBeFocused();
    expect(path(page)).toBe("/stats/");
    expect((await page.request.get("/api/identity/me")).status()).toBe(200);

    // Mobile: the bottom bar's More opens the sheet; its links work by keyboard.
    await page.setViewportSize(viewport(375));
    await page.goto("/stats/");
    await expect(page.locator("main dl dt")).toHaveCount(4);
    await tabTo(page, page.getByRole("button", { name: "More" }));
    await page.keyboard.press("Enter");
    const sheet = page.getByRole("dialog", { name: "More" });
    await expect(sheet).toBeVisible();
    await tabTo(
      page,
      sheet.getByRole("navigation", { name: "More pages" }).getByRole("link", {
        name: "History",
      }),
    );
    await page.keyboard.press("Enter");
    await page.waitForURL((url) => url.pathname === "/history/");
  });

  test("US2: Home regions are reachable and open by keyboard", async ({
    page,
  }) => {
    await page.goto("/");
    const pill = page.getByRole("region", { name: "Continue", exact: true });
    await tabTo(page, pill.getByRole("link", { name: /^Continue with / }));
    await page.keyboard.press("Enter");
    await page.waitForURL((url) => url.pathname === "/player/");
    expect(new URL(page.url()).searchParams.get("id")).toMatch(/^\d+$/);

    // The Library preview's "See all" opens the Library screen.
    await page.goto("/");
    const library = page.getByRole("region", { name: "Library", exact: true });
    await tabTo(page, library.getByRole("link", { name: "See all" }));
    await page.keyboard.press("Enter");
    await page.waitForURL((url) => url.pathname === "/library/");
  });

  test("US3: library search to result, then play/pause, seek, speed and completion by keyboard", async ({
    page,
  }) => {
    await page.goto("/library/");
    const search = page.getByRole("searchbox", {
      name: /Search the library/,
    });
    await expect(search).toBeEditable();
    await typeInto(page, search, "Body Scan");
    await page.keyboard.press("Enter");
    const results = page.getByRole("region", {
      name: "Results for “Body Scan”",
    });
    const result = results.getByRole("link", { name: /^Body Scan/ });
    await expect(result).toBeVisible();
    await tabTo(page, result);
    await page.keyboard.press("Enter");
    await page.waitForURL((url) => url.pathname === "/player/");

    const audio = () =>
      page.evaluate(() => {
        const element = document.querySelector("audio");
        if (!element) throw new Error("no audio element");
        return {
          currentTime: element.currentTime,
          paused: element.paused,
          rate: element.playbackRate,
        };
      });
    await expect(page.locator('main input[type="range"]')).toBeVisible();
    await page.waitForFunction(
      () => (document.querySelector("audio")?.readyState ?? 0) >= 1,
    );

    const play = page.getByRole("button", { name: "Play", exact: true });
    const pause = page.getByRole("button", { name: "Pause", exact: true });
    await tabTo(page, play);
    await page.keyboard.press("Enter");
    await expect(pause).toBeFocused();
    await expect.poll(async () => (await audio()).paused).toBe(false);
    await page.keyboard.press("Space");
    await expect(play).toBeFocused();
    await expect.poll(async () => (await audio()).paused).toBe(true);

    const slider = page.getByRole("slider", { name: "Playback position" });
    await tabTo(page, slider);
    // Under load playback may have advanced a few frames (currentTime ~0.06)
    // before pausing while the step=1 slider still reads 0; Home would then be
    // a no-op (no input event). Move off 0 first so Home is a real seek.
    await page.keyboard.press("ArrowRight");
    await expect
      .poll(async () => (await audio()).currentTime)
      .toBeGreaterThanOrEqual(1);
    await page.keyboard.press("Home");
    await expect.poll(async () => (await audio()).currentTime).toBe(0);
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowRight");
    await expect.poll(async () => (await audio()).currentTime).toBe(2);
    await page.keyboard.press("ArrowLeft");
    await expect.poll(async () => (await audio()).currentTime).toBe(1);

    // Speed is a single cycling button (1x -> 1.25x -> 1.5x -> 1x) and the
    // Completed switch is gone; completion records automatically at the end.
    const speed = page.getByRole("button", { name: /^Speed / });
    await tabTo(page, speed);
    await page.keyboard.press("Space");
    await expect(speed).toHaveAccessibleName("Speed 1.25×");
    expect((await audio()).rate).toBe(1.25);
    await page.keyboard.press("Space");
    await expect(speed).toHaveAccessibleName("Speed 1.5×");
    expect((await audio()).rate).toBe(1.5);
    await page.keyboard.press("Space");
    await expect(speed).toHaveAccessibleName("Speed 1×");
    expect((await audio()).rate).toBe(1);
    await expect(page.getByRole("switch", { name: "Completed" })).toHaveCount(
      0,
    );

    const restart = page.getByRole("button", { name: "Restart" });
    await tabTo(page, restart);
    await page.keyboard.press("Space");
    await expect.poll(async () => (await audio()).currentTime).toBe(0);
  });

  test("US4: mood radios move with arrows and Save records the entry by keyboard", async ({
    page,
  }) => {
    await page.goto("/mood/");
    await expect(page.getByRole("radio", { name: /^3/ })).toBeChecked();
    await tabTo(page, page.getByRole("radio", { checked: true }).first());
    await page.keyboard.press("ArrowRight");
    await expect(page.getByRole("radio", { name: /^4/ })).toBeChecked();
    await page.keyboard.press("ArrowLeft");
    await page.keyboard.press("ArrowLeft");
    await expect(page.getByRole("radio", { name: /^2/ })).toBeChecked();
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowRight");
    await expect(page.getByRole("radio", { name: /^4/ })).toBeChecked();

    const note = `Keyboard mood ${Date.now()}`;
    await typeInto(page, page.getByLabel("Note (optional)"), note);
    const saved = page.waitForResponse(
      (r) =>
        new URL(r.url()).pathname === "/api/mood" &&
        r.request().method() === "POST",
    );
    await tabTo(page, page.getByRole("button", { name: "Save mood" }));
    await page.keyboard.press("Enter");
    const response = await saved;
    expect(response.status()).toBe(200);
    const entry = (await response.json()) as { id: number; score: number };
    expect(entry.score).toBe(4);
    await expect(
      page
        .getByRole("region", { name: "Entries" })
        .getByRole("listitem")
        .filter({ hasText: note }),
    ).toHaveCount(1);

    const api = await newApi(storageStatePath("member"));
    await api.delete(`/api/mood/${entry.id}`);
    await api.dispose();
  });

  test("US6: statistics range chips change with arrow keys", async ({
    page,
  }) => {
    await page.goto("/stats/");
    await expect(page.locator("main dl dt")).toHaveCount(4);
    await expect(page.getByRole("radio", { name: "All time" })).toBeChecked();
    await tabTo(page, page.getByRole("radio", { checked: true }));

    const ranges = async (key: string, range: string) => {
      const response = page.waitForResponse(
        (r) =>
          r.url().includes("/api/meditation/statistics?") &&
          new URL(r.url()).searchParams.get("range") === range,
      );
      await page.keyboard.press(key);
      expect((await response).status()).toBe(200);
    };
    await ranges("ArrowLeft", "last30");
    await expect(
      page.getByRole("radio", { name: "Last 30 days" }),
    ).toBeChecked();
    await ranges("ArrowRight", "all");
    await expect(page.getByRole("radio", { name: "All time" })).toBeChecked();
    await expect(page.locator("main dl dt")).toHaveCount(4);
  });
});

test.describe("admin", () => {
  test.use({ storageState: storageStatePath("admin") });

  test("US5: section chips and the create-user form work by keyboard", async ({
    page,
  }) => {
    await page.goto("/admin/?section=users");
    await expect(page.getByRole("radio", { name: "Users" })).toBeChecked();
    await tabTo(page, page.getByRole("radio", { checked: true }));
    await page.keyboard.press("ArrowRight");
    await expect(page).toHaveURL(/section=meditations$/);
    await expect(
      page.getByRole("region", { name: "Upload meditation audio" }),
    ).toBeVisible();
    await page.keyboard.press("ArrowRight");
    await expect(page).toHaveURL(/section=soundscape$/);
    await expect(
      page.getByRole("region", { name: "Sound categories" }),
    ).toBeVisible();
    await page.keyboard.press("ArrowLeft");
    await page.keyboard.press("ArrowLeft");
    await expect(page).toHaveURL(/section=users$/);

    const form = page.getByRole("region", { name: "Create user" });
    await expect(form).toBeVisible();
    const email = `t25-kbd-${Date.now()}@example.test`;
    await typeInto(page, form.getByLabel("Email"), email);
    await typeInto(
      page,
      form.getByLabel("Temporary password"),
      "kbd-temp-pass-1",
    );
    const created = page.waitForResponse(
      (r) =>
        r.url().endsWith("/api/identity/users") &&
        r.request().method() === "POST",
    );
    await tabTo(page, form.getByRole("button", { name: "Create user" }));
    await page.keyboard.press("Enter");
    const response = await created;
    expect(response.status()).toBeLessThan(300);
    const { id } = (await response.json()) as { id: number };
    try {
      await expect(form.getByTestId("created-user")).toBeVisible();
      await expect(
        page
          .getByRole("region", { name: "Users", exact: true })
          .getByRole("listitem")
          .filter({ hasText: email }),
      ).toBeVisible();
    } finally {
      await deleteUser(id);
    }
  });
});

test.describe("settings (fresh user)", () => {
  test.use({ storageState: signedOut });

  test("US7: password change submits and language radios switch by keyboard", async ({
    page,
    context,
  }) => {
    const user = await createFreshUser("kbd-settings");
    try {
      await signInFresh(context, user.email, user.password);
      await page.goto("/settings/");
      const password = page.getByRole("region", {
        name: "Password",
        exact: true,
      });
      await expect(password).toBeVisible();
      const next = "t25-kbd-next-1";
      await typeInto(
        page,
        password.getByLabel("New password", { exact: true }),
        next,
      );
      await typeInto(page, password.getByLabel("Confirm new password"), next);
      const changed = page.waitForResponse("**/api/identity/me/password");
      await tabTo(
        page,
        password.getByRole("button", { name: "Change password" }),
      );
      await page.keyboard.press("Enter");
      expect((await changed).ok()).toBe(true);

      const english = page.getByRole("radio", { name: "English" });
      await expect(english).toBeChecked();
      await tabTo(page, english);
      await page.keyboard.press("ArrowRight");
      await expect(page.locator("html")).toHaveAttribute("lang", "tr");
      await expect(
        page.getByRole("radio", { name: "English" }),
      ).not.toBeChecked();
      await page.keyboard.press("ArrowLeft");
      await expect(page.locator("html")).toHaveAttribute("lang", "en");
      await expect(page.getByRole("radio", { name: "English" })).toBeChecked();
    } finally {
      await deleteUser(user.id);
    }
  });
});

test.describe("login", () => {
  test.use({ storageState: signedOut });

  test("US8: sign-in submits by keyboard; a refused attempt focuses the password field", async ({
    page,
  }) => {
    await page.goto("/login/");
    const email = page.getByLabel("Email");
    const password = page.getByLabel("Password");
    await expect(email).toBeEditable();

    // A throwaway email, so no seeded user is penalised.
    await typeInto(page, email, `nobody-${Date.now()}@example.test`);
    await typeInto(page, password, "not-the-password");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("alert")).toContainText(
      "The email or password is not correct.",
    );
    await expect(password).toBeFocused();
    expect(path(page)).toBe("/login/");

    await page.goto("/login/");
    await expect(email).toBeEditable();
    await typeInto(page, email, USERS.member.email);
    await typeInto(page, password, USERS.member.password);
    await tabTo(page, page.getByRole("button", { name: "Sign in" }));
    await page.keyboard.press("Enter");
    await page.waitForURL((url) => url.pathname === "/");
  });

  test.describe("Turkish browser", () => {
    test.use(localeOptions("tr"));

    test("US9: switching the language by keyboard keeps the typed email", async ({
      page,
    }) => {
      await page.goto("/login/");
      await expect(page.locator("html")).toHaveAttribute("lang", "tr");
      const email = page.getByLabel("E-posta");
      await expect(email).toBeEditable();
      await typeInto(page, email, "typed@example.test");

      await tabTo(page, page.locator('label[lang="tr"] input'));
      await page.keyboard.press("ArrowRight");
      await expect(page.locator("html")).toHaveAttribute("lang", "en");
      await expect(page.locator("h1")).toHaveText("Sign in");
      await expect(page.getByLabel("Email")).toHaveValue("typed@example.test");
    });
  });
});
