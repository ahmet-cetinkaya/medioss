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
    type Meditation,
    type Program,
  } from "$lib/api/meditation";
  import { type ServerReason } from "$lib/api/server-reason";
  import {
    failureMessage,
    NETWORK_OR_SERVER_REASONS,
    reasonOf,
  } from "$lib/api/failure";
  import { contentMessages } from "$lib/components/content/content.messages";
  import MeditationRow from "$lib/components/content/meditation-row.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
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
  import ProgramHero from "./program-hero.svelte";
  import { programPrimaryAction } from "./program-primary-action";
  import { programMessages as m } from "./program.messages";

  type Loaded = { program: Program; members: Meditation[] };

  let programId = 0;
  let data = $state<Loaded | null>(null);
  let failure = $state<ServerReason | null>(null);
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
    <div
      class="grid min-w-0 gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,26rem)] lg:items-start"
    >
      <section
        aria-labelledby="program-title"
        class="glass-panel flex min-w-0 flex-col gap-4 rounded-panel p-4 sm:p-6 lg:col-start-2 lg:row-start-1"
      >
        <ProgramHero
          {program}
          progress={program.progress}
          progressLabel={programProgressLabel(
            program.progress,
            locale.current,
          ) ?? ""}
        />
        <div class="flex min-w-0 flex-col items-start gap-4">
          <div class="flex min-w-0 flex-1 flex-col gap-2">
            {#if program.description}
              <p
                class="text-body-md break-words whitespace-pre-line text-ink-body"
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
          <div>
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
        </div>
      </section>

      <Panel
        aria-labelledby="program-sessions-heading"
        class="lg:col-start-1 lg:row-start-1"
      >
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
              <li class="min-w-0">
                <MeditationRow
                  title={meditation.name}
                  meta={[
                    formatDuration(
                      meditation.duration_ms,
                      locale.current,
                      "short",
                    ),
                    meditationProgressLabel(meditation, locale.current),
                  ].join(" • ")}
                  description={meditation.description}
                  playLabel={t(contentMessages.play, {
                    title: meditation.name,
                  })}
                  playHref={playerHref(meditation.id)}
                />
              </li>
            {/each}
          </ol>
        </RegionState>
      </Panel>
    </div>
  {/if}
</RegionState>
