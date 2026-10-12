// `seed` project: fills a fresh instance through the API and records ids in
// .agents/qa/seed-<port>.json for the `seeded` suites. Refuses an instance
// that already has content.
import { expect, test } from "../support/test-fixture";
import {
  expectOk,
  getJson,
  postJson,
  postMultipart,
  type Api,
} from "../support/api-client";
import {
  ensureUsers,
  loginAs,
  saveStorageState,
  USERS,
  type UserDto,
} from "../support/auth";
import { ensureFixtures, readFixture } from "../support/fixtures-media";
import { localDate, tzOffsetMinutes, zonedTime } from "../support/locale";
import {
  moodNote,
  SEED,
  writeSeed,
  type Named,
  type SeedRecord,
} from "../support/seed-content";
import { baseURL, e2eDataDir } from "../support/server-env";

type Program = {
  id: number;
  title: string;
  has_cover: boolean;
  category_id: number | null;
};
type Meditation = { id: number; name: string };
type Sound = {
  id: number;
  name: string;
  category_id: number;
  builtin: boolean;
};
type Mix = {
  id: number;
  name: string;
  items: { sound_id: number; volume: number; position: number }[];
};
type MoodEntry = { id: number; score: number; note: string | null };
type Note = { id: number; text: string };
type Statistics = { calendar: { date: string; minutes: number }[] };

test.setTimeout(5 * 60_000);

async function assertNoContent(admin: Api, member: Api): Promise<void> {
  const programs = await getJson<Program[]>(admin, "/api/meditation/programs");
  const categories = await getJson<Named[]>(
    admin,
    "/api/meditation/categories",
  );
  const sounds = await getJson<Sound[]>(admin, "/api/soundscape/sounds");
  const soundCategories = await getJson<Named[]>(
    admin,
    "/api/soundscape/categories",
  );
  const users = await getJson<UserDto[]>(admin, "/api/identity/users");
  const mixes = await getJson<Mix[]>(member, "/api/soundscape/mixes");
  const mood = await getJson<{ items: MoodEntry[] }>(
    member,
    "/api/mood?limit=1",
  );
  const seededSoundCategories = new Set<string>(
    Object.values(SEED.soundCategories),
  );
  const found = {
    programs: programs.length,
    meditationCategories: categories.length,
    uploadedSounds: sounds.filter((sound) => !sound.builtin).length,
    seedSoundCategories: soundCategories.filter((category) =>
      seededSoundCategories.has(category.name),
    ).length,
    extraUsers: users.length - 2,
    mixes: mixes.length,
    moodEntries: mood.items.length,
  };
  if (Object.values(found).some((count) => count > 0)) {
    throw new Error(
      `seed needs a fresh instance (admin and member only); found ${JSON.stringify(found)}`,
    );
  }
}

/** `YYYY-MM-DD` shifted by `days` (calendar arithmetic, no time zone). */
function shiftDate(date: string, days: number): string {
  const day = new Date(`${date}T00:00:00Z`);
  day.setUTCDate(day.getUTCDate() + days);
  return day.toISOString().slice(0, 10);
}

test("seed a fresh instance through the API", async () => {
  const fixtures = ensureFixtures();
  const { admin: adminUser, member: memberUser } = await ensureUsers();
  const admin = await loginAs("admin");
  const member = await loginAs("member");

  await test.step("refuse a non-empty instance", () =>
    assertNoContent(admin, member));

  const flagged =
    await test.step("flagged member (created, never signed in)", () =>
      postJson<UserDto>(admin, "/api/identity/users", {
        email: USERS.flagged.email,
        password: USERS.flagged.temporaryPassword,
      }));
  expect(flagged.must_change_password).toBe(true);

  const categories =
    await test.step("meditation categories A, B, and empty C", async () => {
      const create = (name: string) =>
        postJson<Named>(admin, "/api/meditation/categories", { name });
      const c = SEED.meditationCategories;
      return {
        a: await create(c.a),
        b: await create(c.b),
        c: await create(c.c),
      };
    });

  const programs =
    await test.step("programs with cover, without cover, and empty", async () => {
      const p = SEED.programs;
      const create = (title: string, description: string, cover?: Buffer) =>
        postMultipart<Program>(admin, "/api/meditation/programs", {
          title,
          ...(description ? { description } : {}),
          ...(cover
            ? {
                cover: {
                  name: "cover.png",
                  mimeType: "image/png",
                  buffer: cover,
                },
              }
            : {}),
        });
      const assign = async (program: Program, category: Named | null) => {
        if (!category) return program;
        await expectOk(
          await admin.put(`/api/meditation/programs/${program.id}/category`, {
            data: { category_id: category.id },
          }),
          "assign program category",
        );
        return { ...program, category_id: category.id };
      };
      return {
        withCover: await assign(
          await create(
            p.withCover.title,
            p.withCover.description,
            readFixture(fixtures.coverPng),
          ),
          categories.a,
        ),
        noCover: await assign(
          await create(p.noCover.title, p.noCover.description),
          categories.b,
        ),
        empty: await assign(
          await create(p.empty.title, p.empty.description),
          null,
        ),
      };
    });

  const meditations =
    await test.step("upload meditations (20 s mp3)", async () => {
      const tone = readFixture(fixtures.toneMp3);
      const upload = (programId: number, name: string) =>
        postMultipart<Meditation>(admin, "/api/meditation/audio", {
          name,
          program_id: String(programId),
          file: { name: "tone.mp3", mimeType: "audio/mpeg", buffer: tone },
        });
      const result: Record<"withCover" | "noCover" | "empty", Named[]> = {
        withCover: [],
        noCover: [],
        empty: [],
      };
      for (const key of ["withCover", "noCover", "empty"] as const) {
        for (const name of SEED.programs[key].meditations) {
          const meditation = await upload(programs[key].id, name);
          result[key].push({ id: meditation.id, name: meditation.name });
        }
      }
      const members = await getJson<Meditation[]>(
        admin,
        `/api/meditation/programs/${programs.withCover.id}/members`,
      );
      expect(members.map((m) => m.name)).toEqual([
        ...SEED.programs.withCover.meditations,
      ]);
      return result;
    });
  const bodyScan = meditations.withCover[0];
  const breathFocus = meditations.withCover[1];

  const soundCategories = await test.step("sound categories", async () => {
    const create = (name: string) =>
      postJson<Named>(admin, "/api/soundscape/categories", { name });
    return {
      weather: await create(SEED.soundCategories.weather),
      studio: await create(SEED.soundCategories.studio),
    };
  });

  const sounds = await test.step("upload sounds (30 s ogg loop)", async () => {
    const loop = readFixture(fixtures.loopOgg);
    const upload = (name: string) =>
      postMultipart<Sound>(admin, "/api/soundscape/sounds", {
        name,
        category_id: String(soundCategories.weather.id),
        file: { name: "loop.ogg", mimeType: "audio/ogg", buffer: loop },
      });
    return {
      rain: await upload(SEED.sounds.rain),
      wind: await upload(SEED.sounds.wind),
      temporary: await upload(SEED.sounds.temporary),
    };
  });

  const mixes =
    await test.step("mixes Evening (Rain 30, Wind 70) and Broken Mix", async () => {
      const evening = await postJson<Mix>(member, "/api/soundscape/mixes", {
        name: SEED.mixes.evening,
        items: [
          { sound_id: sounds.rain.id, volume: 30, position: 0 },
          { sound_id: sounds.wind.id, volume: 70, position: 1 },
        ],
      });
      const broken = await postJson<Mix>(member, "/api/soundscape/mixes", {
        name: SEED.mixes.broken,
        items: [
          { sound_id: sounds.rain.id, volume: 50, position: 0 },
          { sound_id: sounds.temporary.id, volume: 50, position: 1 },
        ],
      });
      await expectOk(
        await admin.delete(`/api/soundscape/sounds/${sounds.temporary.id}`),
        "delete temporary sound",
      );
      return {
        evening,
        broken: await getJson<Mix>(
          member,
          `/api/soundscape/mixes/${broken.id}`,
        ),
      };
    });

  const mood =
    await test.step(`${SEED.moodEntryCount} mood entries`, async () => {
      const ids: number[] = [];
      let withNotes = 0;
      for (let index = 0; index < SEED.moodEntryCount; index += 1) {
        const note =
          index % SEED.moodNoteEvery === 0 ? moodNote(index) : undefined;
        if (note) withNotes += 1;
        ids.push(
          (
            await postJson<MoodEntry>(member, "/api/mood", {
              score: (index % 5) + 1,
              note,
            })
          ).id,
        );
      }
      return { count: ids.length, withNotes, ids };
    });

  const notes = await test.step("meditation notes", async () => {
    const created: SeedRecord["meditationNotes"] = [];
    for (const text of SEED.meditationNotes) {
      const note = await postJson<Note>(
        member,
        `/api/meditation/${bodyScan.id}/notes`,
        { text },
      );
      created.push({ id: note.id, meditationId: bodyScan.id, text: note.text });
    }
    return created;
  });

  const completion =
    await test.step("completion at 23:30 America/New_York", async () => {
      const c = SEED.completion;
      const date = shiftDate(localDate(c.timeZone), -c.daysAgo);
      const startedAt = zonedTime(c.timeZone, date, c.localTime);
      const finishedAt = new Date(startedAt.getTime() + c.minutes * 60_000);
      const offset = tzOffsetMinutes(c.timeZone, startedAt);
      await postJson(member, `/api/meditation/${bodyScan.id}/completion`, {
        started_at: startedAt.toISOString(),
        finished_at: finishedAt.toISOString(),
      });
      const record = {
        meditationId: bodyScan.id,
        timeZone: c.timeZone,
        tzOffsetMinutes: offset,
        startedAt: startedAt.toISOString(),
        finishedAt: finishedAt.toISOString(),
        localDate: date,
        utcDate: startedAt.toISOString().slice(0, 10),
        minutes: c.minutes,
      };
      expect(record.utcDate).toBe(shiftDate(date, 1));
      const local = await getJson<Statistics>(
        member,
        `/api/meditation/statistics?range=all&tz_offset_minutes=${offset}`,
      );
      expect(local.calendar).toContainEqual({
        date: record.localDate,
        minutes: c.minutes,
      });
      const utc = await getJson<Statistics>(
        member,
        "/api/meditation/statistics?range=all&tz_offset_minutes=0",
      );
      expect(utc.calendar).toContainEqual({
        date: record.utcDate,
        minutes: c.minutes,
      });
      return record;
    });

  await test.step("playback position for Continue", async () => {
    await expectOk(
      await member.post(`/api/meditation/${breathFocus.id}/playback-state`, {
        data: { position_ms: SEED.playbackPositionMs },
      }),
      "record playback position",
    );
  });

  const toProgram = (key: "withCover" | "noCover" | "empty") => ({
    id: programs[key].id,
    title: programs[key].title,
    categoryId: programs[key].category_id,
    hasCover: programs[key].has_cover,
    meditations: meditations[key],
  });
  const named = (value: Named): Named => ({ id: value.id, name: value.name });
  const file = writeSeed({
    runId: process.env.E2E_RUN_ID,
    baseURL: baseURL(),
    dataDir: e2eDataDir(),
    createdAt: new Date().toISOString(),
    users: {
      admin: {
        id: adminUser.id,
        email: adminUser.email,
        displayName: adminUser.display_name,
      },
      member: {
        id: memberUser.id,
        email: memberUser.email,
        displayName: memberUser.display_name,
      },
      flagged: {
        id: flagged.id,
        email: flagged.email,
        displayName: flagged.display_name,
      },
    },
    meditationCategories: {
      a: named(categories.a),
      b: named(categories.b),
      c: named(categories.c),
    },
    programs: {
      withCover: toProgram("withCover"),
      noCover: toProgram("noCover"),
      empty: toProgram("empty"),
    },
    soundCategories: {
      weather: named(soundCategories.weather),
      studio: named(soundCategories.studio),
    },
    sounds: {
      rain: { ...named(sounds.rain), categoryId: sounds.rain.category_id },
      wind: { ...named(sounds.wind), categoryId: sounds.wind.category_id },
    },
    deletedSound: named(sounds.temporary),
    mixes: {
      evening: { ...named(mixes.evening), items: mixes.evening.items },
      broken: { ...named(mixes.broken), items: mixes.broken.items },
    },
    mood,
    meditationNotes: notes,
    playback: {
      meditationId: breathFocus.id,
      positionMs: SEED.playbackPositionMs,
    },
    completion,
  });
  test.info().annotations.push({ type: "seed", description: file });

  await admin.dispose();
  await member.dispose();
  // Fresh sessions for the seeded suites.
  await saveStorageState("admin");
  await saveStorageState("member");
});
