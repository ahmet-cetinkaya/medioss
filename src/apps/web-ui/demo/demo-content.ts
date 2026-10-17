// Deterministic demo content for a local MediOSS instance. Test-only users
// and passwords come from e2e/support/auth.ts (example.test addresses).
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { qaDir } from "../e2e/support/server-env";

export const DEMO_PROGRAMS = [
  {
    title: "Morning Calm",
    description: "Three short practices to start the day.",
    hue: "0x7a5cc7",
    meditations: ["Body Scan", "Breath Focus", "Quiet Mind"],
  },
  {
    title: "Evening Rest",
    description: "Wind down before sleep.",
    hue: "0x2f6f8f",
    meditations: ["Slow Breathing", "Gratitude Pause"],
  },
  {
    title: "Focus Reset",
    description: "Clear your head between tasks.",
    hue: "0x3f8f6b",
    meditations: ["Five Minute Anchor", "Walking Stillness"],
  },
] as const;

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
  const covers = DEMO_PROGRAMS.map((program, index) => {
    const file = path.join(demoDir, `cover-${index}.png`);
    if (!fs.existsSync(file)) {
      ffmpeg([
        "-f",
        "lavfi",
        "-i",
        `color=c=${program.hue}:s=800x450,format=rgb24,geq=r='r(X,Y)*(1-Y/H*0.6)':g='g(X,Y)*(1-Y/H*0.6)':b='b(X,Y)*(1-Y/H*0.4)'`,
        "-frames:v",
        "1",
        file,
      ]);
    }
    return file;
  });
  return { tone, loop, covers };
}
