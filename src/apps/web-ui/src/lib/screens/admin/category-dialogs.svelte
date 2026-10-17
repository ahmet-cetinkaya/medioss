<!-- @component Add, rename and delete category dialogs; open them through the exported `ask*` functions. -->
<script lang="ts">
  import {
    createCategory,
    deleteCategory,
    renameCategory,
    type Category,
  } from "$lib/api/meditation";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import RenameDialog from "$lib/components/dialogs/rename-dialog.svelte";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { t } from "$lib/i18n/locale.svelte";
  import { meditationAdminMessages as m } from "./admin.messages";
  import { say } from "./meditations-say";

  let {
    id,
    onChanged,
    onDeleting,
  }: {
    id: string;
    onChanged: () => Promise<void>;
    /** Runs before the server delete so the screen can drop a stale filter. */
    onDeleting: (category: Category) => void;
  } = $props();

  let addOpen = $state(false);
  let addName = $state("");

  export function askAdd() {
    addName = "";
    addOpen = true;
  }

  async function runAdd() {
    const category = await createCategory(addName.trim());
    say(m.categoryCreated, { name: category.name });
    await onChanged();
  }

  let renameOpen = $state(false);
  let renameTarget = $state<Category | null>(null);

  export function askRename(category: Category) {
    renameTarget = category;
    renameOpen = true;
  }

  async function runRename(name: string) {
    const category = renameTarget;
    if (!category) return;
    await renameCategory(category.id, name);
    say(m.categoryRenamed, { name });
    await onChanged();
  }

  // Its programs become uncategorized.
  let deleteOpen = $state(false);
  let deleteTarget = $state<Category | null>(null);

  export function askDelete(category: Category) {
    deleteTarget = category;
    deleteOpen = true;
  }

  async function runDelete() {
    const category = deleteTarget;
    if (!category) return;
    onDeleting(category);
    await deleteCategory(category.id);
    say(m.meditationCategoryDeleted, { name: category.name });
    deleteTarget = null;
    await onChanged();
  }
</script>

<FormDialog
  bind:open={addOpen}
  title={t(m.addCategoryTitle)}
  submitLabel={t(m.addCategory)}
  canSubmit={addName.trim() !== ""}
  onSubmit={runAdd}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-category">{t(m.categoryName)}</Label>
    <Input id="{id}-category" bind:value={addName} required />
  </div>
</FormDialog>

<RenameDialog
  bind:open={renameOpen}
  title={t(m.renameCategoryTitle)}
  currentName={renameTarget?.name ?? ""}
  label={t(m.categoryName)}
  onSave={runRename}
/>

<ConfirmDialog
  bind:open={deleteOpen}
  title={t(m.deleteCategoryDialogTitle)}
  itemName={deleteTarget?.name}
  consequence={t(m.deleteCategoryDialogText)}
  confirmLabel={t(m.deleteCategoryDialogConfirm)}
  destructive
  onConfirm={runDelete}
/>
