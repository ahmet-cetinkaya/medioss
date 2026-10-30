<!-- @component The mood timeline grouped by day, with edit/delete per entry and load-more paging. -->
<script lang="ts">
  import type { MoodEntry } from "$lib/api/mood";
  import type { Failure } from "$lib/components/dialogs/form-dialog.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import MoodFace from "$lib/components/mood/mood-face.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDateTime } from "$lib/i18n/formatting/date-time-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { entrySource, entryTime } from "./mood-entry-format";
  import { groupByDay } from "./mood-insights";
  import type { RegionStatus } from "./mood-status";
  import { moodMessages as m } from "./mood.messages";

  let {
    entries,
    status,
    errorMessage,
    nextCursor,
    loadingMore,
    moreFailure,
    headingElement = $bindable(null),
    onRetry,
    onLoadMore,
    onEdit,
    onDelete,
  }: {
    entries: MoodEntry[] | null;
    status: RegionStatus;
    errorMessage: string;
    nextCursor: string | null;
    loadingMore: boolean;
    moreFailure: Failure | null;
    headingElement: HTMLElement | null;
    onRetry: () => void;
    onLoadMore: () => void;
    onEdit: (entry: MoodEntry, trigger: HTMLElement | null) => void;
    onDelete: (entry: MoodEntry) => void;
  } = $props();

  const days = $derived(groupByDay(entries ?? []));

  const dayHeading = (iso: string) =>
    formatDateTime(iso, locale.current, {
      weekday: "long",
      day: "numeric",
      month: "long",
      year: "numeric",
    });
  const timeOnly = (iso: string) =>
    formatDateTime(iso, locale.current, { timeStyle: "short" });
</script>

<Panel aria-labelledby="mood-entries-heading">
  {#snippet heading()}
    <h2
      id="mood-entries-heading"
      bind:this={headingElement}
      tabindex="-1"
      class="text-heading text-ink-strong outline-none"
    >
      {t(m.entriesHeading)}
    </h2>
  {/snippet}
  <RegionState
    {status}
    minHeight="16rem"
    {errorMessage}
    {onRetry}
    emptyTitle={t(m.noEntriesTitle)}
    emptyText={t(m.noEntriesText)}
  >
    <div class="flex flex-col gap-6" data-testid="mood-timeline">
      {#each days as day (day.key)}
        <section class="flex flex-col gap-3" data-testid="mood-day">
          <h3
            class="sticky top-0 z-10 bg-surface-solid/90 py-2 text-label font-semibold text-ink-muted backdrop-blur-sm"
          >
            {dayHeading(day.iso)}
          </h3>
          <ul class="ms-5 flex flex-col gap-3 border-s-2 border-divider ps-6">
            {#each day.items as entry (entry.id)}
              <li
                class="relative flex items-start gap-3 rounded-panel border border-rim bg-panel-tint/40 p-3"
              >
                <MoodFace score={entry.score} />
                <div class="flex min-w-0 flex-1 flex-col gap-1">
                  <p class="font-semibold break-words text-ink-strong">
                    {entrySource(entry)}
                  </p>
                  <p class="text-meta text-ink-muted">
                    {timeOnly(entry.recorded_at)}
                  </p>
                  {#if entry.note}
                    <p
                      class="text-sm break-words whitespace-pre-wrap text-ink-body"
                    >
                      {entry.note}
                    </p>
                  {/if}
                </div>
                <OverflowMenu
                  label={t(m.entryActions, { date: entryTime(entry) })}
                >
                  <OverflowItem
                    label={t(actionMessages.edit)}
                    icon="pencil-simple"
                    onclick={(trigger) => onEdit(entry, trigger)}
                  />
                  <OverflowItem
                    label={t(actionMessages.delete)}
                    icon="trash"
                    destructive
                    onclick={() => onDelete(entry)}
                  />
                </OverflowMenu>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
    {#if moreFailure}
      <InlineMessage tone="error" role="alert" class="mt-4">
        {t(moreFailure.message, moreFailure.params)}
      </InlineMessage>
    {/if}
    {#if nextCursor}
      <Button
        variant="secondary"
        class="mt-4"
        busy={loadingMore}
        onclick={onLoadMore}
      >
        {t(actionMessages.loadMore)}
      </Button>
    {/if}
  </RegionState>
</Panel>
