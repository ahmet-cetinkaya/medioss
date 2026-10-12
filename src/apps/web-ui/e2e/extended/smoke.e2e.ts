// `seeded` smoke: the server is healthy and the seed is reachable as the member.
import { getJson, healthStatus, newApi } from "../support/api-client";
import { storageStatePath, USERS } from "../support/auth";
import { emulateReducedTransparency } from "../support/reduced-transparency";
import { readSeed } from "../support/seed-content";
import { expect, test } from "../support/test-fixture";

type Program = {
  id: number;
  title: string;
  has_cover: boolean;
  category_id: number | null;
};
type Mix = {
  id: number;
  name: string;
  items: { sound_id: number; volume: number }[];
};

test("health endpoint is ok", async () => {
  const api = await newApi();
  expect(await healthStatus(api)).toEqual({ status: "ok" });
  await api.dispose();
});

test("seeded data is reachable through the API as the member", async () => {
  const seed = readSeed();
  const member = await newApi(storageStatePath("member"));

  const me = await getJson<{ email: string; must_change_password: boolean }>(
    member,
    "/api/identity/me",
  );
  expect(me).toMatchObject({
    email: USERS.member.email,
    must_change_password: false,
  });

  const programs = await getJson<Program[]>(member, "/api/meditation/programs");
  expect(programs.map((program) => program.title).sort()).toEqual(
    [
      seed.programs.withCover.title,
      seed.programs.noCover.title,
      seed.programs.empty.title,
    ].sort(),
  );
  expect(
    programs.find((program) => program.id === seed.programs.withCover.id)
      ?.has_cover,
  ).toBe(true);

  const members = await getJson<{ name: string }[]>(
    member,
    `/api/meditation/programs/${seed.programs.withCover.id}/members`,
  );
  expect(members.map((meditation) => meditation.name)).toContain("Body Scan");

  const mixes = await getJson<Mix[]>(member, "/api/soundscape/mixes");
  const evening = mixes.find((mix) => mix.id === seed.mixes.evening.id);
  expect(evening?.items.map((item) => item.volume)).toEqual([30, 70]);
  const broken = mixes.find((mix) => mix.id === seed.mixes.broken.id);
  expect(
    broken?.items.some((item) => item.sound_id === seed.deletedSound.id),
  ).toBe(false);

  const firstPage = await getJson<{
    items: unknown[];
    next_cursor: string | null;
  }>(member, "/api/mood?limit=50");
  expect(firstPage.items).toHaveLength(50);
  expect(firstPage.next_cursor).not.toBeNull();

  const statistics = await getJson<{
    calendar: { date: string; minutes: number }[];
  }>(
    member,
    `/api/meditation/statistics?range=all&tz_offset_minutes=${seed.completion.tzOffsetMinutes}`,
  );
  expect(statistics.calendar).toContainEqual({
    date: seed.completion.localDate,
    minutes: seed.completion.minutes,
  });
  await member.dispose();
});

test.describe("browser", () => {
  test.use({ storageState: storageStatePath("member") });

  test("home loads same-origin only, no native dialog; transparency emulation applies", async ({
    page,
    networkGuard,
  }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    expect(networkGuard.requests.length).toBeGreaterThan(0);
    expect(networkGuard.foreign()).toEqual([]);

    await emulateReducedTransparency(page, { reducedMotion: "reduce" });
    const media = await page.evaluate(() => ({
      transparency: matchMedia("(prefers-reduced-transparency: reduce)")
        .matches,
      motion: matchMedia("(prefers-reduced-motion: reduce)").matches,
    }));
    expect(media).toEqual({ transparency: true, motion: true });
  });
});
