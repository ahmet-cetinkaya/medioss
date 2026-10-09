// T25: language sweep (plan T25; SC-019, SC-022, SC-023, FR-102). For en and tr
// every screen, dialog, and the flagged view shows no other-language and no
// id-like string; Turkish dates, times, numbers, and durations equal Intl
// output; first visits pick the language from the browser (FR-114).
import fs from "node:fs";
import path from "node:path";
import type { Locator, Page } from "@playwright/test";
import { storageStatePath, USERS } from "../support/auth";
import { guardNetwork } from "../support/network-guard";
import { localeOptions, NEW_YORK, type AppLocale } from "../support/locale";
import { readSeed, textSweepExclusions } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";
import { sweepText, type TextSweepInput } from "../support/text-sweep";
import { builtinSoundsDir } from "../support/server-env";
import { viewport } from "../support/viewport";

type AnyMessage = { en: unknown; tr: unknown };
const srcDir = path.resolve(__dirname, "../../src");

function messageFiles(dir: string): string[] {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) return messageFiles(full);
    return entry.name.endsWith(".messages.ts") ? [full] : [];
  });
}

/** Every catalog message, by file and export path; functions are sampled. */
const catalog: { key: string; message: AnyMessage }[] = [];
function collect(value: unknown, key: string): void {
  if (!value || typeof value !== "object") return;
  if ("en" in value && "tr" in value) {
    catalog.push({ key, message: value as AnyMessage });
    return;
  }
  for (const [name, child] of Object.entries(value))
    collect(child, `${key}.${name}`);
}

const SAMPLES = [1, 2, 5];

function values(locale: AppLocale): string[] {
  const out: string[] = [];
  for (const { message } of catalog) {
    const entry = message[locale];
    if (typeof entry === "string") out.push(entry);
    else if (typeof entry === "function") {
      for (const sample of SAMPLES) {
        try {
          const text = entry(new Proxy({}, { get: () => sample }));
          if (typeof text === "string" && text.trim()) out.push(text);
        } catch {
          // A parameter shape the sample cannot satisfy; plain strings still cover it.
        }
      }
    }
  }
  return out;
}

test.beforeAll(async () => {
  for (const file of messageFiles(srcDir)) {
    const module = await import(file);
    for (const [name, value] of Object.entries(module))
      collect(value, `${path.relative(srcDir, file)}.${name}`);
  }
  expect(catalog.length).toBeGreaterThan(200);
});

/** The server shows built-in sound names title-cased ("laundry-room" as "Laundry Room"). */
function builtinDisplayNames(): string[] {
  const manifest = path.join(builtinSoundsDir, "manifest.json");
  const entries = JSON.parse(fs.readFileSync(manifest, "utf8")) as {
    name: string;
  }[];
  return entries.map((entry) =>
    entry.name
      .split(/[-_\s]+/)
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(" "),
  );
}

const input = (locale: AppLocale): TextSweepInput => ({
  active: values(locale),
  other: values(locale === "en" ? "tr" : "en"),
  exclusions: [
    ...textSweepExclusions(),
    ...builtinDisplayNames(),
    "MediOSS",
    "English",
    "Türkçe",
  ],
});

/** A message template as a regular expression (`{name}` matches anything). */
const template = (message: AnyMessage, locale: AppLocale): RegExp => {
  const text = message[locale] as string;
  return new RegExp(
    `^${text
      .split(/\{[^}]+\}/)
      .map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
      .join(".+")}$`,
  );
};
const text = (message: AnyMessage, locale: AppLocale, params = {}): string =>
  Object.entries(params).reduce(
    (out, [name, value]) => out.replaceAll(`{${name}}`, String(value)),
    message[locale] as string,
  );

async function sweep(page: Page, locale: AppLocale, label: string) {
  await page.waitForLoadState("networkidle");
  const hits = await sweepText(page, input(locale));
  expect(hits, `${locale} ${label}`).toEqual([]);
}

/** Opens a dialog from `opener`, sweeps it, and closes it with Escape. */
async function sweepDialog(
  page: Page,
  locale: AppLocale,
  label: string,
  opener: Locator,
  role: "alertdialog" | "dialog" = "alertdialog",
) {
  await opener.first().scrollIntoViewIfNeeded();
  await opener.first().click();
  await expect(page.getByRole(role)).toBeVisible();
  await sweep(page, locale, `dialog ${label}`);
  await page.keyboard.press("Escape");
  await expect(page.getByRole(role)).toHaveCount(0);
}

const catalogOf = async <T>(file: string, name: string): Promise<T> =>
  (await import(path.join(srcDir, file)))[name] as T;

type MoodPage = { items: { recorded_at: string }[] };
type Messages = Record<string, AnyMessage>;

for (const locale of ["en", "tr"] as const) {
  test.describe(`member (${locale})`, () => {
    test.use({
      storageState: storageStatePath("member"),
      ...localeOptions(locale, NEW_YORK),
      viewport: viewport(1440),
    });

    test("every screen and dialog has no other-language or id-like text (SC-019, SC-022)", async ({
      page,
    }) => {
      test.setTimeout(5 * 60_000);
      const seed = readSeed();
      const screens = [
        ["home", "/"],
        ["library", "/library/"],
        ["program", `/program/?id=${seed.programs.withCover.id}`],
        ["program-empty", `/program/?id=${seed.programs.empty.id}`],
        ["player", `/player/?id=${seed.programs.withCover.meditations[0].id}`],
        ["soundscape", "/soundscape/"],
        ["mood", "/mood/"],
        ["history", "/history/"],
        ["stats", "/stats/"],
        ["settings", "/settings/"],
      ] as const;
      for (const [name, url] of screens) {
        await page.goto(url);
        await expect(page.locator("main")).toBeVisible();
        await sweep(page, locale, name);
      }

      const signOut = await catalogOf<Messages>(
        "lib/components/shell/shell.messages.ts",
        "signOutMessages",
      );
      const common = await catalogOf<Messages>(
        "lib/i18n/common.messages.ts",
        "actionMessages",
      );
      const soundscapeMessages = await catalogOf<Messages>(
        "lib/screens/soundscape/soundscape.messages.ts",
        "savedMixMessages",
      ).catch(() => null);

      // Sign out confirm (sidebar button), from a data-rich screen.
      await page.goto("/stats/");
      await sweepDialog(
        page,
        locale,
        "sign-out",
        page
          .getByRole("navigation")
          .getByRole("button", { name: text(common.signOut, locale) }),
      );

      // Mix rename and delete dialogs.
      await page.goto("/soundscape/");
      const mixName = seed.mixes.evening.name;
      const mix = soundscapeMessages ?? (await findMixMessages());
      await sweepDialog(
        page,
        locale,
        "rename-mix",
        page.getByRole("button", {
          name: text(mix.renameMix, locale, { name: mixName }),
        }),
        "dialog",
      );
      await sweepDialog(
        page,
        locale,
        "delete-mix",
        page.getByRole("button", {
          name: text(mix.deleteMix, locale, { name: mixName }),
        }),
      );

      // Mood entry delete confirm.
      const mood = await catalogOf<Messages>(
        "lib/screens/mood/mood.messages.ts",
        "moodMessages",
      );
      await page.goto("/mood/");
      await sweepDialog(
        page,
        locale,
        "delete-mood-entry",
        page.getByRole("button", { name: template(mood.deleteEntry, locale) }),
      );

      // Delete-account confirm.
      const account = await catalogOf<Messages>(
        "lib/screens/settings/settings.messages.ts",
        "deleteAccountMessages",
      );
      await page.goto("/settings/");
      const field = page.getByRole("textbox", {
        name: template(account.typeEmail, locale),
      });
      await field.fill(USERS.member.email);
      await sweepDialog(
        page,
        locale,
        "delete-account",
        page.getByRole("button", { name: text(account.action, locale) }),
      );

      // Mobile bottom bar More sheet.
      await page.setViewportSize(viewport(375));
      await page.goto("/");
      const shell = await catalogOf<Messages>(
        "lib/components/shell/shell.messages.ts",
        "shellMessages",
      );
      await sweepDialog(
        page,
        locale,
        "more-sheet",
        page
          .getByRole("navigation", { name: text(shell.bottomNav, locale) })
          .getByRole("button", { name: text(await moreLabel(), locale) }),
        "dialog",
      );
    });

    test("the flagged view (must change password) is fully translated (SC-019)", async ({
      page,
    }) => {
      await page.route("**/api/identity/me", (route) =>
        route.fulfill({
          json: {
            id: 99,
            email: USERS.flagged.email,
            role: "member",
            must_change_password: true,
            display_name: null,
            has_avatar: false,
          },
        }),
      );
      await page.goto("/");
      await expect(
        page.getByRole("form").or(page.locator("main")),
      ).toBeVisible();
      await expect(page.locator("input[type=password]").first()).toBeVisible();
      await sweep(page, locale, "flagged");
    });
  });

  test.describe(`admin (${locale})`, () => {
    test.use({
      storageState: storageStatePath("admin"),
      ...localeOptions(locale, NEW_YORK),
      viewport: viewport(1440),
    });

    test("admin sections and dialogs have no other-language or id-like text (SC-019)", async ({
      page,
    }) => {
      test.setTimeout(5 * 60_000);
      const seed = readSeed();
      for (const section of ["users", "meditations", "soundscape"]) {
        await page.goto(`/admin/?section=${section}`);
        await expect(page.locator("main")).toBeVisible();
        await sweep(page, locale, `admin-${section}`);
      }

      const users = await catalogOf<Messages>(
        "lib/screens/admin/admin.messages.ts",
        "userMessages",
      );
      await page.goto("/admin/?section=users");
      await sweepDialog(
        page,
        locale,
        "disable-user",
        page.getByRole("button", {
          name: text(users.disableNamed, locale, { email: USERS.member.email }),
        }),
      );
      await sweepDialog(
        page,
        locale,
        "delete-user",
        page.getByRole("button", {
          name: text(users.deleteNamed, locale, { email: USERS.member.email }),
        }),
      );

      const sounds = await catalogOf<Messages>(
        "lib/screens/admin/admin.messages.ts",
        "soundAdminMessages",
      );
      await page.goto("/admin/?section=soundscape");
      const rain = seed.sounds.rain.name;
      await sweepDialog(
        page,
        locale,
        "edit-sound",
        page.getByRole("button", {
          name: text(sounds.editNamed, locale, { name: rain }),
          exact: true,
        }),
        "dialog",
      );
      await sweepDialog(
        page,
        locale,
        "delete-sound",
        page.getByRole("button", {
          name: text(sounds.deleteNamed, locale, { name: rain }),
          exact: true,
        }),
      );
    });
  });
}

async function findMixMessages(): Promise<Messages> {
  const module = await import(
    path.join(srcDir, "lib/screens/soundscape/soundscape.messages.ts")
  );
  const found = Object.values(module).find(
    (value) => value && typeof value === "object" && "renameMix" in value,
  );
  return found as Messages;
}

async function moreLabel(): Promise<AnyMessage> {
  const module = await import(path.join(srcDir, "lib/i18n/common.messages.ts"));
  const found = Object.values(module).find(
    (value) =>
      value &&
      typeof value === "object" &&
      "more" in value &&
      typeof (value as Messages).more.en === "string",
  ) as Messages | undefined;
  return found!.more;
}

test.describe("Turkish formats match Intl (FR-102)", () => {
  test.use({
    storageState: storageStatePath("member"),
    ...localeOptions("tr", NEW_YORK),
    viewport: viewport(1440),
  });

  const dateTime = (iso: string) =>
    new Intl.DateTimeFormat("tr", {
      dateStyle: "medium",
      timeStyle: "short",
      timeZone: NEW_YORK,
    })
      .format(new Date(iso))
      .replace(/\s/g, " ");
  const calendarDay = (day: string) => {
    const [y, m, d] = day.split("-").map(Number);
    return new Intl.DateTimeFormat("tr", {
      dateStyle: "medium",
      timeZone: "UTC",
    }).format(new Date(Date.UTC(y, m - 1, d)));
  };
  const normalized = (page: Page) =>
    page
      .locator("main, aside")
      .evaluateAll((nodes) =>
        nodes.map((n) => (n.textContent ?? "").replace(/\s/g, " ")).join("\n"),
      );

  test("Home: mood times and the suggested duration", async ({ page }) => {
    const moodResponse = page.waitForResponse(
      (r) =>
        new URL(r.url()).pathname === "/api/mood" &&
        r.request().method() === "GET",
    );
    const suggestion = page.waitForResponse((r) =>
      r.url().includes("/api/meditation/daily-suggestion"),
    );
    await page.goto("/");
    const entries = ((await (await moodResponse).json()) as MoodPage).items;
    const meditation = (await (await suggestion).json()) as {
      duration_ms: number;
    } | null;
    await expect(page.locator("main li").first()).toBeVisible();
    await page.waitForLoadState("networkidle");
    const body = await normalized(page);
    expect(entries.length).toBeGreaterThan(0);
    expect(body).toContain(dateTime(entries[0].recorded_at));
    if (meditation) {
      const expected = await page.evaluate((ms) => {
        const total = Math.round(ms / 1000);
        const parts = {
          hours: Math.floor(total / 3600),
          minutes: Math.floor((total % 3600) / 60),
          seconds: total % 60,
        };
        const DF = (Intl as any).DurationFormat;
        if (DF && total > 0)
          return new DF("tr", { style: "long" }).format(parts);
        const units = [
          ["hour", parts.hours],
          ["minute", parts.minutes],
          ["second", parts.seconds],
        ] as const;
        const present = units.filter(([, v]) => v > 0);
        return new Intl.ListFormat("tr", {
          type: "unit",
          style: "long",
        }).format(
          (present.length ? present : [units[2]]).map(([unit, v]) =>
            new Intl.NumberFormat("tr", {
              style: "unit",
              unit,
              unitDisplay: "long",
            }).format(v),
          ),
        );
      }, meditation.duration_ms);
      expect(body).toContain(expected.replace(/\s/g, " "));
    }
  });

  test("Mood: entry times, mean with a decimal comma, percentages", async ({
    page,
  }) => {
    const list = page.waitForResponse(
      (r) =>
        new URL(r.url()).pathname === "/api/mood" &&
        r.request().method() === "GET",
    );
    const stats = page.waitForResponse(
      (r) => new URL(r.url()).pathname === "/api/mood/statistics",
    );
    await page.goto("/mood/");
    const entries = ((await (await list).json()) as MoodPage).items;
    const statistics = (await (await stats).json()) as {
      entry_count: number;
      average_score: number | null;
      distribution_percentages: number[];
    };
    await expect(page.locator("main li").first()).toBeVisible();
    await page.waitForLoadState("networkidle");
    const body = await normalized(page);
    expect(body).toContain(dateTime(entries[0].recorded_at));
    const mean = new Intl.NumberFormat("tr", {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    }).format(statistics.average_score ?? 0);
    expect(mean).toMatch(/^\d,\d$/);
    expect(body).toContain(mean);
    const percent = new Intl.NumberFormat("tr", {
      style: "percent",
      maximumFractionDigits: 0,
    });
    for (const share of statistics.distribution_percentages)
      expect(body).toContain(percent.format(share / 100).replace(/\s/g, " "));
  });

  test("History: event times", async ({ page }) => {
    const timeline = page.waitForResponse(
      (r) => r.url().includes("/api/timeline") && r.ok(),
    );
    await page.goto("/history/");
    const events = ((await (await timeline).json()) as MoodPage).items;
    await expect(page.locator("main li").first()).toBeVisible();
    await page.waitForLoadState("networkidle");
    const body = await normalized(page);
    expect(events.length).toBeGreaterThan(0);
    expect(body).toContain(dateTime(events[0].recorded_at));
  });

  test("Stats: practiced days and minutes", async ({ page }) => {
    const stats = page.waitForResponse(
      (r) => r.url().includes("/api/meditation/statistics?") && r.ok(),
    );
    await page.goto("/stats/");
    const served = (await (await stats).json()) as {
      total_minutes: number;
      total_days: number;
      calendar: { date: string; minutes: number }[];
    };
    await expect(page.locator("main dl dd").first()).toBeVisible();
    const body = await normalized(page);
    const number = new Intl.NumberFormat("tr");
    expect(body).toContain(`${number.format(served.total_minutes)} dakika`);
    expect(body).toContain(`${number.format(served.total_days)} gün`);
    expect(served.calendar.length).toBeGreaterThan(0);
    for (const day of served.calendar) {
      expect(body).toContain(calendarDay(day.date));
      expect(body).toContain(`${number.format(day.minutes)} dakika`);
    }
  });
});

test.describe("first visit follows the browser language (FR-114)", () => {
  const cases = [
    ["de-DE", "en", "Email"],
    ["tr-TR", "tr", "E-posta"],
    ["en-US", "en", "Email"],
  ] as const;
  for (const [browserLocale, expected, label] of cases) {
    test(`${browserLocale} shows ${expected}`, async ({ browser, baseURL }) => {
      const context = await browser.newContext({
        baseURL,
        storageState: { cookies: [], origins: [] },
        locale: browserLocale,
        extraHTTPHeaders: {
          "Accept-Language": `${browserLocale},${browserLocale.split("-")[0]};q=0.9`,
        },
      });
      const page = await context.newPage();
      const guard = guardNetwork(page);
      try {
        await page.goto("/login/");
        await expect(page.locator("html")).toHaveAttribute("lang", expected);
        await expect(page.getByLabel(label, { exact: true })).toBeVisible();
        expect(
          await page.evaluate(() => localStorage.getItem("medioss.locale")),
        ).toBeNull();
        guard.assertSameOrigin();
      } finally {
        await context.close();
      }
    });
  }
});
