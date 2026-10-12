<!--
  @component
  The current mix (FR-050, PB-6/PB-7/PB-8, plan D10): what is playing now.
  A tinted panel with an explicit heading, one row per sound (icon, name,
  volume slider 0–100, remove), and a controls row: Play/Stop, a Random
  button (replaces the contents with `randomMix`: 3 distinct sounds at 50,
  confirm-free — removing or adjusting sounds undoes it; keeps playing when
  it was playing), a Sleep timer button (dialog; only while playing) and a
  Save button (name dialog; needs a sound and a trimmed name). Audio lives in
  the shared MixEngine behind `mix`; the fade gain never changes the sliders,
  so saving stores the sliders.
-->
<script lang="ts">
  import {
    createSoundscapeMix,
    type SoundscapeCategory,
    type SoundscapeMix,
    type SoundscapeSound,
  } from "$lib/api/soundscape";
  import { reasonOf } from "$lib/api/failure";
  import RenameDialog from "$lib/components/dialogs/rename-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import SoundIcon from "$lib/components/icons/sound-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { IconButton } from "$lib/components/ui/icon-button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { formatClock } from "$lib/i18n/formatting/clock-format";
  import { t } from "$lib/i18n/translate";
  import MasterVolumeControl from "./master-volume-control.svelte";
  import type { MixState } from "./mix-state.svelte";
  import { soundscapeMessages as m } from "./soundscape.messages";
  import TimerDialog from "./timer-dialog.svelte";

  type Props = {
    mix: MixState;
    /** The sound library; the Random action draws from it. */
    sounds: SoundscapeSound[];
    /** For the icon fallback of sounds without their own icon. */
    categories?: SoundscapeCategory[];
    onSaved: (mix: SoundscapeMix) => void;
  };

  let { mix, sounds = [], categories = [], onSaved }: Props = $props();

  let timerOpen = $state(false);
  let saveOpen = $state(false);
  let timerButton = $state<HTMLElement | null>(null);

  function cancelTimer() {
    mix.cancelTimer();
    timerButton?.focus();
  }

  const slug = (categoryId: number) =>
    categories.find((category) => category.id === categoryId)?.slug;
  const empty = $derived(mix.entries.length === 0);

  /** Random replaces the mix (reversible: remove or adjust sounds) and keeps
      playing when it was playing; a stop-then-fill would need a second click. */
  async function fillRandom() {
    const wasPlaying = mix.playing;
    const picked = mix.replaceWithRandom(sounds);
    announce(t(m.randomMixFilled, { count: picked.length }));
    if (wasPlaying) await mix.play();
  }

  async function save(name: string) {
    const saved = await createSoundscapeMix(name, mix.payload());
    mix.markSaved(saved);
    onSaved(saved);
    announce(t(m.mixSaved, { name: saved.name }));
  }

  /** Generic refusals show the save message; specific ones their own reason. */
  const saveFailure = (error: unknown) =>
    reasonOf(error) === "generic" ? m.saveFailed : null;
</script>

<Panel aria-labelledby="soundscape-mix-heading" tint>
  {#snippet heading()}
    <div class="flex flex-wrap items-baseline justify-between gap-x-3">
      <h2 id="soundscape-mix-heading" class="text-heading text-ink-strong">
        {t(m.mixHeading)}
      </h2>
      {#if !empty}
        <p class="text-sm text-ink-muted">
          {t(m.soundsInMix, { count: mix.entries.length })}
        </p>
      {/if}
    </div>
  {/snippet}
  <div class="flex flex-col gap-5">
    <MasterVolumeControl {mix} />

    {#if mix.missingSounds}
      <InlineMessage tone="info" role="status">
        {t(m.missingSounds)}
      </InlineMessage>
    {/if}

    <RegionState
      status={empty ? "empty" : "ready"}
      minHeight="5rem"
      emptyTitle={t(m.mixEmptyTitle)}
      emptyText={t(m.mixEmptyText)}
    >
      <ul class="flex flex-col gap-2">
        {#each mix.entries as entry (entry.sound.id)}
          <li
            class="flex items-center gap-3 rounded-tile border border-rim bg-surface-solid/70 px-3 py-2"
          >
            <SoundIcon
              sound={entry.sound}
              categorySlug={slug(entry.sound.category_id)}
              size={36}
            />
            <div class="min-w-0 flex-1">
              <p class="pt-1 text-sm font-medium break-words text-ink-strong">
                {entry.sound.name}
                <span class="font-normal text-ink-muted" aria-hidden="true"
                  >· {entry.volume}</span
                >
              </p>
              <Slider
                value={entry.volume}
                oninput={(event) =>
                  mix.setVolume(
                    entry.sound.id,
                    event.currentTarget.valueAsNumber,
                  )}
                label={t(m.volumeLabel, { name: entry.sound.name })}
                valueText={t(m.volumeValue, { value: entry.volume })}
              />
            </div>
            <IconButton
              icon="x"
              label={t(m.removeFromMix, { name: entry.sound.name })}
              onclick={() => mix.remove(entry.sound.id)}
            />
          </li>
        {/each}
      </ul>
    </RegionState>

    <div class="flex flex-col gap-3 border-t border-rim pt-4">
      <div class="flex flex-wrap items-center gap-3">
        <Button disabled={empty} onclick={mix.playing ? mix.stop : mix.play}>
          <AppIcon name={mix.playing ? "pause" : "play"} size={20} filled />
          {t(mix.playing ? m.stopMix : m.playMix)}
        </Button>
        <Button
          variant="secondary"
          disabled={sounds.length === 0}
          onclick={fillRandom}
        >
          <AppIcon name="shuffle" size={20} />
          {t(m.randomMixButton)}
        </Button>
        <Button
          variant="secondary"
          bind:ref={timerButton}
          disabled={!mix.playing}
          onclick={() => (timerOpen = true)}
        >
          <AppIcon name="timer" size={20} />
          {t(m.timerButton)}
        </Button>
        <Button
          variant="secondary"
          disabled={empty}
          onclick={() => (saveOpen = true)}
        >
          <AppIcon name="check-circle" size={20} />
          {t(m.saveMix)}
        </Button>
      </div>

      <p
        class="inline-flex flex-wrap items-center gap-2 text-sm font-medium text-ink-strong tabular-nums"
        aria-live="polite"
        aria-atomic="true"
      >
        {#if mix.remaining !== null}
          <AppIcon name="timer" size={20} />
          <span>{t(m.timeLeft)}</span>
          <span>{formatClock(mix.remaining)}</span>
        {/if}
      </p>
      {#if mix.remaining !== null}
        <div>
          <Button variant="ghost" onclick={cancelTimer}>
            {t(m.cancelTimer)}
          </Button>
        </div>
      {/if}
      {#if !mix.playing}
        <p class="text-sm text-ink-muted">{t(m.timerUnavailable)}</p>
      {/if}
      {#if empty}
        <p class="text-sm text-ink-muted">{t(m.saveUnavailable)}</p>
      {/if}
      {#if mix.playBlocked}
        <InlineMessage tone="error" role="alert">
          {t(m.playBlocked)}
        </InlineMessage>
      {/if}
    </div>
  </div>
</Panel>

<TimerDialog
  bind:open={timerOpen}
  onStart={(minutes) => mix.startTimer(minutes)}
/>

<RenameDialog
  bind:open={saveOpen}
  title={t(m.saveTitle)}
  currentName=""
  label={t(m.mixNameLabel)}
  submitLabel={t(m.saveMix)}
  maxlength={80}
  onSave={save}
  errorMessage={saveFailure}
/>
