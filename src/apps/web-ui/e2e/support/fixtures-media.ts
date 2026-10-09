// Media and import fixtures generated with ffmpeg into .agents/qa/fixtures/.
// Generation is skipped for files that already exist.
import { execFileSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fixturesDir } from "./server-env";

export const FIXTURES = {
  toneMp3: path.join(fixturesDir, "tone-20s.mp3"),
  coverPng: path.join(fixturesDir, "cover-640x480.png"),
  oversizeAvatarPng: path.join(fixturesDir, "avatar-3mb.png"),
  loopOgg: path.join(fixturesDir, "loop-30s.ogg"),
  badJson: path.join(fixturesDir, "bad.json"),
  badCsv: path.join(fixturesDir, "bad.csv"),
} as const;

const MIN_OVERSIZE_BYTES = 3 * 1024 * 1024;

function ffmpeg(args: string[], input?: Buffer): void {
  execFileSync(
    "ffmpeg",
    ["-hide_banner", "-loglevel", "error", "-y", ...args],
    {
      input,
      stdio: [input ? "pipe" : "ignore", "ignore", "inherit"],
    },
  );
}

function ensure(file: string, make: () => void): void {
  if (!fs.existsSync(file)) make();
}

export function ensureFixtures(): typeof FIXTURES {
  fs.mkdirSync(fixturesDir, { recursive: true });
  const f = FIXTURES;
  ensure(f.toneMp3, () =>
    ffmpeg([
      "-f",
      "lavfi",
      "-i",
      "sine=frequency=440:duration=20",
      "-c:a",
      "libmp3lame",
      "-b:a",
      "64k",
      f.toneMp3,
    ]),
  );
  ensure(f.coverPng, () =>
    ffmpeg([
      "-f",
      "lavfi",
      "-i",
      "testsrc2=size=640x480:rate=1",
      "-frames:v",
      "1",
      f.coverPng,
    ]),
  );
  // Random pixels do not compress, so 1024x1024 RGB stays above 3 MiB.
  ensure(f.oversizeAvatarPng, () =>
    ffmpeg(
      [
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-s",
        "1024x1024",
        "-i",
        "pipe:0",
        "-frames:v",
        "1",
        f.oversizeAvatarPng,
      ],
      randomBytes(1024 * 1024 * 3),
    ),
  );
  ensure(f.loopOgg, () =>
    ffmpeg([
      "-f",
      "lavfi",
      "-i",
      "anoisesrc=d=30:c=pink:a=0.2",
      "-c:a",
      "libvorbis",
      "-q:a",
      "2",
      f.loopOgg,
    ]),
  );
  ensure(f.badJson, () =>
    fs.writeFileSync(f.badJson, '{"format_version": 1, "mood": [{"score": '),
  );
  ensure(f.badCsv, () =>
    fs.writeFileSync(
      f.badCsv,
      "recorded_at;score\nnot-a-date;eleven;extra;column\n",
    ),
  );

  if (fs.statSync(f.oversizeAvatarPng).size < MIN_OVERSIZE_BYTES) {
    throw new Error(`${f.oversizeAvatarPng} is smaller than 3 MiB`);
  }
  return f;
}

export const readFixture = (file: string): Buffer => fs.readFileSync(file);
