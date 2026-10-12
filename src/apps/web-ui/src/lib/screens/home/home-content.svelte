<!--
  @component
  Home main column (FR-022–FR-030, FR-101, FR-108–FR-111), a bento layout in
  one column: the horizontal greeting hero (the page h1), the continue and
  suggestion cards side by side (the continue card is the screen's only
  featured element; when nothing is in progress, the suggestion card takes
  over), the quick check-in card, up to five recent programs with a "See all"
  link to History, and the three recent mood entries. Every region loads
  on its own, with its own error and Retry (FR-029).
-->
<script lang="ts">
  import { onMount } from "svelte";
  import GreetingHero from "$lib/components/content/greeting-hero.svelte";
  import { session } from "$lib/session/session.svelte";
  import CheckInCard from "./check-in-card.svelte";
  import CheckInDialog from "./check-in-dialog.svelte";
  import ContinueRegion from "./continue-region.svelte";
  import { greeting } from "./greeting";
  import RecentMood from "./recent-mood.svelte";
  import SuggestionRegion from "./suggestion-region.svelte";
  import {
    fetchRecentPrograms,
    programCoverUrl,
    type RecentProgram,
  } from "$lib/api/meditation";
  import { type ServerReason } from "$lib/api/server-reason";
  import { failureMessage, reasonOf } from "$lib/api/failure";
  import ContentCard from "$lib/components/content/content-card.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { programProgressLabel } from "$lib/meditation/progress-label";
  import { ROUTES } from "$lib/navigation/routes";
  import { homeLibrary } from "./home-library.svelte";
  import HomeStartAction, { hasStartAction } from "./home-start-action.svelte";
  import { homeMessages as m } from "./home.messages";

  const LEADS = {
    morning: m.greetingMorning,
    afternoon: m.greetingAfternoon,
    evening: m.greetingEvening,
  } as const;

  // Device local time when Home opened (FR-022).
  const now = new Date();
  const hello = $derived(greeting(now, session.user));

  let checkInOpen = $state(false);
  let recentMood = $state<RecentMood | null>(null);

  // FR-101: exactly one featured element, from the first paint. The continue
  // card is featured while anything is in progress (or loading or failed);
  // when nothing is in progress, the suggestion card takes over.
  let continueStatus = $state<"loading" | "error" | "empty" | "ready">(
    "loading",
  );
  const continueFeatured = $derived(continueStatus !== "empty");

  let recent = $state<RecentProgram[] | null>(null);
  let failure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(
    failure
      ? "error"
      : recent === null
        ? "loading"
        : recent.length
          ? "ready"
          : "empty",
  );
  const noContent = $derived(homeLibrary.empty === true);

  async function load() {
    const id = ++request;
    recent = null;
    failure = null;
    try {
      const result = await fetchRecentPrograms(5);
      if (id === request) recent = result.slice(0, 5);
    } catch (error) {
      if (id === request) failure = reasonOf(error);
    }
  }

  onMount(() => {
    void load();
  });
</script>

{#snippet startAction()}<HomeStartAction />{/snippet}

<div class="flex min-w-0 flex-col gap-6">
  <GreetingHero
    lead={t(LEADS[hello.band], { named: hello.name ? 1 : 0 })}
    name={hello.name}
    user={session.user}
    avatarVersion={session.avatarVersion}
  />

  <div class="grid grid-cols-1 items-stretch gap-6 md:grid-cols-2">
    <ContinueRegion
      featured={continueFeatured}
      onStatus={(value) => (continueStatus = value)}
    />
    <SuggestionRegion featured={!continueFeatured} />
  </div>

  <CheckInCard onOpen={() => (checkInOpen = true)} />

  <Panel aria-labelledby="home-recent-heading">
    {#snippet heading()}
      <div class="flex flex-wrap items-center justify-between gap-2">
        <h2 id="home-recent-heading" class="text-heading text-ink-strong">
          {t(m.recentHeading)}
        </h2>
        <a
          href={ROUTES.history.path}
          class="focus-ring inline-flex min-h-11 items-center rounded-full px-2 text-sm font-medium text-accent-strong hover:underline"
        >
          {t(actionMessages.seeAll)}
        </a>
      </div>
    {/snippet}
    <RegionState
      {status}
      minHeight="14rem"
      errorMessage={failure
        ? t(failureMessage(failure, m.recentFailed).message)
        : undefined}
      onRetry={() => void load()}
      emptyTitle={t(noContent ? m.noContentTitle : m.recentEmptyTitle)}
      emptyText={t(noContent ? m.noContentText : m.recentEmptyText)}
      emptyAction={hasStartAction() ? startAction : undefined}
    >
      {#if recent}
        <ul class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-5">
          {#each recent as entry (entry.program.id)}
            <li class="min-w-0">
              <ContentCard
                href={`${ROUTES.program.path}?id=${entry.program.id}`}
                title={entry.program.title}
                meta={[
                  programProgressLabel(
                    entry.program.progress,
                    locale.current,
                  ) ?? "",
                ]}
                coverUrl={entry.program.has_cover
                  ? programCoverUrl(entry.program.id)
                  : null}
                artworkSeed={entry.program.id}
                artworkCategory="meditation"
                completed={entry.program.progress >= 1}
                class="max-w-48 [&>a>span:first-child]:aspect-3/4"
              />
            </li>
          {/each}
        </ul>
      {/if}
    </RegionState>
  </Panel>

  <RecentMood bind:this={recentMood} onCheckIn={() => (checkInOpen = true)} />
</div>

<CheckInDialog
  bind:open={checkInOpen}
  onSaved={() => void recentMood?.reload()}
/>
