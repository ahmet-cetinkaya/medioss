// Shared soundscape mix engine (PB-6): one looping HTMLAudioElement per sound
// id. A sound's level (0–100) is `$state`; the element volume is derived from
// it, `gain` and `master`, so a sleep-timer fade (gain 1 → 0) or the overall
// volume never touches the levels and saving stores what the user chose. Elements are created lazily, on the
// first `set`, so nothing here touches the browser at module load or on
// construction. Each element is hidden in <body> with `data-sound-id`.

import { soundAudioUrl } from "../api/soundscape";
import { audioVolume, clampVolume, DEFAULT_VOLUME } from "./mix-volumes";

export class MixEngine {
  /** Level (slider value 0–100) per sound id. */
  levels = $state<Record<number, number>>({});
  /** Playback was requested and started (false while stopped or paused). */
  playing = $state(false);
  /** The browser refused to start playback (autoplay policy, PB-6). */
  blocked = $state(false);
  #gain = $state(1);
  #master = $state(100);
  #elements = new Map<number, HTMLAudioElement>();

  /** Fade multiplier 0–1 applied on top of every level (sleep timer). */
  get gain(): number {
    return this.#gain;
  }
  set gain(value: number) {
    this.#gain = value;
    for (const id of this.#elements.keys()) this.#apply(id);
  }

  /** Master (overall) volume 0–100 applied on top of every level. */
  get master(): number {
    return this.#master;
  }
  set master(value: number) {
    this.#master = clampVolume(value);
    for (const id of this.#elements.keys()) this.#apply(id);
  }

  has = (id: number): boolean => id in this.levels;

  /** Adds a sound, or changes its level. A sound added while playing starts. */
  set = (id: number, level: number = DEFAULT_VOLUME): void => {
    this.levels[id] = clampVolume(level);
    if (!this.#elements.has(id)) {
      const element = document.createElement("audio");
      element.src = soundAudioUrl(id);
      element.loop = true;
      element.preload = "auto";
      element.hidden = true;
      element.dataset.soundId = String(id);
      document.body.append(element);
      this.#elements.set(id, element);
      if (this.playing) void this.#start([element]);
    }
    this.#apply(id);
  };

  remove = (id: number): void => {
    const element = this.#elements.get(id);
    if (!element) return;
    element.pause();
    element.remove();
    this.#elements.delete(id);
    delete this.levels[id];
  };

  /** Starts every sound; sets `blocked` when the browser refuses (PB-6). */
  play = async (): Promise<boolean> => {
    if (this.playing || this.#elements.size === 0) return this.playing;
    this.blocked = false;
    const started = await this.#start([...this.#elements.values()]);
    if (started) this.playing = true;
    else this.#pauseAll();
    return started;
  };

  /** Pauses in place; `resume()` continues from the same position. */
  pause = (): void => {
    this.#pauseAll();
    this.playing = false;
  };

  resume = (): Promise<boolean> => this.play();

  /** Pauses and rewinds every sound. */
  stop = (): void => {
    this.pause();
    for (const element of this.#elements.values()) element.currentTime = 0;
  };

  /** Removes every sound and stops; the fade gain resets, the overall volume is kept. */
  clear = (): void => {
    this.stop();
    for (const id of [...this.#elements.keys()]) this.remove(id);
    this.blocked = false;
    this.#gain = 1;
  };

  #apply(id: number): void {
    const element = this.#elements.get(id);
    if (element)
      element.volume = audioVolume(this.levels[id], this.#gain, this.#master);
  }

  async #start(elements: HTMLAudioElement[]): Promise<boolean> {
    try {
      await Promise.all(elements.map((element) => element.play()));
      return true;
    } catch {
      this.blocked = true;
      return false;
    }
  }

  #pauseAll(): void {
    for (const element of this.#elements.values()) element.pause();
  }
}
