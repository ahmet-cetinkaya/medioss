<!--
  @component
  Library content (FR-031–FR-037, PB-16, PB-19). Programs are grouped by
  category (library-groups.ts) behind filter chips. The header search keeps
  its query in `?q=` (libraryQuery); while a query is active the groups are
  hidden and matching meditations show as cards that open the Player, each
  with a download link outside the card link.
-->
<script lang="ts">
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import { downloadFile } from "$lib/download-file";
  import { onMount } from "svelte";
  import {
    fetchCategories,
    fetchLibrary,
    fetchPrograms,
    meditationDownloadUrl,
    programCoverUrl,
    type Category,
    type Meditation,
    type Program,
  } from "$lib/api/meditation";
  import { type ServerReason } from "$lib/api/server-reason";
  import { failureMessage, reasonOf } from "$lib/api/failure";
  import ContentCard from "$lib/components/content/content-card.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import RangeChips from "$lib/components/filters/range-chips.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDuration } from "$lib/i18n/formatting/duration-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { libraryQuery } from "$lib/library/library-query.svelte";
  import {
    meditationProgressLabel,
    programProgressLabel,
  } from "$lib/meditation/progress-label";
  import { ROUTES, playerHref } from "$lib/navigation/routes";
  import { session } from "$lib/session/session.svelte";
  import { groupPrograms } from "./library-groups";
  import { libraryMessages as m } from "./library.messages";

  const ALL = "all";

  let programs = $state<Program[] | null>(null);
  let categories = $state<Category[]>([]);
  let groupsFailure = $state<ServerReason | null>(null);
  let filter = $state(ALL);

  let results = $state<Meditation[] | null>(null);
  let searchFailure = $state<ServerReason | null>(null);
  let searchRequest = 0;

  const query = $derived(libraryQuery.query);

  const groups = $derived(programs ? groupPrograms(programs, categories) : []);
  const visibleGroups = $derived(
    filter === ALL ? groups : groups.filter((group) => group.key === filter),
  );
  const chips = $derived([
    { value: ALL, label: t(m.all) },
    ...groups.map((group) => ({
      value: group.key,
      label: group.category?.name ?? t(m.uncategorized),
    })),
  ]);

  const groupsStatus = $derived(
    groupsFailure
      ? "error"
      : !programs
        ? "loading"
        : programs.length === 0
          ? "empty"
          : "ready",
  );
  const searchStatus = $derived(
    searchFailure
      ? "error"
      : !results
        ? "loading"
        : results.length === 0
          ? "empty"
          : "ready",
  );

  const failureText = (
    failure: ServerReason | null,
    fallback: typeof m.loadFailed,
  ) => t(failureMessage(failure, fallback).message);

  async function loadGroups() {
    programs = null;
    groupsFailure = null;
    try {
      const [loadedPrograms, loadedCategories] = await Promise.all([
        fetchPrograms(),
        fetchCategories(),
      ]);
      categories = loadedCategories;
      programs = loadedPrograms;
    } catch (error) {
      groupsFailure = reasonOf(error);
    }
  }

  async function search(text: string) {
    const id = ++searchRequest;
    results = null;
    searchFailure = null;
    if (!text) return;
    try {
      const found = await fetchLibrary(text);
      if (id !== searchRequest) return;
      results = found;
      announce(t(m.resultCount, { count: found.length }));
    } catch (error) {
      if (id === searchRequest) searchFailure = reasonOf(error);
    }
  }

  onMount(() => {
    void loadGroups();
    return libraryQuery.connect();
  });

  $effect(() => {
    void search(query);
  });

  // Independent scrolling row; negative margin + padding keep focus rings and
  // elevation unclipped. min-w-0 on the flex parent chain prevents page scroll.
  const rowClass = "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3";
  const itemClass = "min-w-0";

  // Browsers skip scrolling a partly visible focus target, so a Tab into a
  // half-clipped card would leave it half-clipped. Bring it fully into view.
  const revealFocused = (event: FocusEvent) =>
    (event.target as HTMLElement)
      .closest("li")
      ?.scrollIntoView({ inline: "nearest", block: "nearest" });

  const programHref = (id: number) => `${ROUTES.program.path}?id=${id}`;
  const adminHref = `/admin/meditations/`;
</script>

{#snippet clearSearch()}
  <Button variant="secondary" onclick={libraryQuery.clear}>
    <AppIcon name="x" size={18} />
    {t(actionMessages.clearSearch)}
  </Button>
{/snippet}

{#if query}
  <section
    aria-labelledby="library-results-heading"
    class="flex flex-col gap-6"
  >
    <div class="flex flex-wrap items-center justify-between gap-3">
      <h2
        id="library-results-heading"
        class="text-heading break-words text-ink-strong"
      >
        {t(m.resultsHeading, { query })}
      </h2>
      {#if searchStatus !== "empty"}
        {@render clearSearch()}
      {/if}
    </div>
    <RegionState
      status={searchStatus}
      minHeight="16rem"
      errorMessage={failureText(searchFailure, m.searchFailed)}
      onRetry={() => void search(query)}
      emptyTitle={t(m.noMatchesTitle, { query })}
      emptyText={t(m.noMatchesText)}
    >
      {#snippet emptyAction()}
        {@render clearSearch()}
      {/snippet}
      <ul class="flex flex-wrap gap-4">
        {#each results ?? [] as meditation (meditation.id)}
          <li class={itemClass}>
            <ContentCard
              compact
              class="max-w-48 [&>a>span:first-child]:aspect-3/4"
              href={playerHref(meditation.id)}
              title={meditation.name}
              meta={[
                formatDuration(meditation.duration_ms, locale.current, "short"),
                meditationProgressLabel(meditation, locale.current),
              ].filter((item): item is string => item !== null)}
              coverUrl={meditation.program_has_cover && meditation.program_id
                ? programCoverUrl(meditation.program_id)
                : null}
              artworkSeed={meditation.id}
              artworkCategory="meditation"
              completed={meditation.completed}
            >
              {#snippet actions()}
                <OverflowMenu
                  label={t(m.moreActionsFor, { name: meditation.name })}
                >
                  <OverflowItem
                    icon="download-simple"
                    label={t(m.downloadNamed, { name: meditation.name })}
                    onclick={() =>
                      downloadFile(meditationDownloadUrl(meditation.id))}
                  />
                </OverflowMenu>
              {/snippet}
            </ContentCard>
          </li>
        {/each}
      </ul>
    </RegionState>
  </section>
{:else}
  <RegionState
    status={groupsStatus}
    minHeight="20rem"
    label={t(m.programsLabel)}
    headingLevel={2}
    errorMessage={failureText(groupsFailure, m.loadFailed)}
    onRetry={() => void loadGroups()}
    emptyTitle={t(m.emptyTitle)}
    emptyText={t(m.emptyText)}
    emptyAction={session.isAdmin ? adminLink : undefined}
  >
    <div class="flex flex-col gap-8">
      <RangeChips
        options={chips}
        bind:value={filter}
        name="library-category"
        legend={t(m.categoryLegend)}
      />
      {#each visibleGroups as group (group.key)}
        <section
          aria-labelledby="library-group-{group.key}"
          class="flex flex-col gap-4"
        >
          <h2
            id="library-group-{group.key}"
            class="text-heading break-words text-ink-strong"
          >
            {group.category?.name ?? t(m.uncategorized)}
          </h2>
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <ul class={rowClass} onfocusin={revealFocused}>
            {#each group.programs as program (program.id)}
              <li class={itemClass}>
                <ContentCard
                  class="max-w-48 [&>a>span:first-child]:aspect-3/4"
                  href={programHref(program.id)}
                  title={program.title}
                  meta={[
                    programProgressLabel(program.progress, locale.current),
                  ].filter((item): item is string => item !== null)}
                  coverUrl={program.has_cover
                    ? programCoverUrl(program.id)
                    : null}
                  artworkSeed={program.id}
                  artworkCategory="library"
                  completed={program.progress >= 1}
                >
                  {#snippet actions()}
                    {#if program.description}
                      <p class="line-clamp-2 text-xs break-words text-ink-body">
                        {program.description}
                      </p>
                    {/if}
                  {/snippet}
                </ContentCard>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  </RegionState>
{/if}

{#snippet adminLink()}
  <Button variant="secondary" href={adminHref}>
    {t(m.openAdmin)}
  </Button>
{/snippet}
