// T24: the Admin screen (US5, FR-071–FR-078, PB-13, PB-16, PB-17, SC-009/010/023).
import { execFileSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { Page } from "@playwright/test";
import { expectOk, newApi, postJson, type Api } from "../support/api-client";
import { loginApi, storageStatePath, USERS } from "../support/auth";
import { saveJson, saveScreenshot } from "../support/evidence";
import { ensureFixtures } from "../support/fixtures-media";
import { localeOptions } from "../support/locale";
import { readSeed } from "../support/seed-content";
import { fixturesDir } from "../support/server-env";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

const TASK = "T24";

/** Admin-only endpoints (FR-071): none may be requested for a member. */
const ADMIN_ENDPOINT =
  /\/api\/identity\/users|\/api\/meditation\/(audio|programs|categories)|\/api\/soundscape\/(categories|sounds)/;

/** Another user's private data (PB-17): Admin never reads any of it. */
const PRIVATE_DATA =
  /\/api\/(mood|timeline|portability)|\/api\/meditation\/(\d+\/(notes|playback-state|completion|completed)|statistics|continue|recent-programs|daily-suggestion)/;

// A cover over the 5 MiB program-cover cap but under the 6 MiB route body
// limit, so the server itself refuses it with 413. Random pixels do not compress.
const OVERSIZE_COVER = path.join(fixturesDir, "cover-5_6mb.png");
function ensureOversizeCover(): string {
  if (!fs.existsSync(OVERSIZE_COVER)) {
    fs.mkdirSync(fixturesDir, { recursive: true });
    execFileSync(
      "ffmpeg",
      [
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-s",
        "1400x1400",
        "-i",
        "pipe:0",
        "-frames:v",
        "1",
        OVERSIZE_COVER,
      ],
      {
        input: randomBytes(1400 * 1400 * 3),
        stdio: ["pipe", "ignore", "inherit"],
      },
    );
  }
  const size = fs.statSync(OVERSIZE_COVER).size;
  expect(size).toBeGreaterThan(5 * 1024 * 1024);
  expect(size).toBeLessThan(6 * 1024 * 1024 - 64 * 1024);
  return OVERSIZE_COVER;
}

const adminApi = (): Promise<Api> => newApi(storageStatePath("admin"));

const unique = (prefix: string) =>
  `${prefix}-${Date.now()}-${Math.floor(Math.random() * 1e4)}@example.test`;

async function createUserApi(email: string, password: string) {
  const api = await adminApi();
  const user = await postJson<{ id: number }>(api, "/api/identity/users", {
    email,
    password,
  });
  await api.dispose();
  return user;
}

async function deleteUserApi(id: number) {
  const api = await adminApi();
  await api.delete(`/api/identity/users/${id}`);
  await api.dispose();
}

const apiCalls = (requests: { method: string; url: string }[]) =>
  requests
    .filter((request) => new URL(request.url).pathname.startsWith("/api/"))
    .map((request) => ({
      method: request.method,
      path: new URL(request.url).pathname + new URL(request.url).search,
    }));

const region = (page: Page, name: string) =>
  page.getByRole("region", { name, exact: true });

async function openSection(
  page: Page,
  section: "users" | "meditations" | "soundscape",
) {
  await page.goto(`/admin/?section=${section}`);
  await expect(page.locator(`main input[value="${section}"]`)).toBeChecked();
  await expect(page.locator('main [data-region-state="loading"]')).toHaveCount(
    0,
  );
}

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  test("sees admins-only with a Home link and makes no admin request (FR-071)", async ({
    page,
    networkGuard,
  }) => {
    const log: Record<string, unknown> = {};
    for (const address of ["/admin/", "/admin/?section=soundscape"]) {
      const start = networkGuard.requests.length;
      await page.goto(address);
      await expect(
        page.getByRole("heading", { name: "Admins only" }),
      ).toBeVisible();
      const home = page
        .getByRole("main")
        .getByRole("link", { name: "Back to Home" });
      await expect(home).toHaveAttribute("href", "/");
      await page.waitForLoadState("networkidle");
      await page.waitForTimeout(500);

      const calls = apiCalls(networkGuard.requests.slice(start));
      log[address] = calls;
      expect(calls.filter((call) => ADMIN_ENDPOINT.test(call.path))).toEqual(
        [],
      );
      // Beyond the session probe, nothing at all.
      expect(calls).toEqual([{ method: "GET", path: "/api/identity/me" }]);
      await expect(page.getByRole("radio")).toHaveCount(0);
    }
    saveJson(TASK, "member-admin-requests", log);
  });
});

test.describe("admin", () => {
  test.use({ storageState: storageStatePath("admin") });

  test("section chips switch; reload keeps ?section=soundscape (FR-072)", async ({
    page,
  }) => {
    await page.goto("/admin/");
    await expect(page.getByRole("radio", { name: "Users" })).toBeChecked();
    await expect(page).toHaveURL(/\/admin\/\?section=users$/);
    await expect(region(page, "Create user")).toBeVisible();

    await page.locator("label").filter({ hasText: "Meditations" }).click();
    await expect(
      page.getByRole("radio", { name: "Meditations" }),
    ).toBeChecked();
    await expect(page).toHaveURL(/section=meditations$/);
    await expect(region(page, "Upload meditation audio")).toBeVisible();
    await expect(region(page, "Create user")).toHaveCount(0);

    await page.locator("label").filter({ hasText: "Soundscape" }).click();
    await expect(page).toHaveURL(/section=soundscape$/);
    await expect(region(page, "Sound categories")).toBeVisible();

    await page.reload();
    await expect(page.getByRole("radio", { name: "Soundscape" })).toBeChecked();
    await expect(region(page, "Sound categories")).toBeVisible();
    await expect(region(page, "Upload meditation audio")).toHaveCount(0);

    await page.goto("/admin/?section=bogus");
    await expect(page.getByRole("radio", { name: "Users" })).toBeChecked();
    await expect(page).toHaveURL(/section=users$/);
  });

  test("duplicate email shows the specific message and keeps the form (SC-010)", async ({
    page,
  }) => {
    await openSection(page, "users");
    const form = region(page, "Create user");
    await form.getByLabel("Email").fill(USERS.member.email);
    await form.getByLabel("Temporary password").fill("duplicate-temp-1");
    await form.getByRole("button", { name: "Create user" }).click();

    await expect(form.getByRole("alert")).toContainText(
      "An account with this email already exists.",
    );
    await expect(form.getByLabel("Email")).toHaveValue(USERS.member.email);
    await expect(form.getByLabel("Temporary password")).toHaveValue(
      "duplicate-temp-1",
    );
  });

  test("a created user's temporary password shows once and is gone after reload (PB-13)", async ({
    page,
  }) => {
    const email = unique("created");
    const password = `temp-${randomBytes(6).toString("hex")}`;
    await openSection(page, "users");
    const form = region(page, "Create user");
    await form.getByLabel("Email").fill(email);
    await form.getByLabel("Temporary password").fill(password);
    const created = page.waitForResponse(
      (r) =>
        r.url().endsWith("/api/identity/users") &&
        r.request().method() === "POST",
    );
    await form.getByRole("button", { name: "Create user" }).click();
    const { id } = (await (await created).json()) as { id: number };

    try {
      const shown = form.getByTestId("created-user");
      const field = shown.getByRole("textbox", { name: "Temporary password" });
      await expect(field).toHaveValue(password);
      await expect(field).toHaveAttribute("readonly", "");
      await expect(shown).toContainText("must change it at first sign-in");
      await expect(shown.getByRole("button", { name: "Copy" })).toBeVisible();
      // The create form was cleared.
      await expect(form.getByLabel("Email")).toHaveValue("");

      const list = region(page, "Users");
      await expect(
        list.getByRole("listitem").filter({ hasText: email }),
      ).toContainText("Member · Must change password");

      const stored = await page.evaluate(() =>
        JSON.stringify({ ...localStorage, ...sessionStorage }),
      );
      expect(stored).not.toContain(password);

      await page.reload();
      await expect(
        list.getByRole("listitem").filter({ hasText: email }),
      ).toBeVisible();
      await expect(page.getByTestId("created-user")).toHaveCount(0);
      expect(await page.content()).not.toContain(password);
    } finally {
      await deleteUserApi(id);
    }
  });

  test("disable confirms an immediate sign-out; the user's next request is 401 → login; enable has no confirm; delete names the email (FR-075/076)", async ({
    page,
    browser,
  }) => {
    const email = unique("disable");
    const password = "disable-temp-1";
    const { id } = await createUserApi(email, password);
    const userApi = await loginApi(email, password);
    const userContext = await browser.newContext({
      storageState: await userApi.storageState(),
    });
    const userPage = await userContext.newPage();

    try {
      await userPage.goto("/library/");
      await expect(
        userPage.getByRole("heading", { name: "Change password" }),
      ).toBeVisible();

      await openSection(page, "users");
      const row = region(page, "Users")
        .getByRole("listitem")
        .filter({ hasText: email });
      await row.getByRole("button", { name: `Disable ${email}` }).click();
      const dialog = page.getByRole("alertdialog", {
        name: "Disable this user?",
      });
      await expect(dialog).toContainText(email);
      await expect(dialog).toContainText("signed out immediately");
      await dialog.getByRole("button", { name: "Disable user" }).click();
      await expect(dialog).toHaveCount(0);
      await expect(row).toContainText("Member · Disabled");

      expect((await userApi.get("/api/identity/me")).status()).toBe(401);
      await userPage.reload();
      await userPage.waitForURL("**/login/**");
      expect(new URL(userPage.url()).pathname).toBe("/login/");

      // Enable proceeds without a dialog.
      await row.getByRole("button", { name: `Enable ${email}` }).click();
      await expect(page.getByRole("alertdialog")).toHaveCount(0);
      await expect(row).toContainText("Member · Must change password");

      await row.getByRole("button", { name: `Delete ${email}` }).click();
      const remove = page.getByRole("alertdialog", {
        name: "Delete this user?",
      });
      await expect(remove).toContainText(email);
      await expect(remove).toContainText(
        "permanently erases all of this user's data",
      );
      await remove.getByRole("button", { name: "Delete user" }).click();
      await expect(remove).toHaveCount(0);
      await expect(row).toHaveCount(0);
    } finally {
      await userContext.close();
      await userApi.dispose();
      await deleteUserApi(id).catch(() => undefined);
    }
  });

  test("disabling yourself shows the self-disable message; you stay admin (SC-010)", async ({
    page,
  }) => {
    await openSection(page, "users");
    await region(page, "Users")
      .getByRole("button", { name: `Disable ${USERS.admin.email}` })
      .click();
    const dialog = page.getByRole("alertdialog");
    await dialog.getByRole("button", { name: "Disable user" }).click();
    await expect(dialog.getByRole("alert")).toContainText(
      "You cannot disable your own account.",
    );
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toHaveCount(0);

    await page.reload();
    await expect(region(page, "Create user")).toBeVisible();
    const me = await page.request.get("/api/identity/me");
    expect(await me.json()).toMatchObject({ role: "admin" });
  });

  test("deleting a sound category with sounds shows the in-use message (SC-010)", async ({
    page,
  }) => {
    const { weather } = readSeed().soundCategories;
    await openSection(page, "soundscape");
    const categories = region(page, "Sound categories");
    await categories
      .getByRole("button", { name: `Delete ${weather.name}`, exact: true })
      .click();
    const dialog = page.getByRole("alertdialog", {
      name: "Delete this sound category?",
    });
    await expect(dialog).toContainText(weather.name);
    await dialog.getByRole("button", { name: "Delete category" }).click();
    await expect(dialog.getByRole("alert")).toContainText(
      "This category still contains sounds.",
    );
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(
      categories.getByRole("listitem").filter({ hasText: weather.name }),
    ).toHaveCount(1);
  });

  test("sound edit picks the category by name; no numeric field (SC-009 item 6)", async ({
    page,
  }) => {
    const seed = readSeed();
    const rain = seed.sounds.rain;
    const api = await adminApi();
    const categories = (await (
      await expectOk(
        await api.get("/api/soundscape/categories"),
        "sound categories",
      )
    ).json()) as { id: number; name: string }[];
    await api.dispose();

    await openSection(page, "soundscape");
    await region(page, "Sounds")
      .getByRole("button", { name: `Edit ${rain.name}`, exact: true })
      .click();
    const dialog = page.getByRole("dialog", { name: "Edit sound" });
    await expect(dialog.getByLabel("Sound name")).toHaveValue(rain.name);
    const select = dialog.getByRole("combobox", { name: "Category" });
    await expect(select.locator("option")).toHaveText(
      categories.map((category) => category.name),
    );
    const selected = await select.evaluate((element: HTMLSelectElement) =>
      element.selectedOptions[0]?.textContent?.trim(),
    );
    expect(selected).toBe(
      categories.find((category) => category.id === rain.categoryId)?.name,
    );
    await expect(dialog.locator('input[type="number"]')).toHaveCount(0);
    await expect(page.locator('main input[type="number"]')).toHaveCount(0);
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toHaveCount(0);
  });

  test("the delete-sound confirm states removal from every mix (FR-078)", async ({
    page,
  }) => {
    const { rain } = readSeed().sounds;
    await openSection(page, "soundscape");
    const sounds = region(page, "Sounds");
    await sounds
      .getByRole("button", { name: `Delete ${rain.name}`, exact: true })
      .click();
    const dialog = page.getByRole("alertdialog", {
      name: "Delete this sound?",
    });
    await expect(dialog).toContainText(rain.name);
    await expect(dialog).toContainText("removed from every saved mix");
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toHaveCount(0);
    await expect(
      sounds.getByRole("button", { name: `Delete ${rain.name}`, exact: true }),
    ).toBeVisible();
  });

  test("upload meditation: name prefilled from the file, busy text, item in the library (PB-13)", async ({
    page,
  }) => {
    const fixtures = ensureFixtures();
    const api = await adminApi();
    const title = `Admin Upload QA ${Date.now()}`;
    const program = await (
      await expectOk(
        await api.post("/api/meditation/programs", {
          multipart: { title },
        }),
        "create program",
      )
    ).json();
    let meditationId: number | undefined;

    try {
      // Hold the upload briefly so the busy state is observable.
      await page.route("**/api/meditation/audio", async (route) => {
        await new Promise((resolve) => setTimeout(resolve, 1_500));
        await route.continue();
      });
      await openSection(page, "meditations");
      const upload = region(page, "Upload meditation audio");
      await upload.getByLabel("Program").selectOption({ label: title });
      await upload.getByLabel("Audio file").setInputFiles(fixtures.toneMp3);
      await expect(upload.getByLabel("Name", { exact: true })).toHaveValue(
        "Tone 20s",
      );

      // Description counter and over-limit guard.
      const description = upload.getByLabel("Description (optional)");
      await description.fill("x".repeat(2001));
      await expect(upload.getByText("2001 / 2000")).toBeVisible();
      await expect(
        upload.getByRole("button", { name: "Upload" }),
      ).toBeDisabled();
      await description.fill("A calm tone.");
      await expect(upload.getByText("12 / 2000")).toBeVisible();

      const response = page.waitForResponse(
        (r) =>
          r.url().endsWith("/api/meditation/audio") &&
          r.request().method() === "POST",
      );
      await upload.getByRole("button", { name: "Upload" }).click();
      await expect(
        upload.getByRole("button", { name: "Converting…" }),
      ).toBeVisible();
      const uploaded = await response;
      expect(uploaded.ok()).toBe(true);
      meditationId = ((await uploaded.json()) as { id: number }).id;

      await expect(upload.getByRole("status")).toContainText(
        "“Tone 20s” was uploaded",
      );
      const library = region(page, "Library");
      const item = library
        .getByRole("listitem")
        .filter({ hasText: "Tone 20s" });
      await expect(item).toHaveCount(1);
      await expect(
        item.getByRole("link", { name: "Play Tone 20s" }),
      ).toHaveAttribute("href", `/player/?id=${meditationId}`);
      await expect(
        item.getByRole("link", { name: "Download Tone 20s" }),
      ).toHaveAttribute("href", `/api/meditation/${meditationId}/download`);
      await expect(item).toContainText("A calm tone.");

      // Edit the description in the dialog; empty clears it.
      await item
        .getByRole("button", { name: "Edit description of Tone 20s" })
        .click();
      const dialog = page.getByRole("dialog");
      const field = dialog.getByLabel("Description (optional)");
      await expect(field).toHaveValue("A calm tone.");
      await field.fill("x".repeat(2001));
      await expect(dialog.getByRole("button", { name: "Save" })).toBeDisabled();
      await field.fill("Edited text.");
      await dialog.getByRole("button", { name: "Save" }).click();
      await expect(dialog).toHaveCount(0);
      await expect(item).toContainText("Edited text.");
      await item
        .getByRole("button", { name: "Edit description of Tone 20s" })
        .click();
      await dialog.getByLabel("Description (optional)").fill("");
      await dialog.getByRole("button", { name: "Save" }).click();
      await expect(dialog).toHaveCount(0);
      await expect(item).not.toContainText("Edited text.");
    } finally {
      if (meditationId) await api.delete(`/api/meditation/${meditationId}`);
      await api.delete(`/api/meditation/programs/${program.id}`);
      await api.dispose();
    }
  });

  test("meditation category, program category, and add-to-program work from Admin (PB-13)", async ({
    page,
  }) => {
    const stamp = Date.now();
    const categoryName = `QA Category ${stamp}`;
    const title = `QA Program ${stamp}`;
    const { name: meditationName, id: meditationId } =
      readSeed().programs.withCover.meditations[0];
    const api = await adminApi();
    const program = await (
      await expectOk(
        await api.post("/api/meditation/programs", { multipart: { title } }),
        "create program",
      )
    ).json();
    let categoryId: number | undefined;

    try {
      await openSection(page, "meditations");
      // Create a meditation category.
      const form = region(page, "Meditation categories");
      await form.getByLabel("New category name").fill(categoryName);
      const created = page.waitForResponse(
        (r) =>
          r.url().endsWith("/api/meditation/categories") &&
          r.request().method() === "POST",
      );
      await form.getByRole("button", { name: "Add category" }).click();
      expect((await created).status()).toBe(201);
      categoryId = ((await (await created).json()) as { id: number }).id;
      await expect(
        form.getByRole("listitem").filter({ hasText: categoryName }),
      ).toHaveCount(1);

      // Set the program's category.
      const assigned = page.waitForResponse(
        (r) =>
          r.url().endsWith(`/api/meditation/programs/${program.id}/category`) &&
          r.request().method() !== "GET",
      );
      await page
        .getByLabel(`Category for ${title}`)
        .selectOption({ label: categoryName });
      expect((await assigned).ok()).toBe(true);
      await expect(page.getByLabel(`Category for ${title}`)).toHaveValue(
        String(categoryId),
      );
      const stored = (await (
        await api.get(`/api/meditation/programs/${program.id}`)
      ).json()) as { category_id: number | null };
      expect(stored.category_id).toBe(categoryId);

      // Add a meditation to the program.
      const added = page.waitForResponse(
        (r) =>
          r.url().endsWith(`/api/meditation/programs/${program.id}/members`) &&
          r.request().method() === "POST",
      );
      await page
        .getByLabel(`Add ${meditationName} to a program`)
        .selectOption({ label: title });
      expect((await added).ok()).toBe(true);
      const members = (await (
        await api.get(`/api/meditation/programs/${program.id}/members`)
      ).json()) as { id: number }[];
      expect(members.map((member) => member.id)).toContain(meditationId);
    } finally {
      await api.delete(`/api/meditation/programs/${program.id}`);
      if (categoryId)
        await api.delete(`/api/meditation/categories/${categoryId}`);
      await api.dispose();
    }
  });

  test("sound category add and rename, sound upload, built-in badge, and sound download link (PB-13)", async ({
    page,
  }) => {
    const stamp = Date.now();
    const added = `QA Sounds ${stamp}`;
    const renamed = `QA Sounds Renamed ${stamp}`;
    const soundName = `QA Loop ${stamp}`;
    const api = await adminApi();
    let categoryId: number | undefined;
    let soundId: number | undefined;

    try {
      await openSection(page, "soundscape");
      const categories = region(page, "Sound categories");
      await categories.getByLabel("New sound category").fill(added);
      const created = page.waitForResponse(
        (r) =>
          r.url().endsWith("/api/soundscape/categories") &&
          r.request().method() === "POST",
      );
      await categories.getByRole("button", { name: "Add category" }).click();
      const category = await created;
      expect(category.status()).toBe(201);
      categoryId = ((await category.json()) as { id: number }).id;
      await expect(
        categories.getByRole("listitem").filter({ hasText: added }),
      ).toHaveCount(1);

      // Rename it.
      await categories.getByRole("button", { name: `Rename ${added}` }).click();
      const dialog = page.getByRole("dialog", {
        name: "Rename sound category",
      });
      await expect(dialog.getByLabel("Name")).toHaveValue(added);
      await dialog.getByLabel("Name").fill(renamed);
      await dialog.getByRole("button", { name: "Save" }).click();
      await expect(dialog).toHaveCount(0);
      await expect(
        categories.getByRole("listitem").filter({ hasText: renamed }),
      ).toHaveCount(1);
      await expect(
        categories.getByRole("listitem").filter({ hasText: added }),
      ).toHaveCount(0);

      // Upload a sound into it.
      const sounds = region(page, "Sounds");
      await sounds.getByLabel("Sound name").fill(soundName);
      await sounds.getByLabel("Category").selectOption({ label: renamed });
      await sounds
        .getByLabel("Audio file")
        .setInputFiles(ensureFixtures().loopOgg);
      const uploaded = page.waitForResponse(
        (r) =>
          r.url().endsWith("/api/soundscape/sounds") &&
          r.request().method() === "POST",
      );
      await sounds.getByRole("button", { name: "Upload sound" }).click();
      const sound = await uploaded;
      expect(sound.ok()).toBe(true);
      soundId = ((await sound.json()) as { id: number }).id;

      const row = sounds.getByRole("listitem").filter({ hasText: soundName });
      await expect(row).toHaveCount(1);
      await expect(row).toContainText(renamed);
      await expect(row).not.toContainText("Built-in");
      // Download link: a same-origin API address.
      const download = row.getByRole("link", { name: `Download ${soundName}` });
      await expect(download).toHaveAttribute(
        "href",
        `/api/soundscape/sounds/${soundId}/download`,
      );

      // A built-in sound carries the badge and its own download link.
      const all = (await (await api.get("/api/soundscape/sounds")).json()) as {
        id: number;
        name: string;
        builtin: boolean;
      }[];
      const builtin = all.find((item) => item.builtin);
      expect(builtin).toBeDefined();
      const builtinRow = sounds
        .getByRole("listitem")
        .filter({ hasText: builtin!.name })
        .filter({ hasText: "Built-in" });
      await expect(builtinRow.first()).toBeVisible();
      await expect(
        builtinRow
          .first()
          .getByRole("link", { name: `Download ${builtin!.name}` }),
      ).toHaveAttribute(
        "href",
        `/api/soundscape/sounds/${builtin!.id}/download`,
      );
    } finally {
      if (soundId) await api.delete(`/api/soundscape/sounds/${soundId}`);
      if (categoryId)
        await api.delete(`/api/soundscape/categories/${categoryId}`);
      await api.dispose();
    }
  });

  test("a program cover over 5 MB shows the specific reason", async ({
    page,
  }) => {
    const cover = ensureOversizeCover();
    const title = `Oversize Cover QA ${Date.now()}`;
    await openSection(page, "meditations");
    const form = region(page, "Create program");
    await form.getByLabel("Title").fill(title);
    await form.getByLabel("Cover image (optional)").setInputFiles(cover);
    const response = page.waitForResponse(
      (r) =>
        r.url().endsWith("/api/meditation/programs") &&
        r.request().method() === "POST",
    );
    await form.getByRole("button", { name: "Create program" }).click();
    expect((await response).status()).toBe(413);
    await expect(form.getByRole("alert")).toContainText(
      "The file is larger than 5 MB.",
    );
    await expect(form.getByLabel("Title")).toHaveValue(title);

    const programs = (await (
      await page.request.get("/api/meditation/programs")
    ).json()) as { title: string }[];
    expect(programs.map((program) => program.title)).not.toContain(title);
  });

  test("no admin request reads another user's mood, notes, or timeline (PB-17)", async ({
    page,
    networkGuard,
  }) => {
    for (const section of ["users", "meditations", "soundscape"] as const)
      await openSection(page, section);
    await page.waitForLoadState("networkidle");
    const calls = apiCalls(networkGuard.requests);
    saveJson(TASK, "admin-walkthrough-requests", calls);
    expect(calls.filter((call) => PRIVATE_DATA.test(call.path))).toEqual([]);
    expect(calls.some((call) => call.path === "/api/identity/users")).toBe(
      true,
    );
  });
});

test.describe("admin in Turkish (SC-023)", () => {
  test.use({ storageState: storageStatePath("admin"), ...localeOptions("tr") });

  test("duplicate email, self-disable, and category in use appear in Turkish", async ({
    page,
  }) => {
    await openSection(page, "users");
    const form = region(page, "Kullanıcı oluştur");
    await form.getByLabel("E-posta").fill(USERS.member.email);
    await form.getByLabel("Geçici şifre").fill("duplicate-temp-1");
    await form.getByRole("button", { name: "Kullanıcı oluştur" }).click();
    await expect(form.getByRole("alert")).toContainText(
      "Bu e-posta adresiyle bir hesap zaten var.",
    );

    await region(page, "Kullanıcılar")
      .getByRole("button", { name: `${USERS.admin.email}: devre dışı bırak` })
      .click();
    const disable = page.getByRole("alertdialog");
    await expect(disable).toContainText("Hemen oturumları kapatılır");
    await disable
      .getByRole("button", { name: "Kullanıcıyı devre dışı bırak" })
      .click();
    await expect(disable.getByRole("alert")).toContainText(
      "Kendi hesabınızı devre dışı bırakamazsınız.",
    );
    await disable.getByRole("button", { name: "İptal" }).click();

    const { weather } = readSeed().soundCategories;
    await openSection(page, "soundscape");
    await region(page, "Ses kategorileri")
      .getByRole("button", { name: `${weather.name} sil`, exact: true })
      .click();
    const remove = page.getByRole("alertdialog");
    await remove.getByRole("button", { name: "Kategoriyi sil" }).click();
    await expect(remove.getByRole("alert")).toContainText(
      "Bu kategoride hâlâ sesler var. Önce onları taşıyın veya silin.",
    );
    await remove.getByRole("button", { name: "İptal" }).click();
  });
});

for (const locale of ["en", "tr"] as const) {
  test.describe(`screenshots (${locale})`, () => {
    test.use({
      storageState: storageStatePath("admin"),
      ...localeOptions(locale),
    });

    test("each section at 1440 and 375 px", async ({ page }) => {
      for (const section of ["users", "meditations", "soundscape"] as const) {
        for (const width of [1440, 375] as const) {
          await page.setViewportSize(viewport(width));
          await openSection(page, section);
          await saveScreenshot(
            page,
            TASK,
            `admin-${section}-${locale}-${width}`,
          );
        }
      }
    });
  });
}
