<!--
  @component
  The Player's mixer dialog: Play or Stop for the background mix, a Random
  button that fills the mix (3 distinct sounds at volume 50, started when the
  meditation is playing), a volume row (icon, name, 0–100 slider, remove) per
  sound in the mix, saved mixes to load (which become the last used mix), and
  the other sounds to add, grouped by category (uncategorized last).
-->
<script lang="ts">
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import SoundIcon from "$lib/components/icons/sound-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { IconButton } from "$lib/components/ui/icon-button/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import MasterVolumeControl from "../soundscape/master-volume-control.svelte";
  import { groupAddableSounds } from "./mix-add-groups";
  import type { PlayerMix } from "./player-mix.svelte";
  import { playerMessages as m } from "./player.messages";

  type Props = { open?: boolean; mix: PlayerMix; playing: boolean };

  let { open = $bindable(false), mix, playing }: Props = $props();

  const state = mix.state;
  const entries = $derived(state.entries);
  const available = $derived(
    mix.sounds.filter((sound) => !state.has(sound.id)),
  );
  const groups = $derived(groupAddableSounds(available, mix.categories));
  const slug = (categoryId: number) =>
    mix.categories.find((category) => category.id === categoryId)?.slug;

  $effect(() => {
    if (open) void mix.load();
  });

  // A fresh random mix: 3 distinct sounds at volume 50, replacing whatever
  // is in the mix now, and started too when the meditation is playing.
  async function randomize() {
    await mix.load();
    const picked = state.replaceWithRandom(mix.sounds);
    announce(t(m.mixRandomLoaded, { count: picked.length }));
    if (playing && picked.length > 0) await state.play();
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content showCloseButton={false}>
    <Dialog.Header>
      <Dialog.Title>{t(m.mixDialogTitle)}</Dialog.Title>
      <Dialog.Description>{t(m.mixDialogDescription)}</Dialog.Description>
    </Dialog.Header>

    <div class="grid max-h-[60vh] gap-4 overflow-y-auto">
      <MasterVolumeControl mix={state} />
      <div class="flex flex-wrap items-center gap-3">
        <Button
          disabled={entries.length === 0}
          onclick={() => (state.playing ? state.stop() : void state.play())}
        >
          <AppIcon name={state.playing ? "pause" : "play"} size={20} filled />
          {t(state.playing ? m.mixStop : m.mixPlay)}
        </Button>
        <Button
          variant="secondary"
          disabled={!mix.loaded || mix.sounds.length === 0}
          onclick={() => void randomize()}
        >
          <AppIcon name="shuffle" size={20} />
          {t(m.mixRandom)}
        </Button>
      </div>
      {#if state.playBlocked}
        <InlineMessage tone="error" role="alert"
          >{t(m.mixBlocked)}</InlineMessage
        >
      {/if}

      {#if !mix.loaded}
        <p class="text-sm text-ink-muted" role="status">{t(m.mixLoading)}</p>
      {/if}

      {#if entries.length > 0}
        <ul class="flex flex-col gap-2">
          {#each entries as entry (entry.sound.id)}
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
                </p>
                <Slider
                  value={entry.volume}
                  oninput={(event) =>
                    state.setVolume(
                      entry.sound.id,
                      event.currentTarget.valueAsNumber,
                    )}
                  label={t(m.volumeLabel, { name: entry.sound.name })}
                  valueText={t(m.volumeValue, { value: entry.volume })}
                />
              </div>
              <IconButton
                icon="x"
                label={t(m.removeSound, { name: entry.sound.name })}
                onclick={() => state.remove(entry.sound.id)}
              />
            </li>
          {/each}
        </ul>
      {/if}

      {#if mix.mixes.length > 0}
        <section aria-labelledby="player-mix-saved" class="grid gap-2">
          <h3
            id="player-mix-saved"
            class="text-sm font-semibold text-ink-strong"
          >
            {t(m.mixSavedHeading)}
          </h3>
          <ul class="flex flex-wrap gap-2">
            {#each mix.mixes as saved (saved.id)}
              <li>
                <Button
                  variant="secondary"
                  onclick={() => {
                    state.load(saved, mix.sounds);
                    announce(t(m.mixLoaded, { name: saved.name }));
                  }}
                >
                  {saved.name}
                </Button>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if mix.loaded}
        <section aria-labelledby="player-mix-add" class="grid gap-3">
          <h3 id="player-mix-add" class="text-sm font-semibold text-ink-strong">
            {t(m.mixAddHeading)}
          </h3>
          {#if mix.sounds.length === 0}
            <p class="text-sm text-ink-muted">{t(m.mixNoSounds)}</p>
          {/if}
          {#each groups as group (group.key)}
            <div class="grid gap-2">
              <h4
                id="player-mix-{group.key}"
                class="text-sm font-semibold text-ink-strong"
              >
                {group.category?.name ?? t(m.uncategorized)}
              </h4>
              <ul class="flex flex-wrap gap-2">
                {#each group.sounds as sound (sound.id)}
                  <li>
                    <Button
                      variant="secondary"
                      aria-label={t(m.addSound, { name: sound.name })}
                      onclick={() => state.toggle(sound)}
                    >
                      <SoundIcon
                        {sound}
                        categorySlug={slug(sound.category_id)}
                        size={24}
                      />
                      {sound.name}
                    </Button>
                  </li>
                {/each}
              </ul>
            </div>
          {/each}
        </section>
      {/if}
    </div>

    <Dialog.Footer>
      <Button variant="ghost" onclick={() => (open = false)}>
        {t(actionMessages.close)}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
