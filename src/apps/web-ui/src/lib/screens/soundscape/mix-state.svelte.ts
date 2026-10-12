// The Soundscape screen's current mix (plan D10, PB-6/PB-7/PB-8): a consumer
// of the shared MixEngine, which owns all audio. This adds the screen's
// concerns: which sounds are in the mix (with their data), the sleep timer
// (it only drives the engine's fade gain), the missing-sounds warning and the
// last-used saved mix. One instance per screen mount.

import type { SoundscapeMix, SoundscapeSound } from "$lib/api/soundscape";
import { writeLastMixId } from "$lib/soundscape/last-mix";
import { MixEngine } from "$lib/soundscape/mix-engine.svelte";
import { randomMix } from "$lib/soundscape/random-mix";
import {
  DEFAULT_VOLUME,
  mixEntries,
  mixPayload,
  type MixEntry,
} from "$lib/soundscape/mix-volumes";
import { IDLE_GAIN, isValidTimerMinutes, timerTick } from "./sleep-timer";

const TICK_MS = 250;

export class MixState {
  readonly engine = new MixEngine();
  sounds = $state<SoundscapeSound[]>([]);
  /** Whole seconds left on the sleep timer; null when no timer runs. */
  remaining = $state<number | null>(null);
  /** The last loaded mix had sounds that no longer exist (PB-8). */
  missingSounds = $state(false);

  #timer: ReturnType<typeof setInterval> | undefined;
  #savedId: number | null = null;

  get playing(): boolean {
    return this.engine.playing;
  }
  get master(): number {
    return this.engine.master;
  }
  get playBlocked(): boolean {
    return this.engine.blocked;
  }
  /** Slider entries in mix order; volumes are the slider values. */
  get entries(): MixEntry[] {
    return this.sounds.map((sound) => ({
      sound,
      volume: this.engine.levels[sound.id] ?? DEFAULT_VOLUME,
    }));
  }

  has = (soundId: number): boolean => this.engine.has(soundId);

  toggle = (sound: SoundscapeSound): void => {
    if (this.has(sound.id)) this.remove(sound.id);
    else {
      this.sounds.push(sound);
      this.engine.set(sound.id, DEFAULT_VOLUME);
    }
  };

  remove = (soundId: number): void => {
    this.sounds = this.sounds.filter((sound) => sound.id !== soundId);
    this.engine.remove(soundId);
    if (this.sounds.length === 0) this.stop();
  };

  setVolume = (soundId: number, volume: number): void =>
    this.engine.set(soundId, volume);

  // Overall volume; not part of saved payloads.
  setMaster = (volume: number): void => {
    this.engine.master = volume;
  };

  replaceWithRandom = (library: readonly SoundscapeSound[]): MixEntry[] => {
    const entries = randomMix(library);
    this.stop();
    this.engine.clear();
    this.sounds = [];
    this.missingSounds = false;
    for (const { sound, volume } of entries) {
      this.toggle(sound);
      this.setVolume(sound.id, volume);
    }
    return entries;
  };

  play = async (): Promise<void> => {
    if (this.sounds.length === 0 || this.playing) return;
    if ((await this.engine.play()) && this.#savedId !== null)
      writeLastMixId(this.#savedId);
  };

  stop = (): void => {
    this.cancelTimer();
    this.engine.stop();
  };

  /** Starts the sleep timer; false when not playing or the minutes are invalid. */
  startTimer = (minutes: number): boolean => {
    if (!this.playing || !isValidTimerMinutes(minutes)) return false;
    this.cancelTimer();
    const deadline = Date.now() + minutes * 60_000;
    this.#tick(deadline);
    this.#timer = setInterval(() => this.#tick(deadline), TICK_MS);
    return true;
  };

  /** Stops the countdown and restores the slider volumes (PB-7). */
  cancelTimer = (): void => {
    clearInterval(this.#timer);
    this.#timer = undefined;
    this.remaining = null;
    this.engine.gain = IDLE_GAIN;
  };

  /** Loading stops playback and never starts it (PB-8, FR-051). */
  load = (mix: SoundscapeMix, library: readonly SoundscapeSound[]): void => {
    this.stop();
    this.engine.clear();
    const { entries, missing } = mixEntries(mix, library);
    this.sounds = entries.map((entry) => entry.sound);
    for (const entry of entries) this.engine.set(entry.sound.id, entry.volume);
    this.missingSounds = missing > 0;
    this.#savedId = mix.id;
    writeLastMixId(mix.id);
  };

  /** A freshly saved mix is now the mix in use. */
  markSaved = (mix: SoundscapeMix): void => {
    this.#savedId = mix.id;
    writeLastMixId(mix.id);
  };

  /** The save payload: slider values, never the faded audio volume. */
  payload = () => mixPayload(this.entries);

  /** Leaves the screen: stops the timer and playback, drops the audio. */
  dispose = (): void => {
    this.stop();
    this.engine.clear();
  };

  #tick(deadline: number): void {
    const tick = timerTick(deadline, Date.now());
    this.remaining = tick.remainingSeconds;
    this.engine.gain = tick.gain;
    if (tick.done) this.stop();
  }
}
