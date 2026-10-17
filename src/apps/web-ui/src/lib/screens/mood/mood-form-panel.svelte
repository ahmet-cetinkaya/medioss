<!-- @component The mood form: score, optional note, save and (while editing) cancel. -->
<script lang="ts">
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import type { Failure } from "$lib/components/dialogs/form-dialog.svelte";
  import ScoreSelector, {
    type MoodScore,
  } from "$lib/components/mood/score-selector.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDateTime } from "$lib/i18n/formatting/date-time-format";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { moodMessages as m } from "./mood.messages";

  const NOTE_LIMIT = 10_000;

  let {
    score = $bindable(),
    note = $bindable(),
    editing,
    saving,
    saveFailure,
    onSubmit,
    onCancel,
  }: {
    score: MoodScore;
    note: string;
    editing: boolean;
    saving: boolean;
    saveFailure: Failure | null;
    onSubmit: () => void;
    onCancel: () => void;
  } = $props();

  let selector = $state<ScoreSelector | undefined>();

  export function focus() {
    selector?.focus();
  }

  const todayHeading = $derived(
    formatDateTime(new Date().toISOString(), locale.current, {
      weekday: "long",
      day: "numeric",
      month: "long",
    }),
  );
</script>

<Panel aria-labelledby="mood-form-heading" class="mx-auto w-full max-w-2xl">
  {#snippet heading()}
    <div class="flex flex-col gap-1 text-center">
      <p class="text-label text-ink-muted">{todayHeading}</p>
      <h2 id="mood-form-heading" class="text-heading text-ink-strong">
        {t(editing ? m.editHeading : m.formHeading)}
      </h2>
    </div>
  {/snippet}
  <form
    class="flex flex-col gap-5"
    onsubmit={(event) => {
      event.preventDefault();
      onSubmit();
    }}
  >
    <ScoreSelector
      bind:this={selector}
      bind:value={score}
      name="mood-score"
      legend={t(m.scoreLegend)}
      legendHidden
      size="large"
      disabled={saving}
    />
    <div class="flex flex-col gap-2">
      <label for="mood-note" class="text-sm font-medium text-ink-body">
        {t(m.noteLabel)}
      </label>
      <Textarea
        id="mood-note"
        bind:value={note}
        maxlength={NOTE_LIMIT}
        rows={3}
        class="rounded-panel"
        aria-describedby="mood-note-hint"
        disabled={saving}
      />
      <p id="mood-note-hint" class="text-sm text-ink-muted">
        {t(m.noteHint, { limit: formatNumber(NOTE_LIMIT, locale.current) })}
      </p>
    </div>
    {#if saveFailure}
      <InlineMessage tone="error" role="alert">
        {t(saveFailure.message, saveFailure.params)}
      </InlineMessage>
    {/if}
    <div class="flex flex-wrap justify-center gap-3">
      <Button
        type="submit"
        size="lg"
        class="min-w-48"
        busy={saving}
        disabled={note.length > NOTE_LIMIT}
      >
        {t(editing ? m.saveChanges : m.saveMood)}
      </Button>
      {#if editing}
        <Button
          variant="secondary"
          size="lg"
          disabled={saving}
          onclick={onCancel}
        >
          {t(actionMessages.cancel)}
        </Button>
      {/if}
    </div>
  </form>
</Panel>
