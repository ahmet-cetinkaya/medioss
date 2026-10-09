// The Player's background soundscape (owner feedback): one MixState (and so
// one MixEngine) per screen, that follows the meditation's play and pause
// and can also be played, stopped, and edited on its own from the dialog.
// The library loads once; a failed load is retried on the next need.

import {
  fetchSoundscapeCategories,
  fetchSoundscapeMixes,
  fetchSounds,
  type SoundscapeCategory,
  type SoundscapeMix,
  type SoundscapeSound,
} from "$lib/api/soundscape";
import { readLastMixId } from "$lib/soundscape/last-mix";
import { randomMix } from "$lib/soundscape/random-mix";
import { MixState } from "../soundscape/mix-state.svelte";

export class PlayerMix {
  readonly state = new MixState();
  sounds = $state<SoundscapeSound[]>([]);
  categories = $state<SoundscapeCategory[]>([]);
  mixes = $state<SoundscapeMix[]>([]);
  loaded = $state(false);
  #loading: Promise<void> | null = null;

  /** Loads sounds, categories and saved mixes once. Never rejects. */
  load = (): Promise<void> => {
    if (this.loaded) return Promise.resolve();
    this.#loading ??= Promise.all([
      fetchSounds(),
      fetchSoundscapeCategories(),
      fetchSoundscapeMixes(),
    ])
      .then(([sounds, categories, mixes]) => {
        this.sounds = sounds;
        this.categories = categories;
        this.mixes = mixes;
        this.loaded = true;
      })
      .catch(() => {})
      .finally(() => (this.#loading = null));
    return this.#loading;
  };

  /** The last used saved mix, else a random one at volume 50. */
  #fill(): void {
    const last = readLastMixId();
    const saved = this.mixes.find((mix) => mix.id === last);
    if (saved) this.state.load(saved, this.sounds);
    if (this.state.entries.length > 0) return;
    for (const { sound, volume } of randomMix(this.sounds)) {
      this.state.toggle(sound);
      this.state.setVolume(sound.id, volume);
    }
  }

  /** The meditation started: play the mix, filling it first when empty. */
  start = async (stillWanted: () => boolean): Promise<void> => {
    if (this.state.entries.length === 0) {
      await this.load();
      if (!stillWanted() || this.state.entries.length > 0) return;
      this.#fill();
    }
    if (stillWanted()) await this.state.play();
  };

  /** The meditation paused or ended: the mix holds its place. */
  pause = (): void => this.state.engine.pause();

  dispose = (): void => this.state.dispose();
}
