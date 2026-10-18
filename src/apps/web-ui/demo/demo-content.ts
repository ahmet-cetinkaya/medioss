// Deterministic demo content for a local MediOSS instance. Test-only users
// and passwords come from e2e/support/auth.ts (example.test addresses).
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { qaDir } from "../e2e/support/server-env";

// Demo covers: calm, meditation-suitable Unsplash photos (one per program,
// in DEMO_PROGRAMS order). Stable photo ids with size params keep the CDN
// URL reproducible; images are cached under .agents/qa/demo/. Used under
// the Unsplash License (free use, attribution appreciated but not
// required); photographer credits are in docs/demo.md.
const COVER_PHOTO_IDS = [
  "photo-1499209974431-9dddcece7f88", // person with open arms at sunrise
  "photo-1502082553048-f009c37129b9", // lone tree under a blue sky
  "photo-1500462918059-b1a0cb512f1d", // pink and violet gradient doorway
  "photo-1518495973542-4542c06a5843", // sunlight through a tree canopy
  "photo-1472214103451-9374bd1c798e", // green hills with a stone circle at sunset
  "photo-1469474968028-56623f02e42e", // misty rocks above a green valley
  "photo-1470252649378-9c29740c9fa8", // sunset over a meadow
  "photo-1439066615861-d1af74d74000", // still lake with a jetty
  "photo-1476673160081-cf065607f449", // soft waves on a quiet beach
  "photo-1509316785289-025f5b846b35", // desert mesas under a blue sky
  "photo-1470071459604-3b5ec3a7fe05", // fog rolling over green cliffs
  "photo-1505144808419-1957a94ca61e", // turquoise wave seen from above
  "photo-1508739773434-c26b3d09e071", // red mountain peak in morning light
  "photo-1519681393784-d120267933ba", // milky way over snowy peaks
  "photo-1518709268805-4e9042af9f23", // misty castle gate
  "photo-1500530855697-b586d89ba3ee", // desert road between red rocks
] as const;

const COVER_URL = (photoId: string) =>
  `https://images.unsplash.com/${photoId}?w=800&h=450&fit=crop&q=80&fm=jpg`;

export type DemoProgram = {
  title: string;
  description: string;
  meditations: readonly string[];
};
export type DemoCategory = {
  name: string;
  programs: readonly DemoProgram[];
};

const program = (
  title: string,
  description: string,
  meditations: readonly string[],
): DemoProgram => ({ title, description, meditations });

export const DEMO_CATEGORIES: readonly DemoCategory[] = [
  {
    name: "Mindfulness",
    programs: [
      program("Morning Calm", "Three short practices to start the day.", [
        "Body Scan",
        "Breath Focus",
        "Quiet Mind",
      ]),
      program("Present Moment", "Notice what is here, right now.", [
        "Five Senses",
        "Open Awareness",
      ]),
      program("Gentle Attention", "Train a kind, steady focus.", [
        "Anchor Breath",
        "Soft Gaze",
      ]),
      program("Mindful Walking", "Meditation in motion.", [
        "Slow Steps",
        "Walking Stillness",
      ]),
      program("Everyday Awareness", "Bring attention into daily tasks.", [
        "Mindful Tea",
        "Mindful Listening",
      ]),
    ],
  },
  {
    name: "Sleep",
    programs: [
      program("Evening Rest", "Wind down before sleep.", [
        "Slow Breathing",
        "Gratitude Pause",
      ]),
      program("Drifting Off", "Let the day dissolve.", [
        "Body Release",
        "Counting Down",
      ]),
      program("Night Sky", "A calm imagery journey.", [
        "Stargazing",
        "Moonlit Lake",
      ]),
      program("Deep Rest", "A longer practice for restless nights.", [
        "Progressive Relaxation",
        "Heavy Limbs",
      ]),
    ],
  },
  {
    name: "Focus",
    programs: [
      program("Focus Reset", "Clear your head between tasks.", [
        "Five Minute Anchor",
        "Reset Breath",
      ]),
      program("Deep Work", "Settle in before a long session.", [
        "Single Task",
        "Clear Desk",
      ]),
      program("Study Break", "Recharge without losing the thread.", [
        "Shoulder Release",
        "Eyes Rest",
      ]),
    ],
  },
  {
    name: "Stress Relief",
    programs: [
      program("Calm in Chaos", "Find steadiness when things are loud.", [
        "Grounding",
        "Box Breathing",
      ]),
      program("Release Tension", "Let go of what the body is holding.", [
        "Jaw and Face",
        "Shoulders Down",
      ]),
      program("Kind Mind", "Soften self-criticism.", [
        "Loving Kindness",
        "Self Compassion",
      ]),
      program("Anxiety Ease", "Slow the spiral.", ["Name It", "Long Exhale"]),
    ],
  },
];

export const DEMO_PROGRAMS: readonly DemoProgram[] = DEMO_CATEGORIES.flatMap(
  (category) => category.programs,
);

export const DEMO_SOUND_CATEGORY = "Demo Sounds";
export const DEMO_UPLOADED_SOUNDS = ["Soft Pink Noise", "Night Hum"] as const;
export const DEMO_MIX_NAME = "Rainy Evening";
export const MOOD_DAYS = 45;
export const MEDITATION_DAYS = [1, 2, 4, 5, 8, 9, 10, 14, 15, 21];

const MS_PER_DAY = 86_400_000;
const MOOD_PATTERN = [3, 4, 4, 5, 3, 2, 4, 5, 5, 4, 3, 4];

export const demoDir = path.join(qaDir, "demo");

/** Mood score for a day offset; a fixed pattern, so every run matches. */
export const moodScore = (daysAgo: number): number =>
  MOOD_PATTERN[daysAgo % MOOD_PATTERN.length];

/** CSV the portability import accepts, one noon-UTC entry per day. */
export function moodCsv(now: Date = new Date()): string {
  const rows = ["recorded_at,score,note"];
  for (let daysAgo = MOOD_DAYS; daysAgo >= 1; daysAgo -= 1) {
    const day = new Date(now.getTime() - daysAgo * MS_PER_DAY);
    day.setUTCHours(12, 0, 0, 0);
    const note = daysAgo % 6 === 0 ? "Felt steady after a walk" : "";
    rows.push(`${day.toISOString()},${moodScore(daysAgo)},${note}`);
  }
  return rows.join("\n") + "\n";
}

function ffmpeg(args: string[]): void {
  execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y", ...args]);
}

/** Downloads one Unsplash cover into the demo cache; existing files are reused. */
function cachedCover(index: number): string {
  if (index >= COVER_PHOTO_IDS.length) {
    throw new Error(
      `no cover photo for demo program ${index}: add it to COVER_PHOTO_IDS`,
    );
  }
  const file = path.join(demoDir, `cover-${index}.jpg`);
  if (!fs.existsSync(file)) {
    // Download under a temp name so a failed run never leaves a truncated
    // file that the next run would reuse.
    const partial = `${file}.part`;
    execFileSync("curl", [
      "-fsSL",
      "--retry",
      "3",
      "-o",
      partial,
      COVER_URL(COVER_PHOTO_IDS[index]),
    ]);
    fs.renameSync(partial, file);
  }
  return file;
}

export type DemoFiles = {
  tone: string;
  loop: string;
  covers: string[];
};

/** Generates the audio and cover files once; existing files are reused. */
export function ensureDemoFiles(): DemoFiles {
  fs.mkdirSync(demoDir, { recursive: true });
  const tone = path.join(demoDir, "tone.mp3");
  const loop = path.join(demoDir, "loop.ogg");
  if (!fs.existsSync(tone)) {
    ffmpeg([
      "-f",
      "lavfi",
      "-i",
      "sine=frequency=396:duration=20",
      "-b:a",
      "64k",
      tone,
    ]);
  }
  if (!fs.existsSync(loop)) {
    ffmpeg([
      "-f",
      "lavfi",
      "-i",
      "anoisesrc=d=30:c=pink:a=0.2:seed=7",
      "-c:a",
      "libvorbis",
      loop,
    ]);
  }
  const covers = DEMO_PROGRAMS.map((_, index) => cachedCover(index));
  return { tone, loop, covers };
}
