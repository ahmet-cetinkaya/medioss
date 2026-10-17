<!-- @component Edit-description and remove-from-program dialogs for a program's meditations. -->
<script lang="ts">
  import {
    removeMember,
    updateMeditationDescription,
    type Meditation,
  } from "$lib/api/meditation";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { meditationAdminMessages as m } from "./admin.messages";
  import DescriptionCounter from "./description-counter.svelte";
  import { DESCRIPTION_MAX, descriptionLength } from "./meditations-logic";
  import { say } from "./meditations-say";

  let {
    id,
    selectedId,
    onChanged,
    fallbackFocus,
  }: {
    id: string;
    selectedId: number | null;
    onChanged: () => Promise<void>;
    fallbackFocus: () => HTMLElement | null;
  } = $props();

  let target = $state<Meditation | null>(null);

  let editOpen = $state(false);
  let editDescription = $state("");
  const editTooLong = $derived(
    descriptionLength(editDescription.trim()) > DESCRIPTION_MAX,
  );

  export function askEditDescription(meditation: Meditation) {
    target = meditation;
    editDescription = meditation.description ?? "";
    editOpen = true;
  }

  async function runEdit() {
    if (!target || editTooLong) return;
    const updated = await updateMeditationDescription(
      target.id,
      editDescription.trim() || null,
    );
    say(m.descriptionUpdated, { name: updated.name });
    await onChanged();
  }

  let removeOpen = $state(false);

  export function askRemove(meditation: Meditation) {
    target = meditation;
    removeOpen = true;
  }

  async function runRemove() {
    const removing = target;
    if (!removing || selectedId === null) return;
    await removeMember(selectedId, removing.id);
    say(m.removed, { name: removing.name });
    await onChanged();
  }
</script>

<FormDialog
  bind:open={editOpen}
  title={t(m.editDescriptionTitle)}
  description={target?.name}
  submitLabel={t(actionMessages.save)}
  canSubmit={!editTooLong}
  onSubmit={runEdit}
>
  <div class="grid grid-cols-1 gap-2">
    <Label for="{id}-edit-description">{t(m.description)}</Label>
    <Textarea
      id="{id}-edit-description"
      bind:value={editDescription}
      rows={4}
      aria-invalid={editTooLong}
      aria-describedby="{id}-edit-description-count"
    />
    <DescriptionCounter
      counterId="{id}-edit-description-count"
      text={editDescription}
      over={editTooLong}
    />
    <p class="text-sm text-ink-muted">{t(m.editDescriptionHint)}</p>
  </div>
</FormDialog>

<ConfirmDialog
  bind:open={removeOpen}
  title={t(m.removeTitle)}
  itemName={target?.name}
  consequence={t(m.removeConsequence)}
  confirmLabel={t(m.removeConfirm)}
  destructive
  onConfirm={runRemove}
  {fallbackFocus}
/>
