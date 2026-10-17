import { onDestroy } from "svelte";
import { announce } from "$lib/components/feedback/announcer.svelte";
import { soundAudioUrl, type SoundscapeSound } from "$lib/api/soundscape";
import { t } from "$lib/i18n/translate";
import { soundAdminMessages as m } from "./admin.messages";

/**
 * Local admin preview: one audio element at a time, released when the
 * calling component is destroyed. Call during component initialisation.
 */
export function createSoundPreview() {
  let previewId = $state<number | null>(null);
  let preview: HTMLAudioElement | null = null;

  function stop() {
    if (preview) {
      preview.pause();
      preview.removeAttribute("src");
      preview.load();
      preview = null;
    }
    previewId = null;
  }

  function play(sound: SoundscapeSound) {
    stop();
    const audio = new Audio(soundAudioUrl(sound.id));
    preview = audio;
    previewId = sound.id;
    const fail = () => {
      if (preview !== audio) return;
      stop();
      announce(t(m.previewFailed, { name: sound.name }));
    };
    audio.addEventListener("ended", () => {
      if (preview === audio) stop();
    });
    audio.addEventListener("error", fail);
    audio.play().catch(fail);
  }

  onDestroy(stop);

  return {
    get playingId() {
      return previewId;
    },
    play,
    stop,
  };
}
