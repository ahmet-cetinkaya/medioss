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
    fetchFavorites,
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
  import MeditationRow from "$lib/components/content/meditation-row.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import FilterSelect from "$lib/components/filters/filter-select.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { contentMessages } from "$lib/components/content/content.messages";
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
  import { favoriteProgramNames } from "./favorite-programs";
  import {
    filterGroups,
    groupPrograms,
    type LibraryFilter,
  } from "./library-groups";
  import { libraryMessages as m } from "./library.messages";

  let programs = $state<Program[] | null>(null);
  let categories = $state<Category[]>([]);
  let groupsFailure = $state<ServerReason | null>(null);
  let filter = $state<LibraryFilter>("all");

  let favorites = $state<Meditation[] | null>(null);
  let favoritesFailure = $state<ServerReason | null>(null);

  let results = $state<Meditation[] | null>(null);
  let searchFailure = $state<ServerReason | null>(null);
  let searchRequest = 0;

  const query = $derived(libraryQuery.query);

  const groups = $derived(programs ? groupPrograms(programs, categories) : []);
  const visibleGroups = $derived(
    filter === "favorites" ? [] : filterGroups(groups, filter),
  );
  const chips = $derived([
    { value: "all" as const, label: t(m.all) },
    { value: "completed" as const, label: t(m.filterCompleted) },
    { value: "in-progress" as const, label: t(m.filterInProgress) },
    { value: "not-started" as const, label: t(m.filterNotStarted) },
    { value: "favorites" as const, label: t(m.filterFavorites) },
  ]);

  const favoritesStatus = $derived(
    favoritesFailure
      ? "error"
      : !favorites
        ? "loading"
        : favorites.length === 0
          ? "empty"
          : "ready",
  );
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

  async function loadFavorites() {
    favorites = null;
    favoritesFailure = null;
    try {
      favorites = await fetchFavorites();
    } catch (error) {
      favoritesFailure = reasonOf(error);
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

  // Re-read on every selection so a heart toggled in the Player shows up.
  $effect(() => {
    if (filter === "favorites") void loadFavorites();
  });

  onMount(() => {
    void loadGroups();
    return libraryQuery.connect();
  });

  $effect(() => {
    void search(query);
  });

  // One horizontally scrolling row per category: cards keep their natural
  // width and pack from the left, so a short row never stretches. Negative
  // margin + padding keep focus rings and elevation unclipped.
  const rowClass =
    "-mx-2 flex scroll-px-2 snap-x snap-proximity gap-4 overflow-x-auto px-2 py-2";
  const itemClass = "w-48 shrink-0 snap-start";
  // A flex-wrap item has no width of its own, so without one the card shrinks
  // to its minimum content width; the cards are designed for the w-48 tile.
  const resultItemClass = "w-48 min-w-0";

  // Browsers skip scrolling a partly visible focus target, so a Tab into a
  // half-clipped card would leave it half-clipped. Bring it fully into view.
  const revealFocused = (event: FocusEvent) =>
    (event.target as HTMLElement)
      .closest("li")
      ?.scrollIntoView({ inline: "nearest", block: "nearest" });

  const programsLine = (meditation: Meditation) => {
    const names = favoriteProgramNames(meditation.programs);
    if (names === null) return null;
    const message =
      (meditation.programs?.length ?? 0) > 1
        ? m.favoritePrograms
        : m.favoriteProgram;
    return t(message, { names });
  };

  const programHref = (id: number) => `${ROUTES.program.path}?id=${id}`;
  const adminHref = `/admin/meditations/`;
</script>

{#snippet meditationCards(list: Meditation[])}
  <ul class="flex flex-wrap gap-4">
    {#each list as meditation (meditation.id)}
      <li class={resultItemClass}>
        <ContentCard
          compact
          aspect="3/4"
          overlay
          class="max-w-48"
          href={playerHref(meditation.id)}
          title={meditation.name}
          meta={[
            formatDuration(meditation.duration_ms, locale.current, "short"),
          ].filter((item): item is string => item !== null)}
          progress={meditation.completed
            ? 1
            : meditation.progress_percent / 100}
          progressLabel={meditationProgressLabel(meditation, locale.current) ??
            undefined}
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
{/snippet}

{#snippet favoriteRows(list: Meditation[])}
  <ul class="flex flex-col gap-3">
    {#each list as meditation (meditation.id)}
      <li class="min-w-0">
        <MeditationRow
          title={meditation.name}
          meta={[
            formatDuration(meditation.duration_ms, locale.current, "short"),
            meditationProgressLabel(meditation, locale.current),
          ]
            .filter((part): part is string => part !== null)
            .join(" • ")}
          detail={programsLine(meditation)}
          playLabel={t(contentMessages.play, { title: meditation.name })}
          playHref={playerHref(meditation.id)}
        />
      </li>
    {/each}
  </ul>
{/snippet}

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
      {@render meditationCards(results ?? [])}
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
      <FilterSelect
        options={chips}
        bind:value={filter}
        name="library-filter"
        legend={t(m.filterLegend)}
      />
      {#if filter === "favorites"}
        <RegionState
          status={favoritesStatus}
          minHeight="12rem"
          errorMessage={failureText(favoritesFailure, m.favoritesFailed)}
          onRetry={() => void loadFavorites()}
          emptyTitle={t(m.favoritesEmptyTitle)}
          emptyText={t(m.favoritesEmptyText)}
        >
          {@render favoriteRows(favorites ?? [])}
        </RegionState>
      {:else if visibleGroups.length === 0}
        <p class="text-ink-muted" role="status">
          <span class="font-medium text-ink-strong">
            {t(m.filterEmptyTitle)}
          </span>
          {t(m.filterEmptyText)}
        </p>
      {/if}
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
                  aspect="3/4"
                  overlay
                  class="max-w-48"
                  href={programHref(program.id)}
                  title={program.title}
                  progress={program.progress}
                  progressLabel={programProgressLabel(
                    program.progress,
                    locale.current,
                  ) ?? undefined}
                  coverUrl={program.has_cover
                    ? programCoverUrl(program.id)
                    : null}
                  artworkSeed={program.id}
                  artworkCategory="meditation"
                  completed={program.progress >= 1}
                ></ContentCard>
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
