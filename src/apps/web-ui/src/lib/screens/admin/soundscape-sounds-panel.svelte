<!-- @component Sounds of the selected category: upload, and per-sound preview, edit, download and delete. -->
<script lang="ts">
  import {
    soundDownloadUrl,
    type SoundscapeCategory,
    type SoundscapeSound,
  } from "$lib/api/soundscape";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import SoundIcon from "$lib/components/icons/sound-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { soundAdminMessages as m } from "./admin.messages";

  let {
    id,
    selected,
    sounds,
    previewingId,
    headingElement = $bindable(null),
    onUpload,
    onPlayPreview,
    onStopPreview,
    onEdit,
    onDelete,
  }: {
    id: string;
    selected: SoundscapeCategory | null;
    sounds: SoundscapeSound[];
    previewingId: number | null;
    headingElement: HTMLElement | null;
    onUpload: () => void;
    onPlayPreview: (sound: SoundscapeSound) => void;
    onStopPreview: () => void;
    onEdit: (sound: SoundscapeSound) => void;
    onDelete: (sound: SoundscapeSound) => void;
  } = $props();
</script>

<Panel class="lg:col-span-3" aria-labelledby="{id}-sounds-heading">
  {#snippet heading()}
    <h2
      id="{id}-sounds-heading"
      bind:this={headingElement}
      tabindex="-1"
      class="text-heading text-ink-strong outline-none"
    >
      {selected ? t(m.soundsIn, { name: selected.name }) : t(m.soundsHeading)}
    </h2>
  {/snippet}
  {#if !selected}
    <p class="text-sm text-ink-muted">{t(m.needCategory)}</p>
  {:else}
    <div class="mb-4">
      <Button variant="secondary" onclick={onUpload}>
        <AppIcon name="upload-simple" size={20} />
        {t(m.uploadSound)}
      </Button>
    </div>
    <RegionState
      status={sounds.length ? "ready" : "empty"}
      minHeight="8rem"
      emptyTitle={t(m.noSounds)}
    >
      <ul class="grid grid-cols-1 gap-3 md:grid-cols-2">
        {#each sounds as sound (sound.id)}
          <li
            class="flex items-center gap-3 rounded-tile bg-surface-solid/50 p-2 ps-3"
          >
            <SoundIcon {sound} categorySlug={selected.slug} />
            <span
              class="flex min-w-0 flex-1 flex-wrap items-center gap-2 font-medium break-words text-ink-strong"
            >
              {sound.name}
              {#if sound.builtin}
                <span
                  class="rounded-full border border-rim bg-surface-solid px-2 py-0.5 text-xs font-medium text-ink-body"
                >
                  {t(m.builtin)}
                </span>
              {/if}
            </span>
            <OverflowMenu label={t(m.soundActions, { name: sound.name })}>
              {#if previewingId === sound.id}
                <OverflowItem
                  label={t(m.stopPreview)}
                  icon="pause"
                  onclick={onStopPreview}
                />
              {:else}
                <OverflowItem
                  label={t(m.previewSound)}
                  icon="play"
                  onclick={() => onPlayPreview(sound)}
                />
              {/if}
              <OverflowItem
                label={t(actionMessages.edit)}
                icon="pencil-simple"
                onclick={() => onEdit(sound)}
              />
              <OverflowItem
                label={t(actionMessages.download)}
                icon="download-simple"
                onclick={() =>
                  (window.location.href = soundDownloadUrl(sound.id))}
              />
              <OverflowItem
                label={t(actionMessages.delete)}
                icon="trash"
                destructive
                onclick={() => onDelete(sound)}
              />
            </OverflowMenu>
          </li>
        {/each}
      </ul>
    </RegionState>
  {/if}
</Panel>
