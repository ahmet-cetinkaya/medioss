<!--
  @component
  Admin Soundscape section (FR-077, FR-078, PB-13, PB-16). Sound categories:
  add, rename (RenameDialog), delete (ConfirmDialog; a category with sounds
  is refused as `categoryInUse`). Sounds: upload, edit in a FormDialog with a
  name field and a category chosen by name, delete (confirm: removed from
  every mix), the built-in badge, and downloads.
-->
<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { type ServerReason } from "$lib/api/server-reason";
  import {
    failureMessage,
    NETWORK_OR_SERVER_REASONS,
    reasonOf,
  } from "$lib/api/failure";
  import {
    createSoundscapeCategory,
    clearSoundIcon,
    deleteSound,
    deleteSoundscapeCategory,
    fetchSoundscapeCategories,
    fetchSounds,
    renameSoundscapeCategory,
    soundAudioUrl,
    soundDownloadUrl,
    updateSound,
    uploadSound,
    uploadSoundIcon,
    type SoundscapeCategory,
    type SoundscapeSound,
  } from "$lib/api/soundscape";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import FormDialog, {
    describeFailure,
    type Failure,
  } from "$lib/components/dialogs/form-dialog.svelte";
  import RenameDialog from "$lib/components/dialogs/rename-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { adminMessages, soundAdminMessages as m } from "./admin.messages";
  import NativeSelect from "./native-select.svelte";
  import SoundIconPicker from "./sound-icon-picker.svelte";
  import SoundIcon from "$lib/components/icons/sound-icon.svelte";

  type Data = { categories: SoundscapeCategory[]; sounds: SoundscapeSound[] };

  const id = $props.id();

  let data = $state<Data | null>(null);
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
  const categoryName = (categoryId: number) =>
    data?.categories.find((category) => category.id === categoryId)?.name ?? "";

  // Selected category (defaults to the first); its sounds are listed.
  let selectedId = $state<number | null>(null);
  const selected = $derived(
    data?.categories.find((c) => c.id === selectedId) ??
      data?.categories[0] ??
      null,
  );
  const selectedSounds = $derived(
    selected
      ? (data?.sounds ?? []).filter((x) => x.category_id === selected.id)
      : [],
  );
  const soundCount = (categoryId: number) =>
    (data?.sounds ?? []).filter((x) => x.category_id === categoryId).length;

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

  // Local admin preview: one audio element at a time, released on unmount.
  let previewId = $state<number | null>(null);
  let preview: HTMLAudioElement | null = null;

  function stopPreview() {
    if (preview) {
      preview.pause();
      preview.removeAttribute("src");
      preview.load();
      preview = null;
    }
    previewId = null;
  }

  function playPreview(sound: SoundscapeSound) {
    stopPreview();
    const audio = new Audio(soundAudioUrl(sound.id));
    preview = audio;
    previewId = sound.id;
    audio.addEventListener("ended", () => {
      if (preview === audio) stopPreview();
    });
    audio.addEventListener("error", () => {
      if (preview !== audio) return;
      stopPreview();
      announce(t(m.previewFailed, { name: sound.name }));
    });
    audio.play().catch(() => {
      if (preview !== audio) return;
      stopPreview();
      announce(t(m.previewFailed, { name: sound.name }));
    });
  }

  onDestroy(stopPreview);

  // Add a category (dialog).
  let addCategoryOpen = $state(false);
  let newCategory = $state("");

  function askAddCategory() {
    newCategory = "";
    addCategoryOpen = true;
  }

  async function runAddCategory() {
    const category = await createSoundscapeCategory(newCategory.trim());
    announce(t(m.categoryAdded, { name: category.name }));
    selectedId = category.id;
    await load(false);
  }

  // Rename and delete a category.
  let categoryTarget = $state<SoundscapeCategory | null>(null);
  let renameOpen = $state(false);
  let deleteCategoryOpen = $state(false);
  let categoriesHeading = $state<HTMLElement | null>(null);

  function askRename(category: SoundscapeCategory) {
    categoryTarget = category;
    renameOpen = true;
  }

  async function runRename(name: string) {
    if (!categoryTarget) return;
    await renameSoundscapeCategory(categoryTarget.id, name);
    await load(false);
  }

  function askDeleteCategory(category: SoundscapeCategory) {
    categoryTarget = category;
    deleteCategoryOpen = true;
  }

  async function runDeleteCategory() {
    const target = categoryTarget;
    if (!target) return;
    await deleteSoundscapeCategory(target.id);
    announce(t(m.categoryDeleted, { name: target.name }));
    await load(false);
  }

  // Upload a sound (dialog; category prefilled from the selection).
  let uploadOpen = $state(false);
  let soundName = $state("");
  let soundCategory = $state("");
  let soundFile = $state<File | null>(null);
  let soundIconKey = $state("");
  let soundIconFile = $state<File | null>(null);
  let soundIconClear = $state(false);
  let iconPickerKey = $state(0);

  function askUpload() {
    soundName = "";
    soundCategory = selected ? String(selected.id) : "";
    soundFile = null;
    soundIconKey = "";
    soundIconFile = null;
    iconPickerKey++;
    uploadOpen = true;
  }

  async function runUpload() {
    const name = soundName.trim();
    const categoryId = Number(soundCategory);
    if (!name || !categoryId || !soundFile) return;
    const sound = await uploadSound(name, categoryId, soundFile, {
      iconKey: soundIconKey,
      iconFile: soundIconFile,
    });
    announce(t(m.soundUploaded, { name: sound.name }));
    selectedId = categoryId;
    await load(false);
  }

  // Edit and delete a sound.
  let soundTarget = $state<SoundscapeSound | null>(null);
  let editOpen = $state(false);
  let editName = $state("");
  let editCategory = $state("");
  let editIconKey = $state("");
  let editIconFile = $state<File | null>(null);
  let editIconClear = $state(false);
  let deleteSoundOpen = $state(false);
  let soundsHeading = $state<HTMLElement | null>(null);

  function askEdit(sound: SoundscapeSound) {
    soundTarget = sound;
    editName = sound.name;
    editCategory = String(sound.category_id);
    editIconKey = sound.icon_key ?? "";
    editIconFile = null;
    editIconClear = false;
    editOpen = true;
  }

  async function runEdit() {
    const target = soundTarget;
    if (!target) return;
    const name = editName.trim();
    await updateSound(
      target.id,
      name,
      Number(editCategory),
      editIconKey || null,
    );
    if (editIconFile) await uploadSoundIcon(target.id, editIconFile);
    else if (editIconClear && target.has_icon) await clearSoundIcon(target.id);
    announce(t(m.soundUpdated, { name }));
    await load(false);
  }

  function askDeleteSound(sound: SoundscapeSound) {
    soundTarget = sound;
    deleteSoundOpen = true;
  }

  async function runDeleteSound() {
    const target = soundTarget;
    if (!target) return;
    if (previewId === target.id) stopPreview();
    await deleteSound(target.id);
    announce(t(m.soundDeleted, { name: target.name }));
    await load(false);
  }
</script>

<RegionState
  {status}
  minHeight="24rem"
  errorMessage={loadErrorText}
  onRetry={() => void load()}
>
  {#if data}
    <div class="grid grid-cols-1 gap-6 lg:grid-cols-5 lg:items-start">
      <Panel class="lg:col-span-2" aria-labelledby="{id}-categories-heading">
        {#snippet heading()}
          <h2
            id="{id}-categories-heading"
            bind:this={categoriesHeading}
            tabindex="-1"
            class="text-heading text-ink-strong outline-none"
          >
            {t(m.categoriesHeading)}
          </h2>
        {/snippet}
        {#if data.categories.length}
          <ul class="grid grid-cols-1 gap-2">
            {#each data.categories as category (category.id)}
              <li
                class="flex items-center gap-1 rounded-tile ps-1 pe-1 {category.id ===
                selected?.id
                  ? 'bg-panel-tint'
                  : 'bg-surface-solid/50'}"
              >
                <button
                  type="button"
                  aria-label={t(m.selectCategory, { name: category.name })}
                  aria-current={category.id === selected?.id
                    ? "true"
                    : undefined}
                  class="focus-ring min-h-11 min-w-0 flex-1 rounded-tile px-3 py-2 text-start font-medium break-words text-ink-strong"
                  onclick={() => (selectedId = category.id)}
                >
                  {t(m.categoryCount, {
                    name: category.name,
                    count: soundCount(category.id),
                  })}
                </button>
                <OverflowMenu
                  label={t(m.categoryActions, { name: category.name })}
                >
                  <OverflowItem
                    label={t(actionMessages.rename)}
                    icon="pencil-simple"
                    onclick={() => askRename(category)}
                  />
                  <OverflowItem
                    label={t(actionMessages.delete)}
                    icon="trash"
                    destructive
                    onclick={() => askDeleteCategory(category)}
                  />
                </OverflowMenu>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="text-sm text-ink-muted">{t(m.noCategories)}</p>
        {/if}
        <div class="mt-4">
          <Button variant="secondary" onclick={askAddCategory}>
            <AppIcon name="plus" size={20} />
            {t(m.addCategory)}
          </Button>
        </div>
      </Panel>

      <Panel class="lg:col-span-3" aria-labelledby="{id}-sounds-heading">
        {#snippet heading()}
          <h2
            id="{id}-sounds-heading"
            bind:this={soundsHeading}
            tabindex="-1"
            class="text-heading text-ink-strong outline-none"
          >
            {selected
              ? t(m.soundsIn, { name: selected.name })
              : t(m.soundsHeading)}
          </h2>
        {/snippet}
        {#if !selected}
          <p class="text-sm text-ink-muted">{t(m.needCategory)}</p>
        {:else}
          <div class="mb-4">
            <Button variant="secondary" onclick={askUpload}>
              <AppIcon name="upload-simple" size={20} />
              {t(m.uploadSound)}
            </Button>
          </div>
          <RegionState
            status={selectedSounds.length ? "ready" : "empty"}
            minHeight="8rem"
            emptyTitle={t(m.noSounds)}
          >
            <ul class="grid grid-cols-1 gap-3 md:grid-cols-2">
              {#each selectedSounds as sound (sound.id)}
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
                    {#if previewId === sound.id}
                      <OverflowItem
                        label={t(m.stopPreview)}
                        icon="pause"
                        onclick={stopPreview}
                      />
                    {:else}
                      <OverflowItem
                        label={t(m.previewSound)}
                        icon="play"
                        onclick={() => playPreview(sound)}
                      />
                    {/if}
                    <OverflowItem
                      label={t(actionMessages.edit)}
                      icon="pencil-simple"
                      onclick={() => askEdit(sound)}
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
                      onclick={() => askDeleteSound(sound)}
                    />
                  </OverflowMenu>
                </li>
              {/each}
            </ul>
          </RegionState>
        {/if}
      </Panel>
    </div>
  {/if}
</RegionState>

<FormDialog
  bind:open={addCategoryOpen}
  title={t(m.addCategoryTitle)}
  submitLabel={t(m.addCategory)}
  canSubmit={newCategory.trim() !== ""}
  onSubmit={runAddCategory}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-new-category">{t(m.categoryName)}</Label>
    <Input
      id="{id}-new-category"
      bind:value={newCategory}
      maxlength={80}
      autocomplete="off"
      required
    />
  </div>
</FormDialog>

<FormDialog
  bind:open={uploadOpen}
  title={t(m.uploadSound)}
  submitLabel={t(m.uploadSound)}
  canSubmit={soundName.trim() !== "" && soundCategory !== "" && !!soundFile}
  onSubmit={runUpload}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-sound-name">{t(m.soundName)}</Label>
    <Input id="{id}-sound-name" bind:value={soundName} required />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-sound-category">{t(m.category)}</Label>
    <NativeSelect id="{id}-sound-category" bind:value={soundCategory} required>
      <option value="" disabled>{t(adminMessages.chooseOption)}</option>
      {#each data?.categories ?? [] as category (category.id)}
        <option value={String(category.id)}>{category.name}</option>
      {/each}
    </NativeSelect>
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-sound-file">{t(m.audioFile)}</Label>
    <Input
      id="{id}-sound-file"
      type="file"
      accept="audio/*,.mp3,.wav,.ogg,.opus"
      onchange={(event) => (soundFile = event.currentTarget.files?.[0] ?? null)}
      required
    />
  </div>
  {#key iconPickerKey}
    <SoundIconPicker
      idPrefix="{id}-new"
      bind:key={soundIconKey}
      bind:file={soundIconFile}
      bind:clearImage={soundIconClear}
    />
  {/key}
</FormDialog>

<RenameDialog
  bind:open={renameOpen}
  title={t(m.renameCategoryTitle)}
  currentName={categoryTarget?.name ?? ""}
  onSave={runRename}
  maxlength={80}
/>

<ConfirmDialog
  bind:open={deleteCategoryOpen}
  title={t(m.deleteCategoryTitle)}
  itemName={categoryTarget?.name}
  consequence={t(m.deleteCategoryConsequence)}
  confirmLabel={t(m.deleteCategoryConfirm)}
  destructive
  onConfirm={runDeleteCategory}
  fallbackFocus={() => categoriesHeading}
/>

<FormDialog
  bind:open={editOpen}
  title={t(m.editTitle)}
  submitLabel={t(actionMessages.save)}
  canSubmit={editName.trim() !== "" && editCategory !== ""}
  onSubmit={runEdit}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-name">{t(m.soundName)}</Label>
    <Input
      id="{id}-edit-name"
      bind:value={editName}
      maxlength={80}
      autocomplete="off"
      required
    />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-category">{t(m.category)}</Label>
    <NativeSelect id="{id}-edit-category" bind:value={editCategory} required>
      {#each data?.categories ?? [] as category (category.id)}
        <option value={String(category.id)}>{category.name}</option>
      {/each}
    </NativeSelect>
  </div>
  <SoundIconPicker
    idPrefix="{id}-edit"
    bind:key={editIconKey}
    bind:file={editIconFile}
    bind:clearImage={editIconClear}
    hasImage={soundTarget?.has_icon ?? false}
    soundId={soundTarget?.id}
  />
</FormDialog>

<ConfirmDialog
  bind:open={deleteSoundOpen}
  title={t(m.deleteSoundTitle)}
  itemName={soundTarget?.name}
  consequence={t(m.deleteSoundConsequence)}
  confirmLabel={t(m.deleteSoundConfirm)}
  destructive
  onConfirm={runDeleteSound}
  fallbackFocus={() => soundsHeading}
/>
