<!--
  @component
  Player content (FR-012, FR-043–FR-048, PB-1–PB-5, PB-16, PB-18). Reads
  `?id=`: an invalid id replaces the address with Library (PB-18); a 404
  shows not found with a Library link (FR-012). After load it lifts the
  meditation name to the shell (h1 and document title, FR-043). The
  playback panel and the notes panel load and fail independently.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import {
    fetchMeditation,
    fetchPlaybackState,
    type Meditation,
    type PlaybackState,
    type ProgramSummary,
  } from "$lib/api/meditation";
  import { type ServerReason } from "$lib/api/server-reason";
  import {
    failureMessage,
    NETWORK_OR_SERVER_REASONS,
    reasonOf,
  } from "$lib/api/failure";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { ROUTES } from "$lib/navigation/routes";
  import { PlayerMix } from "./player-mix.svelte";
  import PlayerCompletionDialog from "./player-completion-dialog.svelte";
  import { parseMeditationId } from "./meditation-id";
  import PlayerControls from "./player-controls.svelte";
  import PlayerNotes from "./player-notes.svelte";
  import { playerTitle } from "./player-title.svelte";
  import { playerMessages as m } from "./player.messages";

  let id = $state<number | null>(null);
  let data = $state<{
    meditation: Meditation;
    playback: PlaybackState;
  } | null>(null);
  // undefined while looking up, null when the meditation is in no program.
  let program = $state<ProgramSummary | null | undefined>(undefined);
  let failure = $state<ServerReason | null>(null);
  let request = 0;
  // One background mix (one MixEngine) per screen mount.
  const mix = new PlayerMix();
  let doneOpen = $state(false);
  let notesVersion = $state(0);

  const status = $derived(
    failure === "notFound"
      ? "empty"
      : failure
        ? "error"
        : data
          ? "ready"
          : "loading",
  );

  const errorMessage = $derived(
    t(
      failureMessage(failure, m.loadFailed, {
        specificReasons: NETWORK_OR_SERVER_REASONS,
      }).message,
    ),
  );

  onMount(() => {
    const parsed = parseMeditationId(location.search, location.pathname);
    // Guard against Back during hydration: if the user already left the
    // player before this island mounted, do not yank them back to Library.
    if (parsed === null) {
      if (window.location.pathname.startsWith(ROUTES.player.path))
        location.replace(ROUTES.library.path);
    } else id = parsed;
    return () => {
      playerTitle.name = undefined;
      mix.dispose();
    };
  });

  // The detail response includes the home-program summary, avoiding a fan-out
  // across every program and membership list on each Player load.
  async function load(meditationId: number) {
    const current = ++request;
    data = null;
    failure = null;
    program = undefined;
    try {
      const [meditation, playback] = await Promise.all([
        fetchMeditation(meditationId),
        fetchPlaybackState(meditationId),
      ]);
      if (current !== request) return;
      data = { meditation, playback };
      playerTitle.name = meditation.name;
      program = meditation.program ?? null;
    } catch (error) {
      if (current === request) failure = reasonOf(error);
    }
  }

  $effect(() => {
    if (id !== null) void load(id);
  });
</script>

<RegionState
  {status}
  minHeight="28rem"
  {errorMessage}
  onRetry={id === null ? undefined : () => void load(id!)}
  headingLevel={2}
  emptyTitle={t(m.notFoundTitle)}
  emptyText={t(m.notFoundText)}
>
  {#snippet emptyAction()}
    <Button variant="secondary" href={ROUTES.library.path}>
      {t(actionMessages.backToLibrary)}
    </Button>
  {/snippet}
  {#if data && id !== null}
    <div
      class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,24rem)] lg:items-start"
    >
      <PlayerControls
        meditation={data.meditation}
        playback={data.playback}
        {program}
        {mix}
        onCompleted={() => (doneOpen = true)}
        holdFocus={doneOpen}
      />
      <PlayerNotes meditationId={id} refresh={notesVersion} />
    </div>
  {/if}
</RegionState>

{#if id !== null}
  <PlayerCompletionDialog
    bind:open={doneOpen}
    meditationId={id}
    programId={program === undefined ? undefined : (program?.id ?? null)}
    onSaved={() => notesVersion++}
  />
{/if}
