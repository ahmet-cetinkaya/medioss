<!--
  @component
  History content (FR-060–FR-062, PB-10). Range chips (default All time) and
  the notes-only switch drive one load; "Load more" appends the next cursor
  page de-duplicated by kind+id (appendPage). Each feed item states its event
  type in text with the meditation title or mood score, the local date and
  time, and the note. Events link nowhere (plan scope: no History links).
-->
<script lang="ts">
  import { tick } from "svelte";
  import { timeline, type MoodRange, type TimelineEvent } from "$lib/api/mood";
  import { type ServerReason } from "$lib/api/server-reason";
  import { failureMessage, reasonOf } from "$lib/api/failure";
  import FeedItem from "$lib/components/content/feed-item.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import RangeChips from "$lib/components/filters/range-chips.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { Toggle } from "$lib/components/ui/toggle/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDateTime } from "$lib/i18n/formatting/date-time-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { ROUTES } from "$lib/navigation/routes";
  import { historyMessages as m } from "./history.messages";
  import { appendPage, eventKey } from "./timeline-pages";

  let range = $state<MoodRange>("all");
  let notesOnly = $state(false);
  let events = $state<TimelineEvent[] | null>(null);
  let cursor = $state<string | null>(null);
  let failure = $state<ServerReason | null>(null);
  let moreFailure = $state<ServerReason | null>(null);
  let loadingMore = $state(false);
  let endMarker = $state<HTMLElement>();
  let request = 0;

  const status = $derived(
    failure ? "error" : !events ? "loading" : events.length ? "ready" : "empty",
  );

  const ranges = $derived([
    { value: "today" as const, label: t(m.rangeToday) },
    { value: "last7" as const, label: t(m.rangeLast7) },
    { value: "last30" as const, label: t(m.rangeLast30) },
    { value: "all" as const, label: t(m.rangeAll) },
  ]);

  const rangeLabel = $derived(
    ranges.find((option) => option.value === range)?.label ?? "",
  );

  const reasonText = (reason: ServerReason, fallback: typeof m.loadFailed) =>
    t(failureMessage(reason, fallback).message);

  function describe(event: TimelineEvent): string {
    switch (event.kind) {
      case "Mood":
        return t(m.moodEvent, { score: event.score ?? "" });
      case "MeditationNote":
        return t(m.noteEvent, { title: event.title });
      case "MeditationSession":
        return t(m.completedEvent, { title: event.title });
      default:
        return t(m.otherEvent, { title: event.title });
    }
  }

  async function load(selected: MoodRange, notes: boolean) {
    const id = ++request;
    events = null;
    cursor = null;
    failure = null;
    moreFailure = null;
    loadingMore = false;
    try {
      const page = await timeline(selected, undefined, notes);
      if (id !== request) return;
      events = appendPage([], page.items);
      cursor = page.next_cursor;
    } catch (error) {
      if (id === request) failure = reasonOf(error);
    }
  }

  async function loadMore() {
    if (!events || !cursor || loadingMore) return;
    const id = request;
    loadingMore = true;
    moreFailure = null;
    try {
      const page = await timeline(range, cursor, notesOnly);
      if (id !== request || !events) return;
      const before = events.length;
      events = appendPage(events, page.items);
      cursor = page.next_cursor;
      announce(t(m.loadedMore, { count: events.length - before }));
      if (!cursor) {
        // The Load more button is gone: keep focus on the list's end.
        await tick();
        endMarker?.focus();
      }
    } catch (error) {
      if (id === request) moreFailure = reasonOf(error);
    } finally {
      if (id === request) loadingMore = false;
    }
  }

  $effect(() => {
    void load(range, notesOnly);
  });
</script>

<div class="flex flex-col gap-6">
  <div class="flex flex-wrap items-center justify-between gap-4">
    <RangeChips
      options={ranges}
      bind:value={range}
      name="history-range"
      legend={t(m.rangeLegend)}
    />
    <Toggle bind:checked={notesOnly} label={t(m.notesOnly)} />
  </div>

  <Panel aria-labelledby="history-timeline-heading">
    {#snippet heading()}
      <h2 id="history-timeline-heading" class="text-heading text-ink-strong">
        {t(m.timelineHeading)}
      </h2>
    {/snippet}
    <RegionState
      {status}
      minHeight="20rem"
      errorMessage={failure ? reasonText(failure, m.loadFailed) : undefined}
      onRetry={() => void load(range, notesOnly)}
      emptyTitle={t(m.emptyTitle, { range: rangeLabel })}
      emptyText={t(notesOnly ? m.emptyFilterNotes : m.emptyFilterAll, {
        range: rangeLabel,
      })}
    >
      {#snippet emptyAction()}
        <div class="flex flex-wrap gap-3">
          <Button variant="secondary" href={ROUTES.library.path}>
            {t(m.openLibrary)}
          </Button>
          <Button variant="secondary" href={ROUTES.mood.path}>
            {t(m.openMood)}
          </Button>
        </div>
      {/snippet}
      {#if events}
        <ul class="flex flex-col">
          {#each events as event (eventKey(event))}
            <li
              data-event-key={eventKey(event)}
              class="border-t border-divider first:border-t-0"
            >
              <FeedItem
                source={describe(event)}
                meta={formatDateTime(event.recorded_at, locale.current)}
                body={event.note ?? undefined}
              />
            </li>
          {/each}
        </ul>
        <div class="flex flex-col items-start gap-3 pt-4">
          {#if moreFailure}
            <InlineMessage tone="error" role="alert">
              {reasonText(moreFailure, m.loadMoreFailed)}
            </InlineMessage>
          {/if}
          {#if cursor}
            <Button variant="secondary" busy={loadingMore} onclick={loadMore}>
              {t(actionMessages.loadMore)}
            </Button>
          {:else}
            <p
              bind:this={endMarker}
              tabindex="-1"
              class="text-sm text-ink-muted focus:outline-none"
            >
              {t(m.endReached)}
            </p>
          {/if}
        </div>
      {/if}
    </RegionState>
  </Panel>
</div>
