// Seeds a running, EMPTY MediOSS instance with deterministic demo content.
// Usage (via `make demo-seed`): E2E_PORT=<port> SESSION_SECRET=<test secret> bun demo/seed-demo.ts
import fs from "node:fs";
import {
  expectOk,
  getJson,
  postJson,
  postMultipart,
} from "../e2e/support/api-client";
import { ensureUsers, loginAs, saveStorageState } from "../e2e/support/auth";
import { baseURL } from "../e2e/support/server-env";
import {
  DEMO_MIX_NAME,
  DEMO_CATEGORIES,
  DEMO_SOUND_CATEGORY,
  DEMO_UPLOADED_SOUNDS,
  MEDITATION_DAYS,
  ensureDemoFiles,
  moodCsv,
} from "./demo-content";

type Id = { id: number; name?: string };
type Sound = { id: number; name: string; builtin: boolean };

const COMPLETION_MINUTES = 12;
const MS_PER_DAY = 86_400_000;
const MIX_BUILTIN_SOUND_COUNT = 3;
const MIX_VOLUMES = [60, 35, 20];

async function main(): Promise<void> {
  const files = ensureDemoFiles();
  await ensureUsers();
  const admin = await loginAs("admin");
  const member = await loginAs("member");

  const existing = await getJson<unknown[]>(admin, "/api/meditation/programs");
  if (existing.length > 0) {
    throw new Error(
      "demo seed needs an empty instance; programs already exist",
    );
  }

  const meditationIds: number[] = [];
  let coverIndex = 0;
  for (const demoCategory of DEMO_CATEGORIES) {
    const category = await postJson<Id>(admin, "/api/meditation/categories", {
      name: demoCategory.name,
    });
    for (const program of demoCategory.programs) {
      const created = await postMultipart<Id>(
        admin,
        "/api/meditation/programs",
        {
          title: program.title,
          description: program.description,
          cover: {
            name: "cover.png",
            mimeType: "image/png",
            buffer: fs.readFileSync(files.covers[coverIndex]),
          },
        },
      );
      coverIndex += 1;
      await expectOk(
        await admin.put(`/api/meditation/programs/${created.id}/category`, {
          data: { category_id: category.id },
        }),
        "assign program category",
      );
      for (const name of program.meditations) {
        const meditation = await postMultipart<Id>(
          admin,
          "/api/meditation/audio",
          {
            name,
            program_id: String(created.id),
            file: {
              name: "tone.mp3",
              mimeType: "audio/mpeg",
              buffer: fs.readFileSync(files.tone),
            },
          },
        );
        meditationIds.push(meditation.id);
      }
    }
  }

  const soundCategory = await postJson<Id>(
    admin,
    "/api/soundscape/categories",
    {
      name: DEMO_SOUND_CATEGORY,
    },
  );
  for (const name of DEMO_UPLOADED_SOUNDS) {
    await postMultipart<Sound>(admin, "/api/soundscape/sounds", {
      name,
      category_id: String(soundCategory.id),
      file: {
        name: "loop.ogg",
        mimeType: "audio/ogg",
        buffer: fs.readFileSync(files.loop),
      },
    });
  }
  const sounds = await getJson<Sound[]>(member, "/api/soundscape/sounds");
  const builtin = sounds
    .filter((sound) => sound.builtin)
    .slice(0, MIX_BUILTIN_SOUND_COUNT);
  await postJson(member, "/api/soundscape/mixes", {
    name: DEMO_MIX_NAME,
    items: builtin.map((sound, position) => ({
      sound_id: sound.id,
      volume: MIX_VOLUMES[position],
      position,
    })),
  });

  await expectOk(
    await member.post("/api/portability/import", {
      multipart: {
        file: {
          name: "mood.csv",
          mimeType: "text/csv",
          buffer: Buffer.from(moodCsv()),
        },
      },
    }),
    "import demo mood entries",
  );

  const now = Date.now();
  for (const [index, daysAgo] of MEDITATION_DAYS.entries()) {
    const startedAt = new Date(now - daysAgo * MS_PER_DAY);
    const finishedAt = new Date(
      startedAt.getTime() + COMPLETION_MINUTES * 60_000,
    );
    await postJson(
      member,
      `/api/meditation/${meditationIds[index % meditationIds.length]}/completion`,
      {
        started_at: startedAt.toISOString(),
        finished_at: finishedAt.toISOString(),
      },
    );
  }

  await Promise.all([admin.dispose(), member.dispose()]);
  await saveStorageState("admin");
  await saveStorageState("member");
  console.log(`demo seeded at ${baseURL()}`);
}

main().catch((error: Error) => {
  console.error(error.message);
  process.exit(1);
});
