<!--
  @component
  Sleep-timer settings (PB-7), built on FormDialog: whole minutes from 5 to
  120, Start and Cancel. Start stays disabled while the value is invalid and
  the reason shows as text. `onStart` receives the validated minutes.
-->
<script lang="ts">
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { t } from "$lib/i18n/translate";
  import { isValidTimerMinutes } from "./sleep-timer";
  import { soundscapeMessages as m } from "./soundscape.messages";

  type Props = {
    open?: boolean;
    onStart: (minutes: number) => void;
  };

  let { open = $bindable(false), onStart }: Props = $props();

  const id = $props.id();
  let minutes = $state<number | null>(30);
  const valid = $derived(isValidTimerMinutes(minutes));
</script>

<FormDialog
  bind:open
  title={t(m.timerTitle)}
  submitLabel={t(m.startTimer)}
  canSubmit={valid}
  onSubmit={async () => onStart(minutes as number)}
>
  <div class="grid gap-2">
    <Label for="{id}-minutes">{t(m.timerLabel)}</Label>
    <Input
      id="{id}-minutes"
      class="w-32"
      type="number"
      inputmode="numeric"
      min={5}
      max={120}
      step={1}
      bind:value={minutes}
      aria-invalid={!valid || undefined}
      aria-describedby="{id}-hint"
    />
    <p id="{id}-hint" class="text-sm text-ink-muted">{t(m.timerHint)}</p>
    {#if !valid}
      <InlineMessage tone="error" role="alert">
        {t(m.timerInvalid)}
      </InlineMessage>
    {/if}
  </div>
</FormDialog>
