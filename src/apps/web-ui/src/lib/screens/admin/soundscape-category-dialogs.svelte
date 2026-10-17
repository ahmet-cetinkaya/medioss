<!-- @component Add, rename and delete sound-category dialogs; open them through the exported `ask*` functions. -->
<script lang="ts">
  import {
    createSoundscapeCategory,
    deleteSoundscapeCategory,
    renameSoundscapeCategory,
    type SoundscapeCategory,
  } from "$lib/api/soundscape";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import RenameDialog from "$lib/components/dialogs/rename-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { t } from "$lib/i18n/translate";
  import { soundAdminMessages as m } from "./admin.messages";

  const CATEGORY_NAME_MAX_LENGTH = 80;

  let {
    id,
    onChanged,
    onAdded,
    fallbackFocus,
  }: {
    id: string;
    onChanged: () => Promise<void>;
    /** The new category, so the screen can select it. */
    onAdded: (category: SoundscapeCategory) => void;
    fallbackFocus: () => HTMLElement | null;
  } = $props();

  let addOpen = $state(false);
  let newCategory = $state("");

  export function askAdd() {
    newCategory = "";
    addOpen = true;
  }

  async function runAdd() {
    const category = await createSoundscapeCategory(newCategory.trim());
    announce(t(m.categoryAdded, { name: category.name }));
    onAdded(category);
    await onChanged();
  }

  let target = $state<SoundscapeCategory | null>(null);
  let renameOpen = $state(false);
  let deleteOpen = $state(false);

  export function askRename(category: SoundscapeCategory) {
    target = category;
    renameOpen = true;
  }

  async function runRename(name: string) {
    if (!target) return;
    await renameSoundscapeCategory(target.id, name);
    await onChanged();
  }

  export function askDelete(category: SoundscapeCategory) {
    target = category;
    deleteOpen = true;
  }

  async function runDelete() {
    const category = target;
    if (!category) return;
    await deleteSoundscapeCategory(category.id);
    announce(t(m.categoryDeleted, { name: category.name }));
    await onChanged();
  }
</script>

<FormDialog
  bind:open={addOpen}
  title={t(m.addCategoryTitle)}
  submitLabel={t(m.addCategory)}
  canSubmit={newCategory.trim() !== ""}
  onSubmit={runAdd}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-new-category">{t(m.categoryName)}</Label>
    <Input
      id="{id}-new-category"
      bind:value={newCategory}
      maxlength={CATEGORY_NAME_MAX_LENGTH}
      autocomplete="off"
      required
    />
  </div>
</FormDialog>

<RenameDialog
  bind:open={renameOpen}
  title={t(m.renameCategoryTitle)}
  currentName={target?.name ?? ""}
  onSave={runRename}
  maxlength={CATEGORY_NAME_MAX_LENGTH}
/>

<ConfirmDialog
  bind:open={deleteOpen}
  title={t(m.deleteCategoryTitle)}
  itemName={target?.name}
  consequence={t(m.deleteCategoryConsequence)}
  confirmLabel={t(m.deleteCategoryConfirm)}
  destructive
  onConfirm={runDelete}
  {fallbackFocus}
/>
