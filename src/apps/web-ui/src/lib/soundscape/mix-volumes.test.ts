import { describe, expect, test } from "bun:test";
import type { SoundscapeSound } from "../api/soundscape";
import {
  audioVolume,
  clampVolume,
  mixEntries,
  mixPayload,
} from "./mix-volumes";

const sound = (id: number, name: string): SoundscapeSound => ({
  id,
  name,
  category_id: 1,
  content_type: "audio/ogg",
  size_bytes: 1,
  duration_ms: 30_000,
  created_at: "2026-10-04T00:00:00Z",
  builtin: false,
  icon_key: null,
  has_icon: false,
});

const rain = sound(1, "Rain");
const wind = sound(2, "Wind");

describe("mixEntries", () => {
  test("a saved mix maps to the slider values in saved order", () => {
    const { entries, missing } = mixEntries(
      {
        items: [
          { sound_id: 2, volume: 70, position: 1 },
          { sound_id: 1, volume: 30, position: 0 },
        ],
      },
      [rain, wind],
    );
    expect(missing).toBe(0);
    expect(entries).toEqual([
      { sound: rain, volume: 30 },
      { sound: wind, volume: 70 },
    ]);
  });

  test("items whose sound no longer exists are counted as missing", () => {
    const { entries, missing } = mixEntries(
      {
        items: [
          { sound_id: 1, volume: 50, position: 0 },
          { sound_id: 99, volume: 50, position: 1 },
        ],
      },
      [rain, wind],
    );
    expect(entries).toEqual([{ sound: rain, volume: 50 }]);
    expect(missing).toBe(1);
  });
});

describe("mixPayload", () => {
  test("uses the slider values even while the audio is faded", () => {
    const entries = [
      { sound: rain, volume: 30 },
      { sound: wind, volume: 70 },
    ];
    const fadeGain = 0.2;
    expect(entries.map((e) => audioVolume(e.volume, fadeGain))).toEqual([
      0.3 * 0.2,
      0.7 * 0.2,
    ]);
    expect(mixPayload(entries)).toEqual([
      { sound_id: 1, volume: 30, position: 0 },
      { sound_id: 2, volume: 70, position: 1 },
    ]);
  });
});

describe("clampVolume and audioVolume", () => {
  test("volumes stay whole numbers within 0–100", () => {
    expect(clampVolume(-3)).toBe(0);
    expect(clampVolume(140)).toBe(100);
    expect(clampVolume(42.6)).toBe(43);
    expect(clampVolume(Number.NaN)).toBe(100);
  });

  test("audio volume is slider / 100 times the clamped gain", () => {
    expect(audioVolume(100, 1)).toBe(1);
    expect(audioVolume(50, 1)).toBe(0.5);
    expect(audioVolume(50, 0)).toBe(0);
    expect(audioVolume(50, 2)).toBe(0.5);
  });

  test("master volume scales on top; omitted master is 100", () => {
    expect(audioVolume(80, 1, 50)).toBe(0.4);
    expect(audioVolume(80, 1, 0)).toBe(0);
    expect(audioVolume(80, 0.5)).toBe(audioVolume(80, 0.5, 100));
  });
});
