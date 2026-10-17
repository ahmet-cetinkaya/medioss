<!-- @component Add and edit program dialogs; open them through the exported `ask*` functions. -->
<script lang="ts">
  import {
    assignCategory,
    createProgram,
    replaceProgramCover,
    updateProgram,
    type Category,
    type Program,
  } from "$lib/api/meditation";
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { meditationAdminMessages as m } from "./admin.messages";
  import { say } from "./meditations-say";
  import NativeSelect from "./native-select.svelte";

  const COVER_ACCEPT = "image/png,image/jpeg,image/webp";

  let {
    id,
    categories,
    onChanged,
    onCoverReplaced,
  }: {
    id: string;
    categories: Category[];
    /** Reload after a change; `programId` names the program that changed. */
    onChanged: (programId?: number) => Promise<void>;
    onCoverReplaced: (programId: number) => void;
  } = $props();

  let addOpen = $state(false);
  let addTitle = $state("");
  let addDescription = $state("");
  let addCover = $state<File | null>(null);

  export function askAdd() {
    addTitle = "";
    addDescription = "";
    addCover = null;
    addOpen = true;
  }

  async function runAdd() {
    const program = await createProgram(
      addTitle.trim(),
      addDescription.trim() || null,
      addCover,
    );
    say(m.programCreated, { title: program.title });
    await onChanged();
  }

  // Title, description, category, cover.
  let editOpen = $state(false);
  let editTarget = $state<Program | null>(null);
  let editTitle = $state("");
  let editDescription = $state("");
  let editCategory = $state("");
  let editCover = $state<File | null>(null);

  export function askEdit(program: Program) {
    editTarget = program;
    editTitle = program.title;
    editDescription = program.description ?? "";
    editCategory =
      program.category_id === null ? "" : String(program.category_id);
    editCover = null;
    editOpen = true;
  }

  async function runEdit() {
    const program = editTarget;
    if (!program || editTitle.trim() === "") return;
    await updateProgram(
      program.id,
      editTitle.trim(),
      editDescription.trim() || null,
    );
    const categoryId = editCategory === "" ? null : Number(editCategory);
    if (categoryId !== program.category_id)
      await assignCategory(program.id, categoryId);
    if (editCover) {
      await replaceProgramCover(program.id, editCover);
      onCoverReplaced(program.id);
    }
    say(m.programSaved, { title: editTitle.trim() });
    await onChanged(program.id);
  }
</script>

<FormDialog
  bind:open={addOpen}
  title={t(m.addProgramTitle)}
  submitLabel={t(m.createProgram)}
  canSubmit={addTitle.trim() !== ""}
  onSubmit={runAdd}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-title">{t(m.title)}</Label>
    <Input id="{id}-title" bind:value={addTitle} required />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-program-description">{t(m.description)}</Label>
    <Textarea id="{id}-program-description" bind:value={addDescription} />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-cover">{t(m.cover)}</Label>
    <Input
      id="{id}-cover"
      type="file"
      accept={COVER_ACCEPT}
      aria-describedby="{id}-cover-hint"
      onchange={(event) => (addCover = event.currentTarget.files?.[0] ?? null)}
    />
    <p id="{id}-cover-hint" class="text-sm text-ink-muted">{t(m.coverHint)}</p>
  </div>
</FormDialog>

<FormDialog
  bind:open={editOpen}
  title={t(m.editProgramTitle)}
  description={editTarget?.title}
  submitLabel={t(actionMessages.save)}
  canSubmit={editTitle.trim() !== ""}
  onSubmit={runEdit}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-title">{t(m.title)}</Label>
    <Input id="{id}-edit-title" bind:value={editTitle} required />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-program-description">{t(m.description)}</Label>
    <Textarea
      id="{id}-edit-program-description"
      bind:value={editDescription}
      rows={3}
    />
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-category">{t(m.categoryHeading)}</Label>
    <NativeSelect id="{id}-edit-category" bind:value={editCategory}>
      <option value="">{t(m.uncategorized)}</option>
      {#each categories as category (category.id)}
        <option value={String(category.id)}>{category.name}</option>
      {/each}
    </NativeSelect>
  </div>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-cover">{t(m.cover)}</Label>
    <Input
      id="{id}-edit-cover"
      type="file"
      accept={COVER_ACCEPT}
      aria-describedby="{id}-edit-cover-hint"
      onchange={(event) => (editCover = event.currentTarget.files?.[0] ?? null)}
    />
    <p id="{id}-edit-cover-hint" class="text-sm text-ink-muted">
      {t(m.editProgramHint)}
    </p>
  </div>
</FormDialog>
