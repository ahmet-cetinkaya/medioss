<!-- @component Upload, edit and delete sound dialogs; open them through the exported `ask*` functions. -->
<script lang="ts">
  import {
    clearSoundIcon,
    deleteSound,
    updateSound,
    uploadSound,
    uploadSoundIcon,
    type SoundscapeCategory,
    type SoundscapeSound,
  } from "$lib/api/soundscape";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { adminMessages, soundAdminMessages as m } from "./admin.messages";
  import NativeSelect from "./native-select.svelte";
  import SoundIconPicker from "./sound-icon-picker.svelte";

  const SOUND_NAME_MAX_LENGTH = 80;

  let {
    id,
    categories,
    onChanged,
    onUploaded,
    onDeleting,
    fallbackFocus,
  }: {
    id: string;
    categories: SoundscapeCategory[];
    onChanged: () => Promise<void>;
    /** The category the new sound went into, so the screen can select it. */
    onUploaded: (categoryId: number) => void;
    /** Runs before the server delete so the screen can stop a preview. */
    onDeleting: (sound: SoundscapeSound) => void;
    fallbackFocus: () => HTMLElement | null;
  } = $props();

  // Upload (category prefilled from the selection).
  let uploadOpen = $state(false);
  let soundName = $state("");
  let soundCategory = $state("");
  let soundFile = $state<File | null>(null);
  let soundIconKey = $state("");
  let soundIconFile = $state<File | null>(null);
  let soundIconClear = $state(false);
  let iconPickerKey = $state(0);

  export function askUpload(selected: SoundscapeCategory | null) {
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
    onUploaded(categoryId);
    await onChanged();
  }

  // Edit.
  let target = $state<SoundscapeSound | null>(null);
  let editOpen = $state(false);
  let editName = $state("");
  let editCategory = $state("");
  let editIconKey = $state("");
  let editIconFile = $state<File | null>(null);
  let editIconClear = $state(false);

  export function askEdit(sound: SoundscapeSound) {
    target = sound;
    editName = sound.name;
    editCategory = String(sound.category_id);
    editIconKey = sound.icon_key ?? "";
    editIconFile = null;
    editIconClear = false;
    editOpen = true;
  }

  async function runEdit() {
    const sound = target;
    if (!sound) return;
    const name = editName.trim();
    await updateSound(
      sound.id,
      name,
      Number(editCategory),
      editIconKey || null,
    );
    if (editIconFile) await uploadSoundIcon(sound.id, editIconFile);
    else if (editIconClear && sound.has_icon) await clearSoundIcon(sound.id);
    announce(t(m.soundUpdated, { name }));
    await onChanged();
  }

  // Delete (removed from every mix).
  let deleteOpen = $state(false);

  export function askDelete(sound: SoundscapeSound) {
    target = sound;
    deleteOpen = true;
  }

  async function runDelete() {
    const sound = target;
    if (!sound) return;
    onDeleting(sound);
    await deleteSound(sound.id);
    announce(t(m.soundDeleted, { name: sound.name }));
    await onChanged();
  }
</script>

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
      {#each categories as category (category.id)}
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
      maxlength={SOUND_NAME_MAX_LENGTH}
      autocomplete="off"
      required
    />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-category">{t(m.category)}</Label>
    <NativeSelect id="{id}-edit-category" bind:value={editCategory} required>
      {#each categories as category (category.id)}
        <option value={String(category.id)}>{category.name}</option>
      {/each}
    </NativeSelect>
  </div>
  <SoundIconPicker
    idPrefix="{id}-edit"
    bind:key={editIconKey}
    bind:file={editIconFile}
    bind:clearImage={editIconClear}
    hasImage={target?.has_icon ?? false}
    soundId={target?.id}
  />
</FormDialog>

<ConfirmDialog
  bind:open={deleteOpen}
  title={t(m.deleteSoundTitle)}
  itemName={target?.name}
  consequence={t(m.deleteSoundConsequence)}
  confirmLabel={t(m.deleteSoundConfirm)}
  destructive
  onConfirm={runDelete}
  {fallbackFocus}
/>
