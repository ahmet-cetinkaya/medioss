<!--
  @component
  Quick mood check-in dialog (FR-108–FR-111), opened from the check-in card.
  The score starts at 3; the note is optional and limited to 10 000
  characters: over the limit the counter says so and Save is disabled, the
  text is kept (no `maxlength`, so pasted text is never cut). On save:
  announce, reset, close, then `onSaved`. On failure the dialog stays open
  with score and note kept and the reason shown (FormDialog).

  Props: `open` (bindable), `onSaved`.
-->
<script lang="ts">
  import FormDialog from "$lib/components/dialogs/form-dialog.svelte";
  import { createMood } from "$lib/api/mood";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import ScoreSelector, {
    type MoodScore,
  } from "$lib/components/mood/score-selector.svelte";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDateTime } from "$lib/i18n/formatting/date-time-format";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { homeMessages as m } from "./home.messages";
  import { NOTE_LIMIT, noteLength, noteOverLimit } from "./note-limit";

  type Props = { open?: boolean; onSaved: () => void };

  let { open = $bindable(false), onSaved }: Props = $props();

  let score = $state<MoodScore>(3);
  let note = $state("");

  const today = $derived(
    formatDateTime(new Date().toISOString(), locale.current, {
      weekday: "long",
      day: "numeric",
      month: "long",
    }),
  );
  const over = $derived(noteOverLimit(note));
  const count = (value: number) => formatNumber(value, locale.current);

  async function save() {
    await createMood(score, note.trim() ? note : undefined);
    score = 3;
    note = "";
    announce(t(m.saved));
    onSaved();
  }
</script>

<FormDialog
  bind:open
  title={t(m.checkInHeading)}
  submitLabel={t(actionMessages.save)}
  onSubmit={save}
  canSubmit={over === 0}
>
  <ScoreSelector
    bind:value={score}
    name="home-mood-score"
    legend={t(m.scoreLegend)}
    legendHidden
    size="large"
  />
  <p class="text-center text-label text-ink-muted">{today}</p>
  <div class="flex flex-col gap-2">
    <label for="home-mood-note" class="text-sm font-medium text-ink-body">
      {t(m.noteLabel)}
    </label>
    <Textarea
      id="home-mood-note"
      bind:value={note}
      rows={4}
      class="min-h-28 rounded-panel"
      aria-invalid={over > 0 || undefined}
      aria-describedby="home-mood-note-count"
    />
    <div id="home-mood-note-count" aria-live="polite">
      {#if over > 0}
        <InlineMessage tone="error">
          {t(m.noteTooLong, { over: count(over), limit: count(NOTE_LIMIT) })}
        </InlineMessage>
      {:else}
        <p class="text-meta text-ink-muted">
          {t(m.noteCount, {
            count: count(noteLength(note)),
            limit: count(NOTE_LIMIT),
          })}
        </p>
      {/if}
    </div>
  </div>
</FormDialog>
