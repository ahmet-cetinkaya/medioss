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
  import RangeChips from "$lib/components/filters/range-chips.svelte";
  import { type MoodScore } from "$lib/components/mood/score-selector.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import MoodEntriesPanel from "./mood-entries-panel.svelte";
  import { entrySource, entryTime } from "./mood-entry-format";
  import MoodFormPanel from "./mood-form-panel.svelte";
  import { appendUnique } from "./mood-insights";
  import MoodInsightsPanel from "./mood-insights-panel.svelte";
  import { regionStatus } from "./mood-status";
  import { moodMessages as m } from "./mood.messages";

  const DEFAULT_SCORE: MoodScore = 3;

  // Form
  let score = $state<MoodScore>(DEFAULT_SCORE);
  let note = $state("");
  let editing = $state<MoodEntry | null>(null);
  let editTrigger: HTMLElement | null = null;
  let saving = $state(false);
  let saveFailure = $state<Failure | null>(null);
  let form = $state<ReturnType<typeof MoodFormPanel>>();

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
    regionStatus(!!statsFailure, !!statistics, statistics?.entry_count === 0),
  );
  const entriesStatus = $derived(
    regionStatus(!!listFailure, !!entries, entries?.length === 0),
  );

  const loadError = (reason: ServerReason | null) =>
    t(failureMessage(reason, m.loadFailed).message);

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
    form?.focus();
  }

  function cancelEdit() {
    const trigger = editTrigger;
    resetForm();
    if (trigger?.isConnected) trigger.focus();
    else form?.focus();
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
  <MoodFormPanel
    bind:this={form}
    bind:score
    bind:note
    editing={!!editing}
    {saving}
    {saveFailure}
    onSubmit={() => void save()}
    onCancel={cancelEdit}
  />

  <RangeChips
    options={ranges}
    bind:value={range}
    name="mood-range"
    legend={t(m.rangeLegend)}
  />

  <div class="grid grid-cols-1 items-start gap-6 lg:grid-cols-2">
    <MoodEntriesPanel
      {entries}
      status={entriesStatus}
      errorMessage={loadError(listFailure)}
      {nextCursor}
      {loadingMore}
      {moreFailure}
      bind:headingElement={entriesHeading}
      onRetry={() => void load(range)}
      onLoadMore={loadMore}
      onEdit={startEdit}
      onDelete={askDelete}
    />
    <MoodInsightsPanel
      {statistics}
      status={insightsStatus}
      errorMessage={loadError(statsFailure)}
      onRetry={() => void load(range)}
      onChooseScore={() => form?.focus()}
    />
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
