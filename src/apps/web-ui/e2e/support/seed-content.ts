// Deterministic seed names, the seed record shape, and the SC-019 exclusions.
import fs from "node:fs";
import path from "node:path";
import { USERS } from "./auth";
import { builtinSoundsDir, e2eDataDir, seedFile } from "./server-env";

export const SEED = {
  meditationCategories: { a: "Category A", b: "Category B", c: "Category C" },
  programs: {
    withCover: {
      title: "Morning Calm",
      description: "Three short practices to start the day.",
      meditations: ["Body Scan", "Breath Focus", "Quiet Mind"],
    },
    noCover: {
      title: "Evening Rest",
      description: "Wind down before sleep.",
      meditations: ["Slow Breathing"],
    },
    empty: {
      title: "Empty Program",
      description: "",
      meditations: [] as string[],
    },
  },
  soundCategories: { weather: "Weather", studio: "Studio" },
  sounds: { rain: "Rain", wind: "Wind", temporary: "Thunder Clap" },
  mixes: { evening: "Evening", broken: "Broken Mix" },
  moodEntryCount: 60,
  /** Every fourth mood entry carries a note. */
  moodNoteEvery: 4,
  meditationNotes: [
    "Felt calm after the body scan.",
    "Shoulders relaxed by the end.",
  ],
  completion: {
    timeZone: "America/New_York",
    localTime: "23:30",
    minutes: 20,
    daysAgo: 2,
  },
  playbackPositionMs: 8_000,
} as const;

export const moodNote = (index: number): string => `Mood note ${index + 1}`;

export type Named = { id: number; name: string };

export type SeedRecord = {
  runId: string | undefined;
  baseURL: string;
  dataDir: string;
  createdAt: string;
  users: Record<
    "admin" | "member" | "flagged",
    { id: number; email: string; displayName: string | null }
  >;
  meditationCategories: Record<"a" | "b" | "c", Named>;
  programs: Record<
    "withCover" | "noCover" | "empty",
    {
      id: number;
      title: string;
      categoryId: number | null;
      hasCover: boolean;
      meditations: Named[];
    }
  >;
  soundCategories: Record<"weather" | "studio", Named>;
  sounds: Record<"rain" | "wind", Named & { categoryId: number }>;
  deletedSound: Named;
  mixes: Record<
    "evening" | "broken",
    Named & { items: { sound_id: number; volume: number; position: number }[] }
  >;
  mood: { count: number; withNotes: number; ids: number[] };
  meditationNotes: { id: number; meditationId: number; text: string }[];
  playback: { meditationId: number; positionMs: number };
  completion: {
    meditationId: number;
    timeZone: string;
    /** East-positive minutes, as `tz_offset_minutes` expects (New York is -240 or -300). */
    tzOffsetMinutes: number;
    startedAt: string;
    finishedAt: string;
    /** The New York calendar day the session belongs to. */
    localDate: string;
    /** The UTC calendar day of the same instant (the next day). */
    utcDate: string;
    minutes: number;
  };
};

export function writeSeed(record: SeedRecord): string {
  fs.writeFileSync(seedFile(), `${JSON.stringify(record, null, 2)}\n`);
  return seedFile();
}

/** Reads the seed of the current run; refuses a record from another data dir. */
export function readSeed(): SeedRecord {
  const file = seedFile();
  if (!fs.existsSync(file))
    throw new Error(`no seed record at ${file}; run the "seed" project first`);
  const record = JSON.parse(fs.readFileSync(file, "utf8")) as SeedRecord;
  if (record.dataDir !== e2eDataDir()) {
    throw new Error(
      `${file} belongs to ${record.dataDir}, not this run's ${e2eDataDir()}; rerun the "seed" project`,
    );
  }
  return record;
}

type ManifestEntry = { category_name: string; name: string };

/** Built-in sound and category names (shown as shipped, never translated). */
export function builtinSoundNames(): string[] {
  const manifest = path.join(builtinSoundsDir, "manifest.json");
  if (!fs.existsSync(manifest)) return [];
  const entries = JSON.parse(
    fs.readFileSync(manifest, "utf8"),
  ) as ManifestEntry[];
  return [
    ...new Set(entries.flatMap((entry) => [entry.name, entry.category_name])),
  ];
}

/** SC-019 exclusions: user- or host-entered content and built-in sound names. */
export function textSweepExclusions(): string[] {
  const p = SEED.programs;
  return [
    ...Object.values(SEED.meditationCategories),
    ...[p.withCover, p.noCover, p.empty].flatMap((program) => [
      program.title,
      program.description,
      ...program.meditations,
    ]),
    ...Object.values(SEED.soundCategories),
    ...Object.values(SEED.sounds),
    ...Object.values(SEED.mixes),
    ...SEED.meditationNotes,
    ...Array.from({ length: SEED.moodEntryCount }, (_, index) =>
      moodNote(index),
    ),
    USERS.admin.email,
    USERS.admin.displayName,
    USERS.member.email,
    USERS.flagged.email,
    ...builtinSoundNames(),
  ].filter(Boolean);
}
