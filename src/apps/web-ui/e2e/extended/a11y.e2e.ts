// T25: axe-core (WCAG 2.0 A/AA + 2.1 AA) on every screen, the forced
// change-password view, and every dialog/sheet, in English and Turkish
// (SC-012). Zero serious or critical violations. Dialogs are only opened and
// cancelled, so no shared data changes. JSON evidence goes to
// .agents/evidence/ui-ux-design/T25/a11y/.
import fs from "node:fs";
import path from "node:path";
import type { Locator, Page } from "@playwright/test";
import { newApi, postJson } from "../support/api-client";
import { scanA11y } from "../support/a11y";
import { loginApi, storageStatePath, USERS } from "../support/auth";
import { evidenceDir } from "../support/evidence";
import { localeOptions, type AppLocale } from "../support/locale";
import { readSeed, SEED } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { viewport } from "../support/viewport";
import {
  deleteAccountMessages,
  profileMessages,
} from "../../src/lib/screens/settings/settings.messages";
import {
  soundAdminMessages,
  userMessages,
} from "../../src/lib/screens/admin/admin.messages";
import { playerMessages } from "../../src/lib/screens/player/player.messages";
import { soundscapeMessages } from "../../src/lib/screens/soundscape/soundscape.messages";
import { navMessages } from "../../src/lib/i18n/common.messages";
import { signOutMessages } from "../../src/lib/components/shell/shell.messages";
import { FIXTURES } from "../support/fixtures-media";

type Copy = Record<AppLocale, string | ((p: any) => string)>;

/** Resolves a co-located message for `locale`, filling `{param}` placeholders. */
function copy(
  message: Copy,
  locale: AppLocale,
  params: Record<string, string | number> = {},
): string {
  const entry = message[locale];
  if (typeof entry === "function") return entry(params);
  return entry.replace(/\{(\w+)\}/g, (_, key) => String(params[key]));
}

const dialogs = (page: Page): Locator =>
  page.locator('[role="dialog"], [role="alertdialog"]');

/** Waits for the screen's data to load and for animations to finish. */
async function settle(page: Page): Promise<void> {
  await page.waitForLoadState("networkidle");
  await expect(page.locator('[data-region-state="loading"]')).toHaveCount(0);
  await page.evaluate(() =>
    Promise.all(
      document
        .getAnimations()
        .filter(
          (animation) =>
            animation.effect?.getComputedTiming().iterations !== Infinity,
        )
        .map((animation) => animation.finished.catch(() => undefined)),
    ),
  );
}

async function scan(page: Page, name: string): Promise<void> {
  await settle(page);
  const violations = await scanA11y(page, { task: "T25", name });
  // scanA11y writes into T25/; keep the per-scan evidence under T25/a11y/.
  const dir = evidenceDir("T25");
  const target = path.join(dir, "a11y");
  fs.mkdirSync(target, { recursive: true });
  const file = `axe-${name.replace(/[^a-zA-Z0-9._-]+/g, "-")}.json`;
  fs.renameSync(path.join(dir, file), path.join(target, file));
  expect(violations, JSON.stringify(violations, null, 2)).toEqual([]);
}

async function scanDialog(page: Page, name: string): Promise<void> {
  await expect(dialogs(page)).toHaveCount(1);
  await scan(page, name);
  await page.keyboard.press("Escape");
  await expect(dialogs(page)).toHaveCount(0);
}

/** A new member with a permanent password, so avatar/delete never touch shared users. */
async function createFreshUser(
  tag: string,
): Promise<{ email: string; password: string }> {
  const email = `t25-${tag}-${Date.now()}@example.test`;
  const temporary = "t25-e2e-temp-1";
  const password = "t25-e2e-pass-1";
  const admin = await newApi(storageStatePath("admin"));
  await postJson(admin, "/api/identity/users", { email, password: temporary });
  await admin.dispose();
  const first = await loginApi(email, temporary);
  await postJson(first, "/api/identity/me/password", { password });
  await first.dispose();
  return { email, password };
}

for (const locale of ["en", "tr"] as const) {
  test.describe(`a11y (${locale})`, () => {
    test.use(localeOptions(locale));

    test.describe("member screens", () => {
      test.use({ storageState: storageStatePath("member") });

      test("screens, flagged view and dialogs have no serious or critical violations", async ({
        page,
      }) => {
        test.setTimeout(240_000);
        const seed = readSeed();
        const memberScreens: [string, string][] = [
          ["home", "/"],
          ["library", "/library/"],
          ["program", `/program/?id=${seed.programs.withCover.id}`],
          ["player", `/player/?id=${seed.meditationNotes[0].meditationId}`],
          ["soundscape", "/soundscape/"],
          ["mood", "/mood/"],
          ["history", "/history/"],
          ["stats", "/stats/"],
          ["settings", "/settings/"],
        ];
        for (const [name, url] of memberScreens) {
          await page.goto(url);
          await scan(page, `${locale}-screen-${name}`);
        }

        // Sign-out (shell).
        await page.goto("/stats/");
        await settle(page);
        await page
          .getByRole("navigation")
          .getByRole("button", { name: copy(navMessages.signOut, locale) })
          .first()
          .click();
        await expect(
          page.getByRole("alertdialog", {
            name: copy(signOutMessages.title, locale),
          }),
        ).toBeVisible();
        await scanDialog(page, `${locale}-dialog-sign-out`);

        // Note delete (player).
        await page.goto(`/player/?id=${seed.meditationNotes[0].meditationId}`);
        await settle(page);
        await page
          .getByRole("button", {
            name: copy(playerMessages.deleteNote, locale),
          })
          .first()
          .click();
        await scanDialog(page, `${locale}-dialog-note-delete`);

        // Mix rename + delete (soundscape).
        await page.goto("/soundscape/");
        await settle(page);
        const mix = { name: SEED.mixes.evening };
        await page
          .getByRole("button", {
            name: copy(soundscapeMessages.renameMix, locale, mix),
          })
          .click();
        await scanDialog(page, `${locale}-dialog-mix-rename`);
        await page
          .getByRole("button", {
            name: copy(soundscapeMessages.deleteMix, locale, mix),
          })
          .click();
        await scanDialog(page, `${locale}-dialog-mix-delete`);

        // Mood delete.
        await page.goto("/mood/");
        await settle(page);
        await page
          .getByRole("button", {
            name: locale === "en" ? /^Delete entry from/ : /kaydını sil$/,
          })
          .first()
          .click();
        await scanDialog(page, `${locale}-dialog-mood-delete`);

        // More sheet (375px).
        await page.setViewportSize(viewport(375));
        await page.goto("/stats/");
        await settle(page);
        await page
          .getByRole("button", { name: copy(navMessages.more, locale) })
          .click();
        await scanDialog(page, `${locale}-sheet-more-375`);
      });

      test("the forced change-password view has no serious or critical violations", async ({
        page,
      }) => {
        await page.route("**/api/identity/me", async (route) => {
          if (route.request().method() !== "GET") return route.fallback();
          const response = await route.fetch();
          const body = (await response.json()) as Record<string, unknown>;
          await route.fulfill({
            response,
            json: { ...body, must_change_password: true },
          });
        });
        await page.goto("/");
        await expect(
          page.locator("main form input[type=password]").first(),
        ).toBeVisible();
        await scan(page, `${locale}-screen-change-password`);
      });
    });

    test.describe("login", () => {
      test.use({ storageState: { cookies: [], origins: [] } });

      test("login has no serious or critical violations", async ({ page }) => {
        await page.goto("/login/");
        await expect(page.locator("form input[type=password]")).toBeVisible();
        await scan(page, `${locale}-screen-login`);
      });
    });

    test.describe("admin", () => {
      test.use({ storageState: storageStatePath("admin") });

      test("admin screen and dialogs have no serious or critical violations", async ({
        page,
      }) => {
        test.setTimeout(180_000);
        const seed = readSeed();
        await page.goto("/admin/");
        await scan(page, `${locale}-screen-admin`);

        await page.goto("/admin/?section=users");
        await settle(page);
        const email = { email: USERS.member.email };
        await page
          .getByRole("button", {
            name: copy(userMessages.disableNamed, locale, email),
          })
          .click();
        await scanDialog(page, `${locale}-dialog-user-disable`);
        await page
          .getByRole("button", {
            name: copy(userMessages.deleteNamed, locale, email),
          })
          .click();
        await scanDialog(page, `${locale}-dialog-user-delete`);

        await page.goto("/admin/?section=soundscape");
        await settle(page);
        const category = { name: seed.soundCategories.studio.name };
        await page
          .getByRole("button", {
            name: copy(soundAdminMessages.renameNamed, locale, category),
            exact: true,
          })
          .click();
        await scanDialog(page, `${locale}-dialog-category-rename`);
        await page
          .getByRole("button", {
            name: copy(soundAdminMessages.deleteNamed, locale, category),
            exact: true,
          })
          .click();
        await scanDialog(page, `${locale}-dialog-category-delete`);

        const sound = { name: seed.sounds.rain.name };
        const sounds = page.getByRole("region", {
          name: locale === "en" ? "Sounds" : "Sesler",
          exact: true,
        });
        await sounds
          .getByRole("button", {
            name: copy(soundAdminMessages.editNamed, locale, sound),
            exact: true,
          })
          .click();
        await scanDialog(page, `${locale}-dialog-sound-edit`);
        await sounds
          .getByRole("button", {
            name: copy(soundAdminMessages.deleteNamed, locale, sound),
            exact: true,
          })
          .click();
        await scanDialog(page, `${locale}-dialog-sound-delete`);
      });
    });

    test.describe("settings dialogs (fresh user)", () => {
      test.use({ storageState: { cookies: [], origins: [] } });

      test("avatar remove and account delete dialogs have no serious or critical violations", async ({
        page,
        context,
      }) => {
        const user = await createFreshUser(`a11y-${locale}`);
        const api = await loginApi(user.email, user.password);
        await context.addCookies((await api.storageState()).cookies);
        await api.dispose();

        await page.goto("/settings/");
        await settle(page);
        await page
          .locator('input[type="file"][accept^="image/"]')
          .setInputFiles(FIXTURES.coverPng);
        const remove = page.getByRole("button", {
          name: copy(profileMessages.removePicture, locale),
        });
        await expect(remove).toBeVisible();
        await remove.click();
        await scanDialog(page, `${locale}-dialog-avatar-remove`);

        const danger = page.getByRole("region", {
          name: copy(deleteAccountMessages.heading, locale),
          exact: true,
        });
        await danger.getByRole("textbox").fill(user.email);
        await danger
          .getByRole("button", {
            name: copy(deleteAccountMessages.action, locale),
          })
          .click();
        await scanDialog(page, `${locale}-dialog-account-delete`);
      });
    });
  });
}
