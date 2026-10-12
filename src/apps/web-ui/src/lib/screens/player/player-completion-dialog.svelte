<!--
  @component
  Opens when the meditation ends and its completion records (PB-3): a warm
  congratulations dialog, nothing required. The note is optional: the newest
  existing note is prefilled, and whatever text stands in the field when the
  dialog closes (Continue, X, Escape, or an outside click) is saved —
  created when no note existed, updated otherwise. An empty field saves
  nothing; an emptied prefill changes nothing. Continue then navigates back
  to the program (or Library when the meditation is in no program), after
  the note is safely on its way.
-->
<script lang="ts">
  import {
    createNote,
    fetchNotes,
    updateNote,
    type Note,
  } from "$lib/api/meditation";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import GeneratedArtwork from "$lib/components/content/generated-artwork.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { t } from "$lib/i18n/translate";
  import { ROUTES } from "$lib/navigation/routes";
  import { playerMessages as m } from "./player.messages";

  type Props = {
    open?: boolean;
    meditationId: number;
    /** null when the meditation is in no program; undefined while looking up. */
    programId: number | null | undefined;
    onSaved: () => void;
  };

  let {
    open = $bindable(false),
    meditationId,
    programId,
    onSaved,
  }: Props = $props();

  let draft = $state("");
  let existing = $state<Note | null>(null);
  let saveDone = false;

  $effect(() => {
    if (!open) return;
    draft = "";
    existing = null;
    saveDone = false;
    let current = true;
    fetchNotes(meditationId)
      .then((notes) => {
        if (!current || notes.length === 0) return;
        const newest = notes.reduce((a, b) =>
          Date.parse(b.created_at) > Date.parse(a.created_at) ||
          (b.created_at === a.created_at && b.id > a.id)
            ? b
            : a,
        );
        existing = newest;
        if (!draft) draft = newest.text;
      })
      .catch(() => {});
    return () => (current = false);
  });

  // Closing is the save, exactly once per dialog session: whatever stands in
  // the field goes to the server — created when no note existed, the
  // prefilled one updated otherwise. Empty saves nothing; an emptied prefill
  // keeps the stored note as it was. A failed save never blocks closing; the
  // note can still be edited in the notes panel.
  async function saveOnClose(): Promise<void> {
    if (saveDone) return;
    saveDone = true;
    const text = draft.trim();
    if (!text || (existing && text === existing.text)) return;
    try {
      if (existing) await updateNote(meditationId, existing.id, text);
      else await createNote(meditationId, text);
      announce(t(m.doneSaved));
      onSaved();
    } catch {
      // Swallowed on purpose: closing is never blocked on the note.
    }
  }

  // Every close path (X, Escape, outside click, unmount) runs through this
  // effect's cleanup, which fires once per open→closed transition.
  let wasOpen = false;
  $effect(() => {
    const opened = open;
    if (opened) wasOpen = true;
    return () => {
      if (opened && wasOpen) {
        wasOpen = false;
        void saveOnClose();
      }
    };
  });

  // Continue saves first (so navigation cannot cancel the request), then
  // leaves for the program page — or Library when there is no program.
  async function continueToProgram() {
    await saveOnClose();
    open = false;
    location.assign(
      programId
        ? `${ROUTES.program.path}?id=${programId}`
        : ROUTES.library.path,
    );
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content>
    <div class="flex flex-col items-center gap-2 text-center">
      <span
        aria-hidden="true"
        class="relative block size-24 overflow-hidden rounded-tile motion-safe:animate-breathe"
      >
        <GeneratedArtwork seed={meditationId} category="meditation" wash />
      </span>
      <Dialog.Title class="text-greeting text-ink-strong">
        {t(m.doneTitle)}
      </Dialog.Title>
      <Dialog.Description>{t(m.doneText)}</Dialog.Description>
    </div>

    <label for="player-done-note" class="text-sm font-medium text-ink-body">
      {t(m.doneNoteLabel)}
    </label>
    <Textarea
      id="player-done-note"
      bind:value={draft}
      placeholder={t(m.notePlaceholder)}
    />

    <Dialog.Footer>
      <Button onclick={() => void continueToProgram()}>
        {t(programId ? m.doneContinue : m.doneContinueLibrary)}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
