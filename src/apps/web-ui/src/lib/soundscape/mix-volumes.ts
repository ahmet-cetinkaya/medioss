// Mix volumes (plan D10, PB-6/PB-8, FR-051): the slider value (0–100) is the
// source of truth; the audio gain is derived from it and the sleep-timer fade.
// Saving always uses the slider values, never the faded audio volume.

import type {
  SoundscapeMix,
  SoundscapeMixItem,
  SoundscapeSound,
} from "../api/soundscape";

export interface MixEntry {
  sound: SoundscapeSound;
  /** Slider value, 0–100. */
  volume: number;
}

export const DEFAULT_VOLUME = 100;

export function clampVolume(volume: number): number {
  if (!Number.isFinite(volume)) return DEFAULT_VOLUME;
  return Math.min(100, Math.max(0, Math.round(volume)));
}

/**
 * The slider entries of a saved mix, in saved order, plus how many of its
 * items point at sounds that no longer exist.
 */
export function mixEntries(
  mix: Pick<SoundscapeMix, "items">,
  sounds: readonly SoundscapeSound[],
): { entries: MixEntry[]; missing: number } {
  const byId = new Map(sounds.map((sound) => [sound.id, sound]));
  const entries: MixEntry[] = [];
  let missing = 0;
  for (const item of [...mix.items].sort((a, b) => a.position - b.position)) {
    const sound = byId.get(item.sound_id);
    if (sound) entries.push({ sound, volume: clampVolume(item.volume) });
    else missing += 1;
  }
  return { entries, missing };
}

/** The save payload: the slider values, positions in mix order. */
export function mixPayload(
  entries: readonly Pick<MixEntry, "sound" | "volume">[],
): SoundscapeMixItem[] {
  return entries.map((entry, position) => ({
    sound_id: entry.sound.id,
    volume: clampVolume(entry.volume),
    position,
  }));
}

/**
 * The audio element volume (0–1) for a slider value, a fade gain and the
 * master (overall) volume, 0–100.
 */
export function audioVolume(
  volume: number,
  gain: number,
  master = 100,
): number {
  return (
    (clampVolume(volume) / 100) *
    Math.min(1, Math.max(0, gain)) *
    (clampVolume(master) / 100)
  );
}
