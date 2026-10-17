<!--
  @component
  Admin Soundscape section (FR-077, FR-078, PB-13, PB-16). Sound categories:
  add, rename (RenameDialog), delete (ConfirmDialog; a category with sounds
  is refused as `categoryInUse`). Sounds: upload, edit in a FormDialog with a
  name field and a category chosen by name, delete (confirm: removed from
  every mix), the built-in badge, and downloads.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { type ServerReason } from "$lib/api/server-reason";
  import {
    failureMessage,
    NETWORK_OR_SERVER_REASONS,
    reasonOf,
  } from "$lib/api/failure";
  import { fetchSoundscapeCategories, fetchSounds } from "$lib/api/soundscape";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { t } from "$lib/i18n/translate";
  import { adminMessages } from "./admin.messages";
  import CategoryDialogs from "./soundscape-category-dialogs.svelte";
  import CategoriesPanel from "./soundscape-categories-panel.svelte";
  import {
    selectedCategory,
    soundsInCategory,
    type SoundscapeData,
  } from "./soundscape-logic";
  import { createSoundPreview } from "./soundscape-preview.svelte";
  import SoundDialogs from "./soundscape-sound-dialogs.svelte";
  import SoundsPanel from "./soundscape-sounds-panel.svelte";

  const id = $props.id();

  let data = $state<SoundscapeData | null>(null);
  let loadFailure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(loadFailure ? "error" : data ? "ready" : "loading");
  const loadErrorText = $derived(
    t(
      failureMessage(loadFailure, adminMessages.loadFailed, {
        specificReasons: NETWORK_OR_SERVER_REASONS,
      }).message,
    ),
  );

  // Selected category (defaults to the first); its sounds are listed.
  let selectedId = $state<number | null>(null);
  const selected = $derived(
    selectedCategory(data?.categories ?? [], selectedId),
  );
  const selectedSounds = $derived(
    selected ? soundsInCategory(data?.sounds ?? [], selected.id) : [],
  );

  async function load(reset = true) {
    const current = ++request;
    if (reset) {
      data = null;
      loadFailure = null;
    }
    try {
      const [categories, sounds] = await Promise.all([
        fetchSoundscapeCategories(),
        fetchSounds(),
      ]);
      if (current !== request) return;
      data = { categories, sounds };
      loadFailure = null;
    } catch (error) {
      if (current === request) loadFailure = reasonOf(error);
    }
  }

  onMount(() => void load());

  const preview = createSoundPreview();

  let categoryDialogs = $state<ReturnType<typeof CategoryDialogs>>();
  let soundDialogs = $state<ReturnType<typeof SoundDialogs>>();
  let categoriesHeading = $state<HTMLElement | null>(null);
  let soundsHeading = $state<HTMLElement | null>(null);
</script>

<RegionState
  {status}
  minHeight="24rem"
  errorMessage={loadErrorText}
  onRetry={() => void load()}
>
  {#if data}
    <div class="grid grid-cols-1 gap-6 lg:grid-cols-5 lg:items-start">
      <CategoriesPanel
        {id}
        categories={data.categories}
        selectedId={selected?.id}
        countIn={(categoryId) =>
          soundsInCategory(data?.sounds ?? [], categoryId).length}
        bind:headingElement={categoriesHeading}
        onSelect={(categoryId) => (selectedId = categoryId)}
        onAdd={() => categoryDialogs?.askAdd()}
        onRename={(category) => categoryDialogs?.askRename(category)}
        onDelete={(category) => categoryDialogs?.askDelete(category)}
      />
      <SoundsPanel
        {id}
        {selected}
        sounds={selectedSounds}
        previewingId={preview.playingId}
        bind:headingElement={soundsHeading}
        onUpload={() => soundDialogs?.askUpload(selected)}
        onPlayPreview={preview.play}
        onStopPreview={preview.stop}
        onEdit={(sound) => soundDialogs?.askEdit(sound)}
        onDelete={(sound) => soundDialogs?.askDelete(sound)}
      />
    </div>
  {/if}
</RegionState>

<CategoryDialogs
  bind:this={categoryDialogs}
  {id}
  onChanged={() => load(false)}
  onAdded={(category) => (selectedId = category.id)}
  fallbackFocus={() => categoriesHeading}
/>

<SoundDialogs
  bind:this={soundDialogs}
  {id}
  categories={data?.categories ?? []}
  onChanged={() => load(false)}
  onUploaded={(categoryId) => (selectedId = categoryId)}
  onDeleting={(sound) => {
    if (preview.playingId === sound.id) preview.stop();
  }}
  fallbackFocus={() => soundsHeading}
/>
