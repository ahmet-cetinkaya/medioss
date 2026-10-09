<!--
  @component
  Saved mixes (FR-051–FR-054, PB-8, SC-008): load (stops playback, never
  autoplays, sets every slider to its saved volume), rename (RenameDialog,
  blank refused) and delete (ConfirmDialog naming the mix; Cancel keeps it).
  A mix already deleted elsewhere (404) shows the error and reloads the list.
  After a delete the row is gone, so focus falls back to the "Saved mixes"
  heading.
-->
<script lang="ts">
  import { ApiError } from "$lib/api/api-error";
  import { serverReason } from "$lib/api/server-reason";
  import {
    deleteSoundscapeMix,
    updateSoundscapeMix,
    type SoundscapeMix,
    type SoundscapeSound,
  } from "$lib/api/soundscape";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import RenameDialog from "$lib/components/dialogs/rename-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { IconButton } from "$lib/components/ui/icon-button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import type { StringMessage } from "$lib/i18n/message";
  import { t } from "$lib/i18n/translate";
  import type { MixState } from "./mix-state.svelte";
  import { soundscapeMessages as m } from "./soundscape.messages";

  type Props = {
    mix: MixState;
    mixes: SoundscapeMix[] | null;
    /** The sound library; null until it loads (Load waits for it). */
    sounds: SoundscapeSound[] | null;
    status: "loading" | "error" | "empty" | "ready";
    errorMessage?: string;
    onReload: () => void;
  };

  let {
    mix,
    mixes = $bindable(),
    sounds,
    status,
    errorMessage,
    onReload,
  }: Props = $props();

  let headingRef = $state<HTMLElement | null>(null);
  let renameOpen = $state(false);
  let deleteOpen = $state(false);
  let target = $state<SoundscapeMix | null>(null);

  /** Generic refusals show this action's own message; specific ones their reason. */
  const ownFailure = (fallback: StringMessage) => (error: unknown) =>
    error instanceof ApiError && serverReason(error).reason !== "generic"
      ? null
      : fallback;

  /** A mix deleted in another tab: refresh the list, keep the error showing. */
  function refreshIfGone(error: unknown): never {
    if (error instanceof ApiError && error.status === 404) onReload();
    throw error;
  }

  function load(saved: SoundscapeMix) {
    if (!sounds) return;
    mix.load(saved, sounds);
    announce(t(m.mixLoaded, { name: saved.name }));
  }

  async function rename(name: string) {
    const current = target!;
    try {
      const updated = await updateSoundscapeMix(
        current.id,
        name,
        current.items,
      );
      mixes = (mixes ?? []).map((item) =>
        item.id === updated.id ? updated : item,
      );
    } catch (error) {
      refreshIfGone(error);
    }
  }

  async function remove() {
    const current = target!;
    try {
      await deleteSoundscapeMix(current.id);
    } catch (error) {
      refreshIfGone(error);
    }
    mixes = (mixes ?? []).filter((item) => item.id !== current.id);
    announce(t(m.mixDeleted, { name: current.name }));
  }
</script>

<Panel aria-labelledby="soundscape-saved-heading">
  {#snippet heading()}
    <h2
      id="soundscape-saved-heading"
      class="text-heading text-ink-strong focus:outline-none"
      tabindex="-1"
      bind:this={headingRef}
    >
      {t(m.savedHeading)}
    </h2>
  {/snippet}
  <RegionState
    {status}
    minHeight="6rem"
    {errorMessage}
    onRetry={onReload}
    emptyTitle={t(m.noMixesTitle)}
    emptyText={t(m.noMixesText)}
  >
    <ul class="flex flex-col gap-2">
      {#each mixes ?? [] as saved (saved.id)}
        <li
          class="flex flex-wrap items-center justify-between gap-2 rounded-tile bg-surface-solid/50 px-4 py-3"
        >
          <div class="min-w-0 flex-1 basis-32">
            <p class="font-medium break-words text-ink-strong">{saved.name}</p>
            <p class="text-sm text-ink-muted">
              {t(m.soundCount, { count: saved.items.length })}
            </p>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Button
              variant="secondary"
              size="sm"
              disabled={!sounds}
              aria-label={t(m.loadMix, { name: saved.name })}
              onclick={() => load(saved)}
            >
              {t(m.load)}
            </Button>
            <IconButton
              icon="pencil-simple"
              label={t(m.renameMix, { name: saved.name })}
              onclick={() => {
                target = saved;
                renameOpen = true;
              }}
            />
            <IconButton
              icon="trash"
              label={t(m.deleteMix, { name: saved.name })}
              onclick={() => {
                target = saved;
                deleteOpen = true;
              }}
            />
          </div>
        </li>
      {/each}
    </ul>
  </RegionState>
</Panel>

<RenameDialog
  bind:open={renameOpen}
  title={t(m.renameTitle)}
  currentName={target?.name ?? ""}
  label={t(m.mixNameLabel)}
  maxlength={80}
  onSave={rename}
  fallbackFocus={() => headingRef}
  errorMessage={ownFailure(m.renameFailed)}
/>

<ConfirmDialog
  bind:open={deleteOpen}
  title={t(m.deleteTitle)}
  itemName={target?.name}
  consequence={t(m.deleteConsequence)}
  confirmLabel={t(m.deleteConfirm)}
  destructive
  onConfirm={remove}
  fallbackFocus={() => headingRef}
  errorMessage={ownFailure(m.deleteFailed)}
/>
