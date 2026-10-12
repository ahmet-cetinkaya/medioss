<!--
  @component
  Rename dialog (plan D6, FR-019), built on FormDialog. Each open prefills
  `currentName`; Save stays disabled while the trimmed name is empty, and
  `onSave` receives the trimmed name.

  Props: `open` (bindable), `title`, `currentName`, `onSave: (name) =>
  Promise<void>`, `label?` (field label, default "Name"), `submitLabel?`
  (default "Save"), `maxlength?`, and FormDialog's `returnFocusTo?`,
  `fallbackFocus?`, `reasonContext?`, `errorMessage?`. Busy and error
  handling are FormDialog's.
-->
<script lang="ts">
  import type { ServerReasonContext } from "$lib/api/server-reason";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/translate";
  import { dialogMessages } from "./dialog.messages";
  import FormDialog, { type ErrorMessage } from "./form-dialog.svelte";
  import type { FocusSource } from "./return-focus";

  type Props = {
    open?: boolean;
    title: string;
    currentName: string;
    onSave: (name: string) => Promise<void>;
    label?: string;
    submitLabel?: string;
    maxlength?: number;
    returnFocusTo?: HTMLElement | null;
    fallbackFocus?: FocusSource<HTMLElement>;
    reasonContext?: ServerReasonContext;
    errorMessage?: ErrorMessage;
  };

  let {
    open = $bindable(false),
    title,
    currentName,
    onSave,
    label,
    submitLabel,
    maxlength,
    ...rest
  }: Props = $props();

  const id = $props.id();
  let name = $state("");
  const trimmed = $derived(name.trim());

  $effect.pre(() => {
    if (open) name = currentName;
  });
</script>

<FormDialog
  bind:open
  {title}
  submitLabel={submitLabel ?? t(actionMessages.save)}
  canSubmit={trimmed !== ""}
  onSubmit={() => onSave(trimmed)}
  {...rest}
>
  <div class="grid gap-2">
    <Label for="{id}-name">{label ?? t(dialogMessages.name)}</Label>
    <Input
      id="{id}-name"
      bind:value={name}
      {maxlength}
      autocomplete="off"
      required
      aria-invalid={trimmed === "" || undefined}
      aria-describedby={trimmed === "" ? `${id}-hint` : undefined}
    />
    {#if trimmed === ""}
      <p id="{id}-hint" class="text-sm text-ink-muted">
        {t(dialogMessages.nameRequired)}
      </p>
    {/if}
  </div>
</FormDialog>
