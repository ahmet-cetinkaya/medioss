<!--
  @component
  Program content (FR-012, FR-038–FR-042, PB-18, PB-19). The id is parsed on
  mount before any request: missing or invalid → Library (replace). The hero
  is the screen's single featured element: cover (or generated artwork),
  title, description, overall progress and the one primary action. It and
  the session list render only once loaded, so no placeholder progress shows.
  A 404 shows not found with a Library link, never Login.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import {
    fetchProgram,
    fetchProgramMembers,
    programCoverUrl,
    type Meditation,
    type Program,
  } from "$lib/api/meditation";
  import { type ServerReason } from "$lib/api/server-reason";
  import {
    failureMessage,
    NETWORK_OR_SERVER_REASONS,
    reasonOf,
  } from "$lib/api/failure";
  import GeneratedArtwork from "$lib/components/content/generated-artwork.svelte";
  import { contentMessages } from "$lib/components/content/content.messages";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { IconButton } from "$lib/components/ui/icon-button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatDuration } from "$lib/i18n/formatting/duration-format";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import {
    meditationProgressLabel,
    programProgressLabel,
  } from "$lib/meditation/progress-label";
  import { ROUTES, playerHref } from "$lib/navigation/routes";
  import { parseProgramId } from "./program-id";
  import { programPrimaryAction } from "./program-primary-action";
  import { programMessages as m } from "./program.messages";

  type Loaded = { program: Program; members: Meditation[] };

  let programId = 0;
  let data = $state<Loaded | null>(null);
  let failure = $state<ServerReason | null>(null);
  let failedCover = $state(false);
  let request = 0;

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

  const action = $derived(data ? programPrimaryAction(data.members) : null);
  const actionLabel = $derived(
    action?.kind === "continue"
      ? m.continue
      : action?.kind === "playAgain"
        ? m.playAgain
        : m.start,
  );

  async function load(id: number) {
    const current = ++request;
    data = null;
    failure = null;
    failedCover = false;
    try {
      const [program, members] = await Promise.all([
        fetchProgram(id),
        fetchProgramMembers(id),
      ]);
      if (current === request) data = { program, members };
    } catch (error) {
      if (current === request) failure = reasonOf(error);
    }
  }

  onMount(() => {
    const id = parseProgramId(location.search);
    if (id === null) {
      // Guard against Back during hydration: if the user already left the
      // program page before this island mounted, do not redirect them.
      if (window.location.pathname === ROUTES.program.path)
        location.replace(ROUTES.library.path);
      return;
    }
    programId = id;
    void load(id);
  });
</script>

<RegionState
  {status}
  minHeight="24rem"
  {errorMessage}
  onRetry={() => void load(programId)}
  headingLevel={2}
  emptyTitle={t(m.notFoundTitle)}
  emptyText={t(m.notFoundText)}
>
  {#snippet emptyAction()}
    <Button variant="secondary" href={ROUTES.library.path}>
      {t(actionMessages.backToLibrary)}
    </Button>
  {/snippet}
  {#if data && action}
    {@const { program, members } = data}
    <div class="flex flex-col gap-6">
      <section
        aria-labelledby="program-title"
        data-elevation="featured"
        class="elevation-3 flex min-w-0 flex-wrap items-center gap-3 rounded-tile border border-rim bg-surface-solid/70 p-3"
      >
        <span
          class="relative block size-16 shrink-0 overflow-hidden rounded-thumb sm:size-20"
          aria-hidden="true"
        >
          {#if program.has_cover && !failedCover}
            <img
              src={programCoverUrl(program.id)}
              alt=""
              decoding="async"
              class="absolute inset-0 size-full object-cover"
              onerror={() => (failedCover = true)}
            />
          {:else}
            <GeneratedArtwork seed={program.id} category="meditation" />
          {/if}
        </span>
        <div class="flex min-w-0 flex-1 flex-col gap-1">
          <h2
            id="program-title"
            class="text-heading break-words text-ink-strong"
          >
            {program.title}
          </h2>
          {#if program.description}
            <p
              class="text-body-md line-clamp-2 break-words whitespace-pre-line text-ink-body"
            >
              {program.description}
            </p>
          {/if}
          <p class="text-meta text-ink-muted">
            {programProgressLabel(program.progress, locale.current)}
          </p>
          {#if action.kind === "unavailable"}
            <p id="program-action-reason" class="text-sm text-ink-body">
              {t(m.unavailable)}
            </p>
          {/if}
        </div>
        <div class="self-center">
          {#if action.kind === "unavailable"}
            <Button disabled aria-describedby="program-action-reason">
              {t(actionLabel)}
            </Button>
          {:else}
            <Button href={playerHref(action.meditationId)}>
              {t(actionLabel)}
            </Button>
          {/if}
        </div>
      </section>

      <Panel aria-labelledby="program-sessions-heading">
        {#snippet heading()}
          <h2
            id="program-sessions-heading"
            class="text-heading text-ink-strong"
          >
            {t(m.sessionsHeading)}
          </h2>
        {/snippet}
        <RegionState
          status={members.length ? "ready" : "empty"}
          minHeight="8rem"
          emptyTitle={t(m.emptyTitle)}
          emptyText={t(m.emptyText)}
        >
          <ol class="flex flex-col gap-3">
            {#each members as meditation (meditation.id)}
              <li
                class="flex w-full min-w-0 items-center gap-2 rounded-thumb border border-rim bg-surface-solid/70 p-3"
              >
                <div class="flex min-w-0 flex-1 flex-col gap-1">
                  <h3 class="text-body-md break-words text-ink-strong">
                    {meditation.name}
                  </h3>
                  <p class="text-meta text-ink-muted">
                    {[
                      formatDuration(
                        meditation.duration_ms,
                        locale.current,
                        "short",
                      ),
                      meditationProgressLabel(meditation, locale.current),
                    ].join(" • ")}
                  </p>
                  {#if meditation.description}
                    <p
                      class="line-clamp-3 text-sm break-words whitespace-pre-line text-ink-body"
                    >
                      {meditation.description}
                    </p>
                  {/if}
                </div>
                <IconButton
                  label={t(contentMessages.play, { title: meditation.name })}
                  icon="play"
                  filled
                  href={playerHref(meditation.id)}
                  class="shrink-0 text-accent-strong"
                />
              </li>
            {/each}
          </ol>
        </RegionState>
      </Panel>
    </div>
  {/if}
</RegionState>
