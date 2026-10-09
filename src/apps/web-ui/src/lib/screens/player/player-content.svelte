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
  import { ApiError } from "$lib/api/api-error";
  import {
    fetchMeditation,
    fetchPlaybackState,
    fetchProgramMembers,
    fetchPrograms,
    type Meditation,
    type PlaybackState,
    type Program,
  } from "$lib/api/meditation";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { t } from "$lib/i18n/locale.svelte";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
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
  let program = $state<Program | null | undefined>(undefined);
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
    failure === "network" || failure === "server"
      ? t(serverReasonMessages[failure])
      : t(m.loadFailed),
  );

  onMount(() => {
    const parsed = parseMeditationId(location.search, location.pathname);
    // Guard against Back during hydration: if the user already left the
    // player before this island mounted, do not yank them back to Library.
    if (parsed === null) {
      if (window.location.pathname === ROUTES.player.path)
        location.replace(ROUTES.library.path);
    } else id = parsed;
    return () => {
      playerTitle.name = undefined;
      mix.dispose();
    };
  });

  // `GET /meditation/{id}` names no program, so look the meditation up in
  // the program member lists; the first program that contains it wins.
  async function findProgram(meditationId: number): Promise<Program | null> {
    const programs = await fetchPrograms();
    const members = await Promise.all(
      programs.map((item) => fetchProgramMembers(item.id)),
    );
    const index = members.findIndex((list) =>
      list.some((member) => member.id === meditationId),
    );
    return index === -1 ? null : programs[index];
  }

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
      const found = await findProgram(meditationId).catch(() => null);
      if (current === request) program = found;
    } catch (error) {
      if (current === request)
        failure =
          error instanceof ApiError ? serverReason(error).reason : "generic";
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
