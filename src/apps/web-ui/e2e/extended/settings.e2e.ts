// T23: the Settings screen (US5, US7, US9; FR-066–FR-070, FR-103/104,
// FR-112/113/115, PB-12, PB-16, SC-010, SC-016, SC-022, SC-023).
// Tests that change a password, sign out, upload, import, or delete use a
// dedicated fresh user, so the shared admin/member sessions stay valid.
import fs from "node:fs";
import path from "node:path";
import type { BrowserContext, Page } from "@playwright/test";
import { expectOk, getJson, newApi, postJson } from "../support/api-client";
import {
  loginApi,
  storageStatePath,
  USERS,
  type UserDto,
} from "../support/auth";
import { saveJson, saveScreenshot } from "../support/evidence";
import { localeOptions } from "../support/locale";
import { baseURL, fixturesDir } from "../support/server-env";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";

const signedOut = { cookies: [], origins: [] };
const fixture = (name: string) => path.join(fixturesDir, name);

const COPY = {
  en: {
    headings: [
      "Profile",
      "Language",
      "Password",
      "Your data",
      "Delete account",
    ],
    profile: "Profile",
    deleteAccount: "Delete account",
    tooLarge:
      "This image is larger than 2 MB. Choose a JPEG, PNG, or WebP image of at most 2 MB.",
    lastAdmin:
      "You are the last active administrator, so this would leave the instance without one.",
  },
  tr: {
    headings: ["Profil", "Dil", "Parola", "Verileriniz", "Hesabı sil"],
    profile: "Profil",
    deleteAccount: "Hesabı sil",
    tooLarge:
      "Bu görsel 2 MB'tan büyük. En fazla 2 MB'lık bir JPEG, PNG veya WebP görseli seçin.",
    lastAdmin:
      "Son etkin yönetici sizsiniz; bu işlem kurulumu yöneticisiz bırakır.",
  },
} as const;

const REPORT_LABELS = [
  "Mood entries added",
  "Mood entries skipped as duplicates",
  "Practice records linked",
  "Practice records skipped (no matching meditation)",
  "Practice records skipped as duplicates",
];

const panel = (page: Page, name: string) =>
  page.getByRole("region", { name, exact: true });

const avatarInput = (page: Page) =>
  page.locator('input[type="file"][accept^="image/"]');

const sidebar = (page: Page) =>
  page.getByRole("navigation", { name: "Main navigation" });

async function openSettings(page: Page, profile: string = COPY.en.profile) {
  await page.goto("/settings/");
  await expect(
    page.getByRole("heading", { level: 2, name: profile }),
  ).toBeVisible();
}

/** A new member with a permanent password, created through the admin API. */
async function createFreshUser(
  tag: string,
): Promise<{ email: string; password: string }> {
  const email = `t23-${tag}-${Date.now()}@example.test`;
  const temporary = "t23-e2e-temp-1";
  const password = "t23-e2e-pass-1";
  const admin = await newApi(storageStatePath("admin"));
  await postJson(admin, "/api/identity/users", { email, password: temporary });
  await admin.dispose();
  const first = await loginApi(email, temporary);
  await postJson(first, "/api/identity/me/password", { password });
  await first.dispose();
  return { email, password };
}

/** A new session for this context only. */
async function signInFresh(
  context: BrowserContext,
  email: string,
  password: string,
): Promise<void> {
  const api = await loginApi(email, password);
  await context.addCookies((await api.storageState()).cookies);
  await api.dispose();
}

async function signOutThroughShell(page: Page): Promise<void> {
  await sidebar(page).getByRole("button", { name: "Sign out" }).click();
  await page
    .getByRole("alertdialog")
    .getByRole("button", { name: "Sign out" })
    .click();
  await page.waitForURL((url) => url.pathname === "/login/");
}

async function signInThroughLogin(
  page: Page,
  email: string,
  password: string,
): Promise<void> {
  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Password", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Sign in" }).click();
  await page.waitForURL((url) => url.pathname === "/");
}

/**
 * API requests the page made after `from` (an index into the guard's log).
 * Static assets (fonts first used by the refusal text) are not server calls.
 */
const requestsSince = (
  requests: { method: string; url: string }[],
  from: number,
) =>
  requests
    .slice(from)
    .filter((r) => new URL(r.url).pathname.startsWith("/api/"))
    .map((r) => `${r.method} ${r.url}`);

for (const locale of ["en", "tr"] as const) {
  test.describe(`member (${locale})`, () => {
    test.use({
      storageState: storageStatePath("member"),
      ...localeOptions(locale),
    });

    test("panels in order; a 3 MB png is refused before any request (SC-010); screenshots", async ({
      page,
      networkGuard,
    }) => {
      await openSettings(page, COPY[locale].profile);
      await expect(page.locator("main h2")).toHaveText([
        ...COPY[locale].headings,
      ]);
      await expect(
        panel(page, COPY[locale].deleteAccount).locator(
          '[data-variant="primary"]',
        ),
      ).toHaveCount(0);

      for (const width of [1440, 375] as const) {
        await page.setViewportSize(viewport(width));
        // Headless Chromium rasterizes the blurred glass panels lazily: let
        // them paint before each capture.
        await page.evaluate(() => window.scrollTo(0, 0));
        await page.waitForTimeout(500);
        await saveScreenshot(page, "T23", `settings-${locale}-${width}`);
        // Also capture the lower panels in view.
        await panel(page, COPY[locale].deleteAccount).scrollIntoViewIfNeeded();
        await page.waitForTimeout(500);
        await saveScreenshot(
          page,
          "T23",
          `settings-${locale}-${width}-bottom`,
          false,
        );
      }
      await page.setViewportSize(viewport(1440));

      const before = networkGuard.requests.length;
      await avatarInput(page).setInputFiles(fixture("avatar-3mb.png"));
      await expect(
        panel(page, COPY[locale].profile).getByRole("alert"),
      ).toHaveText(COPY[locale].tooLarge);
      await page.waitForTimeout(750);
      const after = requestsSince(networkGuard.requests, before);
      saveJson("T23", `avatar-3mb-request-count-${locale}`, {
        fixture: "avatar-3mb.png",
        bytes: fs.statSync(fixture("avatar-3mb.png")).size,
        apiRequestsAfterChoosing: after.length,
        allRequests: networkGuard.requests
          .slice(before)
          .map((r) => `${r.method} ${r.url}`),
        requests: after,
      });
      expect(after).toEqual([]);
    });
  });
}

test.describe("member", () => {
  test.use({ storageState: storageStatePath("member") });

  test("a GIF is refused before any request, stating the types and limit", async ({
    page,
    networkGuard,
  }) => {
    await openSettings(page);
    const before = networkGuard.requests.length;
    await avatarInput(page).setInputFiles({
      name: "avatar.gif",
      mimeType: "image/gif",
      buffer: Buffer.from("GIF89a"),
    });
    await expect(panel(page, "Profile").getByRole("alert")).toHaveText(
      "This file is not a JPEG, PNG, or WebP image. Choose an image of one of these types, at most 2 MB.",
    );
    await page.waitForTimeout(500);
    expect(requestsSince(networkGuard.requests, before)).toEqual([]);
  });

  test("export links are real same-origin downloads labeled by scope (PB-16)", async ({
    page,
  }) => {
    await openSettings(page);
    const data = panel(page, "Your data");
    const json = data.getByRole("link", { name: "Download all data (JSON)" });
    const csv = data.getByRole("link", {
      name: "Download mood entries only (CSV)",
    });
    const origin = new URL(baseURL()).origin;
    for (const [link, href] of [
      [json, "/api/portability/export/json"],
      [csv, "/api/portability/export/csv"],
    ] as const) {
      await expect(link).toHaveAttribute("href", href);
      await expect(link).toHaveAttribute("download", "");
      const resolved = new URL((await link.getAttribute("href"))!, page.url());
      expect(resolved.origin).toBe(origin);
      expect(await link.evaluate((a) => a.closest("button"))).toBeNull();
    }
    const download = page.waitForEvent("download");
    await csv.click();
    expect((await download).url()).toBe(`${origin}/api/portability/export/csv`);
  });

  test("bad.json shows the specific reason and no report", async ({ page }) => {
    await openSettings(page);
    const data = panel(page, "Your data");
    await data
      .getByLabel("Import a previous export (JSON or CSV)")
      .setInputFiles(fixture("bad.json"));
    const response = page.waitForResponse("**/api/portability/import");
    await data.getByRole("button", { name: "Import", exact: true }).click();
    expect((await response).status()).toBe(400);
    await expect(data.getByRole("alert")).toHaveText(
      "This file is not a MediOSS export.",
    );
    await expect(
      data.getByRole("heading", { name: "Import report" }),
    ).toHaveCount(0);
  });

  test("a 7-character password and a mismatch are refused with no request (FR-103)", async ({
    page,
    networkGuard,
  }) => {
    await openSettings(page);
    const password = panel(page, "Password");
    const before = networkGuard.requests.length;

    await password.getByLabel("New password", { exact: true }).fill("short-7");
    await password.getByLabel("Confirm new password").fill("short-7");
    await password.getByRole("button", { name: "Change password" }).click();
    await expect(password.getByRole("alert")).toHaveText(
      "The password must be at least 8 characters long.",
    );

    await password
      .getByLabel("New password", { exact: true })
      .fill("long-enough-1");
    await password.getByLabel("Confirm new password").fill("long-enough-2");
    await password.getByRole("button", { name: "Change password" }).click();
    await expect(password.getByRole("alert")).toHaveText(
      "The two passwords do not match.",
    );

    await page.waitForTimeout(500);
    const after = requestsSince(networkGuard.requests, before);
    saveJson("T23", "password-refusals-request-count", {
      apiRequestsAfterSubmitting: after.length,
      allRequests: networkGuard.requests
        .slice(before)
        .map((r) => `${r.method} ${r.url}`),
      requests: after,
    });
    expect(after).toEqual([]);
  });
});

for (const locale of ["en", "tr"] as const) {
  test.describe(`only admin (${locale})`, () => {
    test.use({
      storageState: storageStatePath("admin"),
      ...localeOptions(locale),
    });

    test("deleting their own account shows the last-active-admin message and stays signed in (SC-010, FR-070)", async ({
      page,
    }) => {
      // Safety: this must only ever run against the single seeded admin.
      const users = await getJson<(UserDto & { disabled: boolean })[]>(
        page.request,
        "/api/identity/users",
      );
      expect(
        users.filter((u) => u.role === "admin" && !u.disabled),
      ).toHaveLength(1);

      await openSettings(page, COPY[locale].profile);
      const danger = panel(page, COPY[locale].deleteAccount);
      const button = danger.getByRole("button", {
        name: COPY[locale].deleteAccount,
      });
      await expect(button).toBeDisabled();
      await danger.getByRole("textbox").fill(`${USERS.admin.email} `);
      await expect(button).toBeDisabled();
      await danger.getByRole("textbox").fill(USERS.admin.email);
      await button.click();

      const dialog = page.getByRole("alertdialog");
      const refused = page.waitForResponse(
        (r) =>
          r.url().endsWith("/api/identity/me") &&
          r.request().method() === "DELETE",
      );
      await dialog
        .getByRole("button", { name: COPY[locale].deleteAccount })
        .click();
      expect((await refused).status()).toBe(400);

      await expect(dialog).toHaveCount(0);
      await expect(danger.getByRole("alert")).toHaveText(
        COPY[locale].lastAdmin,
      );
      expect(new URL(page.url()).pathname).toBe("/settings/");
      expect((await page.request.get("/api/identity/me")).status()).toBe(200);
    });
  });
}

test.describe("fresh user", () => {
  test.use({ storageState: signedOut });

  test("avatar upload, remove cancel keeps it, remove; display name saves and clears", async ({
    page,
    context,
  }) => {
    const user = await createFreshUser("avatar");
    await signInFresh(context, user.email, user.password);
    await openSettings(page);
    const profile = panel(page, "Profile");
    const preview = profile.getByRole("img", { name: "Your profile picture" });
    const accountImage = page
      .getByRole("link", { name: "Account settings" })
      .first()
      .locator("img");
    await expect(preview).toHaveCount(0);
    await expect(
      profile.getByRole("button", { name: "Remove picture" }),
    ).toHaveCount(0);

    const uploaded = page.waitForResponse("**/api/identity/me/avatar");
    await avatarInput(page).setInputFiles(fixture("cover-640x480.png"));
    expect((await uploaded).ok()).toBe(true);
    await expect(preview).toBeVisible();
    await expect(accountImage).toBeVisible();

    // Cancel keeps the picture.
    await profile.getByRole("button", { name: "Remove picture" }).click();
    const dialog = page.getByRole("alertdialog", {
      name: "Remove your profile picture?",
    });
    await dialog.getByRole("button", { name: "Cancel" }).click();
    await expect(dialog).toHaveCount(0);
    await expect(preview).toBeVisible();
    let me = await getJson<UserDto>(page.request, "/api/identity/me");
    expect(me.has_avatar).toBe(true);

    // Confirm removes it.
    await profile.getByRole("button", { name: "Remove picture" }).click();
    await dialog.getByRole("button", { name: "Remove picture" }).click();
    await expect(dialog).toHaveCount(0);
    await expect(preview).toHaveCount(0);
    await expect(accountImage).toHaveCount(0);
    me = await getJson<UserDto>(page.request, "/api/identity/me");
    expect(me.has_avatar).toBe(false);

    // Display name: max 64, email placeholder, saves, then clears.
    const name = profile.getByLabel("Display name");
    await expect(name).toHaveAttribute("maxlength", "64");
    await expect(name).toHaveAttribute("placeholder", user.email);
    await name.fill("zeynep");
    await profile.getByRole("button", { name: "Save display name" }).click();
    await expect(
      profile.getByText("Your display name has been saved."),
    ).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Account settings" }).first(),
    ).toHaveText("Z");
    me = await getJson<UserDto>(page.request, "/api/identity/me");
    expect(me.display_name).toBe("zeynep");

    await name.fill("");
    await profile.getByRole("button", { name: "Save display name" }).click();
    await expect(
      profile.getByText("Your display name has been saved."),
    ).toBeVisible();
    me = await getJson<UserDto>(page.request, "/api/identity/me");
    expect(me.display_name).toBeNull();
  });

  test("the import report lists all 5 counts (FR-069)", async ({
    page,
    context,
  }) => {
    const member = await loginApi(USERS.member.email, USERS.member.password);
    const exported = await expectOk(
      await member.get("/api/portability/export/json"),
      "export member json",
    );
    const buffer = await exported.body();
    await member.dispose();

    const user = await createFreshUser("import");
    await signInFresh(context, user.email, user.password);
    await openSettings(page);
    const data = panel(page, "Your data");
    await data
      .getByLabel("Import a previous export (JSON or CSV)")
      .setInputFiles({
        name: "export.json",
        mimeType: "application/json",
        buffer,
      });
    const response = page.waitForResponse("**/api/portability/import");
    await data.getByRole("button", { name: "Import", exact: true }).click();
    const report = (await (await response).json()) as Record<string, number>;

    const section = data.getByRole("region", { name: "Import report" });
    await expect(section.locator("dt")).toHaveText(REPORT_LABELS);
    await expect(section.locator("dd")).toHaveText(
      [
        report.mood_created,
        report.mood_duplicates_skipped,
        report.practice_linked,
        report.practice_skipped,
        report.practice_duplicates_skipped,
      ].map(String),
    );
    expect(report.mood_created).toBeGreaterThan(0);
  });

  test("a valid password change keeps the screen; sign-in with the new password works (SC-016)", async ({
    page,
    context,
  }) => {
    const user = await createFreshUser("password");
    const next = "t23-e2e-new-pass-2";
    await signInFresh(context, user.email, user.password);
    await openSettings(page);
    const password = panel(page, "Password");
    await password.getByLabel("New password", { exact: true }).fill(next);
    await password.getByLabel("Confirm new password").fill(next);
    const changed = page.waitForResponse("**/api/identity/me/password");
    await password.getByRole("button", { name: "Change password" }).click();
    expect((await changed).status()).toBe(200);

    await expect(
      password.getByLabel("New password", { exact: true }),
    ).toHaveValue("");
    await expect(panel(page, "Profile")).toBeVisible();
    expect(new URL(page.url()).pathname).toBe("/settings/");
    expect((await page.request.get("/api/identity/me")).status()).toBe(200);

    await signOutThroughShell(page);
    await signInThroughLogin(page, user.email, next);
    expect((await page.request.get("/api/identity/me")).status()).toBe(200);
  });

  test("delete account: exact email, confirm, then login with no next (PB-12)", async ({
    page,
    context,
  }) => {
    const user = await createFreshUser("delete");
    await signInFresh(context, user.email, user.password);
    await openSettings(page);
    const danger = panel(page, "Delete account");
    const button = danger.getByRole("button", { name: "Delete account" });
    await danger.getByRole("textbox").fill(user.email.toUpperCase());
    await expect(button).toBeDisabled();
    await danger.getByRole("textbox").fill(user.email);
    await button.click();
    const dialog = page.getByRole("alertdialog", {
      name: "Delete your account?",
    });
    await expect(dialog).toContainText(user.email);
    await dialog.getByRole("button", { name: "Delete account" }).click();
    await page.waitForURL((url) => url.pathname === "/login/");
    expect(new URL(page.url()).search).toBe("");
    expect((await page.request.get("/api/identity/me")).status()).toBe(401);
  });
});

test.describe("language (fresh user, Turkish browser)", () => {
  test.use({ storageState: signedOut, ...localeOptions("tr") });

  test("switching to English keeps an unsaved name; survives reload and sign-out/sign-in 3 times (FR-113, FR-115, SC-022)", async ({
    page,
    context,
  }) => {
    const user = await createFreshUser("language");
    await signInFresh(context, user.email, user.password);
    await openSettings(page, "Profil");
    await expect(page.locator("html")).toHaveAttribute("lang", "tr");

    const typed = "Henüz kaydedilmedi";
    await panel(page, "Profil").getByLabel("Görünen ad").fill(typed);
    await page.locator('label[lang="en"]').click();
    await expect(page.getByRole("radio", { name: "English" })).toBeChecked();
    await expect(
      page.getByRole("heading", { level: 2, name: "Profile" }),
    ).toBeVisible();
    await expect(page.locator("main h2")).toHaveText([...COPY.en.headings]);
    await expect(panel(page, "Profile").getByLabel("Display name")).toHaveValue(
      typed,
    );
    await expect(page.locator("html")).toHaveAttribute("lang", "en");
    expect((await page.request.get("/api/identity/me")).status()).toBe(200);

    await page.reload();
    await expect(
      page.getByRole("heading", { level: 2, name: "Profile" }),
    ).toBeVisible();

    for (let round = 1; round <= 3; round++) {
      await signOutThroughShell(page);
      await expect(page.locator("html")).toHaveAttribute("lang", "en");
      await signInThroughLogin(page, user.email, user.password);
      await openSettings(page);
      await expect(page.locator("html")).toHaveAttribute("lang", "en");
      await expect(page.getByRole("radio", { name: "English" })).toBeChecked();
    }
  });
});
