<!--
  @component
  Mood content (FR-055–FR-059, PB-9, PB-17). The form records a score (3
  preselected) and an optional note of up to 10 000 characters; Edit loads an
  entry into the form (the server keeps its recorded time) and Cancel resets
  it. The range chips (default All time) drive one load of insights and
  entries together. No data in the range → a no-data state in place of the
  insights and chart, whose action focuses the score selector.
-->
<script lang="ts">
  import {
    createMood,
    deleteMood,
    listMood,
    moodStatistics,
    updateMood,
    type MoodEntry,
    type MoodRange,
    type MoodStatistics,
  } from "$lib/api/mood";
  import { type ServerReason } from "$lib/api/server-reason";
  import { failureMessage, reasonOf } from "$lib/api/failure";
  import ConfirmDialog from "$lib/components/dialogs/confirm-dialog.svelte";
  import {
    describeFailure,
    type Failure,
  } from "$lib/components/dialogs/form-dialog.svelte";
  import { announce } from "$lib/components/feedback/announcer.svelte";
  import InlineMessage from "$lib/components/feedback/inline-message.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import RangeChips from "$lib/components/filters/range-chips.svelte";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { moodScoreMessages } from "$lib/components/mood/mood-score.messages";
  import ScoreSelector, {
    type MoodScore,
  } from "$lib/components/mood/score-selector.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { Textarea } from "$lib/components/ui/textarea/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDateTime } from "$lib/i18n/formatting/date-time-format";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import {
    appendUnique,
    distributionRows,
    formatMean,
    groupByDay,
  } from "./mood-insights";
  import { moodMessages as m } from "./mood.messages";
  import TrendChart from "./trend-chart.svelte";

  const NOTE_LIMIT = 10_000;
  const DEFAULT_SCORE: MoodScore = 3;

  // Form
  let score = $state<MoodScore>(DEFAULT_SCORE);
  let note = $state("");
  let editing = $state<MoodEntry | null>(null);
  let editTrigger: HTMLElement | null = null;
  let saving = $state(false);
  let saveFailure = $state<Failure | null>(null);
  let selector = $state<ScoreSelector | undefined>();

  // Data
  let range = $state<MoodRange>("all");
  let statistics = $state<MoodStatistics | null>(null);
  let entries = $state<MoodEntry[] | null>(null);
  let nextCursor = $state<string | null>(null);
  let statsFailure = $state<ServerReason | null>(null);
  let listFailure = $state<ServerReason | null>(null);
  let loadingMore = $state(false);
  let moreFailure = $state<Failure | null>(null);
  let request = 0;

  // Delete
  let pendingDelete = $state<MoodEntry | null>(null);
  let deleteOpen = $state(false);
  let entriesHeading = $state<HTMLElement | null>(null);

  const ranges = $derived([
    { value: "today" as const, label: t(m.rangeToday) },
    { value: "last7" as const, label: t(m.rangeLast7) },
    { value: "last30" as const, label: t(m.rangeLast30) },
    { value: "all" as const, label: t(m.rangeAll) },
  ]);

  const insightsStatus = $derived(
    statsFailure
      ? "error"
      : !statistics
        ? "loading"
        : statistics.entry_count === 0
          ? "empty"
          : "ready",
  );
  const entriesStatus = $derived(
    listFailure
      ? "error"
      : !entries
        ? "loading"
        : entries.length === 0
          ? "empty"
          : "ready",
  );

  const loadError = (reason: ServerReason | null) =>
    t(failureMessage(reason, m.loadFailed).message);

  const entryTime = (entry: MoodEntry) =>
    formatDateTime(entry.recorded_at, locale.current);

  const todayHeading = $derived(
    formatDateTime(new Date().toISOString(), locale.current, {
      weekday: "long",
      day: "numeric",
      month: "long",
    }),
  );
  const dayHeading = (iso: string) =>
    formatDateTime(iso, locale.current, {
      weekday: "long",
      day: "numeric",
      month: "long",
      year: "numeric",
    });
  const timeOnly = (iso: string) =>
    formatDateTime(iso, locale.current, { timeStyle: "short" });
  const days = $derived(groupByDay(entries ?? []));

  const MOOD_BG: Record<MoodScore, string> = {
    1: "bg-mood-1",
    2: "bg-mood-2",
    3: "bg-mood-3",
    4: "bg-mood-4",
    5: "bg-mood-5",
  };

  const entrySource = (entry: MoodEntry) =>
    t(m.entrySource, {
      score: entry.score,
      label: t(moodScoreMessages[entry.score as MoodScore]),
    });

  /** Loads insights and the first page for `selected`. `quiet` keeps the
   * current data on screen until the refreshed data arrives (after a save). */
  async function load(selected: MoodRange, quiet = false) {
    const id = ++request;
    statsFailure = null;
    listFailure = null;
    moreFailure = null;
    if (!quiet) {
      statistics = null;
      entries = null;
      nextCursor = null;
    }
    await Promise.all([
      moodStatistics(selected).then(
        (result) => {
          if (id === request) statistics = result;
        },
        (error) => {
          if (id === request) statsFailure = reasonOf(error);
        },
      ),
      listMood(selected).then(
        (page) => {
          if (id !== request) return;
          entries = page.items;
          nextCursor = page.next_cursor;
        },
        (error) => {
          if (id === request) listFailure = reasonOf(error);
        },
      ),
    ]);
  }

  $effect(() => {
    void load(range);
  });

  async function loadMore() {
    if (!nextCursor || loadingMore) return;
    const id = request;
    loadingMore = true;
    moreFailure = null;
    try {
      const page = await listMood(range, nextCursor);
      if (id === request && entries) {
        entries = appendUnique(entries, page.items);
        nextCursor = page.next_cursor;
      }
    } catch (error) {
      if (id === request) moreFailure = describeFailure(error);
    } finally {
      loadingMore = false;
    }
  }

  function resetForm() {
    editing = null;
    editTrigger = null;
    score = DEFAULT_SCORE;
    note = "";
    saveFailure = null;
  }

  function startEdit(entry: MoodEntry, trigger: HTMLElement | null) {
    editing = entry;
    editTrigger = trigger;
    score = entry.score as MoodScore;
    note = entry.note ?? "";
    saveFailure = null;
    selector?.focus();
  }

  function cancelEdit() {
    const trigger = editTrigger;
    resetForm();
    if (trigger?.isConnected) trigger.focus();
    else selector?.focus();
  }

  async function save() {
    if (saving) return;
    saving = true;
    saveFailure = null;
    try {
      const wasEditing = editing;
      if (wasEditing) await updateMood(wasEditing.id, score, note);
      else await createMood(score, note);
      announce(t(wasEditing ? m.updated : m.saved));
      const trigger = editTrigger;
      resetForm();
      await load(range, true);
      if (trigger?.isConnected) trigger.focus();
    } catch (error) {
      saveFailure = describeFailure(error);
    } finally {
      saving = false;
    }
  }

  function askDelete(entry: MoodEntry) {
    pendingDelete = entry;
    deleteOpen = true;
  }

  async function runDelete() {
    const entry = pendingDelete;
    if (!entry) return;
    await deleteMood(entry.id);
    if (editing?.id === entry.id) resetForm();
    announce(t(m.deleted));
    await load(range, true);
  }
</script>

<div class="flex flex-col gap-6">
  <Panel aria-labelledby="mood-form-heading" class="mx-auto w-full max-w-2xl">
    {#snippet heading()}
      <div class="flex flex-col gap-1 text-center">
        <p class="text-label text-ink-muted">{todayHeading}</p>
        <h2 id="mood-form-heading" class="text-heading text-ink-strong">
          {t(editing ? m.editHeading : m.formHeading)}
        </h2>
      </div>
    {/snippet}
    <form
      class="flex flex-col gap-5"
      onsubmit={(event) => {
        event.preventDefault();
        void save();
      }}
    >
      <ScoreSelector
        bind:this={selector}
        bind:value={score}
        name="mood-score"
        legend={t(m.scoreLegend)}
        legendHidden
        size="large"
        disabled={saving}
      />
      <div class="flex flex-col gap-2">
        <label for="mood-note" class="text-sm font-medium text-ink-body">
          {t(m.noteLabel)}
        </label>
        <Textarea
          id="mood-note"
          bind:value={note}
          maxlength={NOTE_LIMIT}
          rows={3}
          class="rounded-panel"
          aria-describedby="mood-note-hint"
          disabled={saving}
        />
        <p id="mood-note-hint" class="text-sm text-ink-muted">
          {t(m.noteHint, { limit: formatNumber(NOTE_LIMIT, locale.current) })}
        </p>
      </div>
      {#if saveFailure}
        <InlineMessage tone="error" role="alert">
          {t(saveFailure.message, saveFailure.params)}
        </InlineMessage>
      {/if}
      <div class="flex flex-wrap justify-center gap-3">
        <Button
          type="submit"
          size="lg"
          class="min-w-48"
          busy={saving}
          disabled={note.length > NOTE_LIMIT}
        >
          {t(editing ? m.saveChanges : m.saveMood)}
        </Button>
        {#if editing}
          <Button
            variant="secondary"
            size="lg"
            disabled={saving}
            onclick={cancelEdit}
          >
            {t(actionMessages.cancel)}
          </Button>
        {/if}
      </div>
    </form>
  </Panel>

  <RangeChips
    options={ranges}
    bind:value={range}
    name="mood-range"
    legend={t(m.rangeLegend)}
  />

  <div class="grid grid-cols-1 items-start gap-6 lg:grid-cols-2">
    <Panel aria-labelledby="mood-entries-heading">
      {#snippet heading()}
        <h2
          id="mood-entries-heading"
          bind:this={entriesHeading}
          tabindex="-1"
          class="text-heading text-ink-strong outline-none"
        >
          {t(m.entriesHeading)}
        </h2>
      {/snippet}
      <RegionState
        status={entriesStatus}
        minHeight="16rem"
        errorMessage={loadError(listFailure)}
        onRetry={() => void load(range)}
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
              <ul
                class="ms-5 flex flex-col gap-3 border-s-2 border-divider ps-6"
              >
                {#each day.items as entry (entry.id)}
                  <li
                    class="relative flex items-start gap-3 rounded-panel border border-rim bg-panel-tint/40 p-3"
                  >
                    <span
                      data-mood-dot={entry.score}
                      aria-hidden="true"
                      class={`grid size-10 shrink-0 place-items-center rounded-full text-primary-foreground ${MOOD_BG[entry.score as MoodScore]}`}
                    >
                      <AppIcon name="smiley" size={22} filled />
                    </span>
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
                        onclick={(trigger) => startEdit(entry, trigger)}
                      />
                      <OverflowItem
                        label={t(actionMessages.delete)}
                        icon="trash"
                        destructive
                        onclick={() => askDelete(entry)}
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
            onclick={loadMore}
          >
            {t(actionMessages.loadMore)}
          </Button>
        {/if}
      </RegionState>
    </Panel>
    <Panel aria-labelledby="mood-insights-heading">
      {#snippet heading()}
        <h2 id="mood-insights-heading" class="text-heading text-ink-strong">
          {t(m.insightsHeading)}
        </h2>
      {/snippet}
      <RegionState
        status={insightsStatus}
        minHeight="16rem"
        errorMessage={loadError(statsFailure)}
        onRetry={() => void load(range)}
        emptyTitle={t(m.noDataTitle)}
        emptyText={t(m.noDataText)}
      >
        {#snippet emptyAction()}
          <Button variant="secondary" onclick={() => selector?.focus()}>
            {t(m.chooseScore)}
          </Button>
        {/snippet}
        {#if statistics && statistics.entry_count > 0}
          <div class="flex flex-col gap-6">
            <dl class="grid grid-cols-2 gap-4">
              <div
                class="flex flex-col gap-1 rounded-tile bg-surface-solid/50 p-4"
              >
                <dt class="text-label text-ink-muted">{t(m.entryCount)}</dt>
                <dd class="text-title-lg font-semibold text-ink-strong">
                  {formatNumber(statistics.entry_count, locale.current)}
                </dd>
              </div>
              <div
                class="flex flex-col gap-1 rounded-tile bg-surface-solid/50 p-4"
              >
                <dt class="text-label text-ink-muted">{t(m.meanScore)}</dt>
                <dd class="text-title-lg font-semibold text-ink-strong">
                  {formatMean(statistics.average_score ?? 0, locale.current)}
                </dd>
              </div>
            </dl>

            <section
              aria-labelledby="mood-distribution-heading"
              class="flex flex-col gap-3"
            >
              <h3
                id="mood-distribution-heading"
                class="font-semibold text-ink-strong"
              >
                {t(m.distributionHeading)}
              </h3>
              <ul class="flex flex-col gap-3">
                {#each distributionRows(statistics.distribution, statistics.distribution_percentages, locale.current) as row (row.score)}
                  <li class="flex flex-col gap-1">
                    <div
                      class="flex flex-wrap items-baseline justify-between gap-x-3 text-sm"
                    >
                      <span class="font-medium text-ink-strong">
                        {row.score} · {t(moodScoreMessages[row.score])}
                      </span>
                      <span class="text-ink-body">
                        {t(m.distributionValue, {
                          count: row.count,
                          percent: row.percent,
                        })}
                      </span>
                    </div>
                    <div
                      aria-hidden="true"
                      class="h-2 overflow-hidden rounded-full bg-surface-solid/60"
                    >
                      <div
                        class="h-full rounded-full bg-chart-mark"
                        style:width="{row.share * 100}%"
                      ></div>
                    </div>
                  </li>
                {/each}
              </ul>
            </section>

            <section
              aria-labelledby="mood-trend-heading"
              class="flex flex-col gap-3"
            >
              <h3 id="mood-trend-heading" class="font-semibold text-ink-strong">
                {t(m.trendHeading)}
              </h3>
              <TrendChart trend={statistics.trend} />
            </section>
          </div>
        {/if}
      </RegionState>
    </Panel>
  </div>
</div>

<ConfirmDialog
  bind:open={deleteOpen}
  title={t(m.deleteTitle)}
  itemName={pendingDelete
    ? `${entrySource(pendingDelete)} · ${entryTime(pendingDelete)}`
    : undefined}
  consequence={t(m.deleteConsequence)}
  confirmLabel={t(m.deleteConfirm)}
  destructive
  onConfirm={runDelete}
  fallbackFocus={() => entriesHeading}
/>
