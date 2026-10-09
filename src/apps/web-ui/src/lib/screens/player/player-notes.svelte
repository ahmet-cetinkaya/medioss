<!--
  @component
  Meditation notes (FR-046, FR-047): feed items newest first with their
  local date and time, a pill input whose Add is unavailable while blank,
  and delete through the shared ConfirmDialog (Cancel keeps the note).
  Loads on its own, so a playback failure never blocks it (FR-048).
-->
<script lang="ts">
  import { ApiError } from "$lib/api/api-error";
  import {
    createNote,
    deleteNote,
    fetchNotes,
    type Note,
  } from "$lib/api/meditation";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import FeedItem from "$lib/components/content/feed-item.svelte";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { formatDateTime } from "$lib/i18n/formatting/date-time-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import { playerMessages as m } from "./player.messages";

  type Props = { meditationId: number; refresh?: number };

  let { meditationId, refresh = 0 }: Props = $props();

  let notes = $state<Note[] | null>(null);
  let failure = $state<ServerReason | null>(null);
  let draft = $state("");
  let adding = $state(false);
  let addFailure = $state<ServerReason | null>(null);
  let pendingDelete = $state<Note | null>(null);
  let confirmOpen = $state(false);
  let input = $state<HTMLTextAreaElement | null>(null);
  let request = 0;

  const newestFirst = (list: Note[]) =>
    [...list].sort(
      (a, b) =>
        Date.parse(b.created_at) - Date.parse(a.created_at) || b.id - a.id,
    );

  const status = $derived(
    failure ? "error" : notes ? (notes.length ? "ready" : "empty") : "loading",
  );

  const reasonText = (
    reason: ServerReason,
    fallback: typeof m.noteAddFailed,
  ) =>
    reason === "network" || reason === "server"
      ? t(serverReasonMessages[reason])
      : t(fallback);

  async function load(id: number) {
    const current = ++request;
    notes = null;
    failure = null;
    try {
      const result = await fetchNotes(id);
      if (current === request) notes = newestFirst(result);
    } catch (error) {
      if (current === request)
        failure =
          error instanceof ApiError ? serverReason(error).reason : "generic";
    }
  }

  $effect(() => {
    void refresh;
    void load(meditationId);
  });

  async function addNote(event: SubmitEvent) {
    event.preventDefault();
    const text = draft.trim();
    if (!text || adding) return;
    adding = true;
    addFailure = null;
    try {
      const created = await createNote(meditationId, text);
      notes = newestFirst([created, ...(notes ?? [])]);
      draft = "";
      announce(t(m.noteAdded));
    } catch (error) {
      addFailure =
        error instanceof ApiError ? serverReason(error).reason : "generic";
    } finally {
      adding = false;
    }
  }

  function askDelete(note: Note) {
    pendingDelete = note;
    confirmOpen = true;
  }

  async function runDelete() {
    const note = pendingDelete;
    if (!note) return;
    await deleteNote(meditationId, note.id);
    notes = (notes ?? []).filter((item) => item.id !== note.id);
    announce(t(m.noteDeleted));
  }
</script>

<Panel aria-labelledby="player-notes-heading">
  {#snippet heading()}
    <h2 id="player-notes-heading" class="text-heading text-ink-strong">
      {t(m.notesHeading)}
    </h2>
    <p class="mt-1 text-sm text-ink-body">{t(m.notesHint)}</p>
  {/snippet}

  <form class="mb-4 flex flex-col gap-3" onsubmit={addNote}>
    <label for="player-note" class="text-sm font-medium text-ink-body">
      {t(m.noteLabel)}
    </label>
    <Textarea
      id="player-note"
      bind:ref={input}
      bind:value={draft}
      placeholder={t(m.notePlaceholder)}
    />
    {#if addFailure}
      <InlineMessage tone="error" role="alert">
        {reasonText(addFailure, m.noteAddFailed)}
      </InlineMessage>
    {/if}
    <Button
      type="submit"
      variant="primary"
      class="self-start"
      busy={adding}
      disabled={!draft.trim()}
    >
      {t(m.addNote)}
    </Button>
  </form>

  <RegionState
    {status}
    minHeight="8rem"
    errorMessage={failure ? reasonText(failure, m.notesLoadFailed) : undefined}
    onRetry={() => void load(meditationId)}
    emptyTitle={t(m.notesEmptyTitle)}
    emptyText={t(m.notesEmptyText)}
  >
    <ul class="flex flex-col">
      {#each notes ?? [] as note (note.id)}
        <FeedItem
          as="li"
          source={t(m.noteSource)}
          body={note.text}
          meta={formatDateTime(note.created_at, locale.current)}
          align="end"
        >
          {#snippet actions()}
            <Button variant="ghost" size="sm" onclick={() => askDelete(note)}>
              <AppIcon name="trash" size={18} />
              {t(m.deleteNote)}
            </Button>
          {/snippet}
        </FeedItem>
      {/each}
    </ul>
  </RegionState>
</Panel>

<ConfirmDialog
  bind:open={confirmOpen}
  title={t(m.deleteNoteTitle)}
  itemName={pendingDelete?.text}
  consequence={t(m.deleteNoteConsequence)}
  confirmLabel={t(m.deleteNote)}
  destructive
  onConfirm={runDelete}
  fallbackFocus={() => input}
/>
