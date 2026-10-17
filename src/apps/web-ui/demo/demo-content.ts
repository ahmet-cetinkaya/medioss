// Deterministic demo content for a local MediOSS instance. Test-only users
// and passwords come from e2e/support/auth.ts (example.test addresses).
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { qaDir } from "../e2e/support/server-env";

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

const COVER_COLOR_STOPS = 4;
const GOLDEN_ANGLE_DEGREES = 137.5;

function hslToHex(hue: number, saturation: number, lightness: number): string {
  const chroma = saturation * Math.min(lightness, 1 - lightness);
  const channel = (offset: number): string => {
    const k = (offset + hue / 30) % 12;
    const value = lightness - chroma * Math.max(-1, Math.min(k - 3, 9 - k, 1));
    return Math.round(value * 255)
      .toString(16)
      .padStart(2, "0");
  };
  return `0x${channel(0)}${channel(8)}${channel(4)}`;
}

const MUDDY_HUE_FROM = 38;
const MUDDY_HUE_TO = 98;
const COVER_SATURATION = 0.72;
const COVER_LIGHTNESS = 0.5;
const COVER_HUE_STEP_DEGREES = 34;

/** Olive and mustard mixes read as mud: move those hues to clean orange or lime. */
function avoidMuddyHue(hue: number): number {
  if (hue < MUDDY_HUE_FROM || hue > MUDDY_HUE_TO) return hue;
  const middle = (MUDDY_HUE_FROM + MUDDY_HUE_TO) / 2;
  return hue < middle ? MUDDY_HUE_FROM - 8 : MUDDY_HUE_TO + 8;
}

/** A saturated multi-stop gradient whose hues are unique per program index. */
export function gradientSource(index: number): string {
  const baseHue = (index * GOLDEN_ANGLE_DEGREES) % 360;
  const colors = Array.from({ length: COVER_COLOR_STOPS }, (_, stop) =>
    hslToHex(
      avoidMuddyHue((baseHue + stop * COVER_HUE_STEP_DEGREES) % 360),
      COVER_SATURATION,
      COVER_LIGHTNESS,
    ),
  );
  const options = colors.map((color, stop) => `c${stop}=${color}`).join(":");
  return `gradients=s=800x450:${options}:nb_colors=${COVER_COLOR_STOPS}:seed=${index + 1}:speed=0:type=linear`;
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
  const covers = DEMO_PROGRAMS.map((_, index) => {
    const file = path.join(demoDir, `cover-${index}.png`);
    if (!fs.existsSync(file)) {
      ffmpeg([
        "-f",
        "lavfi",
        "-i",
        gradientSource(index),
        "-frames:v",
        "1",
        file,
      ]);
    }
    return file;
  });
  return { tone, loop, covers };
}
