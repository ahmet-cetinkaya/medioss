<!--
  @component
  Statistics content (FR-063–FR-065, PB-11). Range chips (default All time)
  drive one load; the summary always shows the four figures (zeros included,
  FR-065) and the practiced days list each day's calendar date exactly as the
  server reports it (formatCalendarDay, never shifted) with its minutes.
-->
<script lang="ts">
  import {
    fetchStatistics,
    type StatisticsRange,
    type UserStatistics,
  } from "$lib/api/meditation";
  import { type ServerReason } from "$lib/api/server-reason";
  import {
    failureMessage,
    NETWORK_OR_SERVER_REASONS,
    reasonOf,
  } from "$lib/api/failure";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import RangeChips from "$lib/components/filters/range-chips.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatCalendarDay } from "$lib/i18n/formatting/date-time-format";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { ROUTES } from "$lib/navigation/routes";
  import { buildHeatmap } from "./practice-heatmap";
  import { statsMessages as m } from "./stats.messages";

  let range = $state<StatisticsRange>("all");
  let statistics = $state<UserStatistics | null>(null);
  let failure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(failure ? "error" : statistics ? "ready" : "loading");

  const ranges = $derived([
    { value: "today" as const, label: t(m.rangeToday) },
    { value: "last30" as const, label: t(m.rangeLast30) },
    { value: "all" as const, label: t(m.rangeAll) },
  ]);

  const figures = $derived(
    statistics
      ? [
          {
            label: m.totalMinutes,
            value: statistics.total_minutes,
            unit: "minute",
          },
          { label: m.totalDays, value: statistics.total_days, unit: "day" },
          {
            label: m.currentStreak,
            value: statistics.current_streak,
            unit: "day",
          },
          {
            label: m.longestStreak,
            value: statistics.longest_streak,
            unit: "day",
          },
        ]
      : [],
  );

  const errorMessage = $derived(
    t(
      failureMessage(failure, m.loadFailed, {
        specificReasons: NETWORK_OR_SERVER_REASONS,
      }).message,
    ),
  );

  // The heatmap's "today" is the browser's calendar day, as the server's
  // calendar is built from the offset the client sends.
  const localToday = () => {
    const now = new Date();
    const pad = (value: number) => String(value).padStart(2, "0");
    return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
  };
  const weeks = $derived(
    statistics ? buildHeatmap(statistics.calendar, range, localToday()) : [],
  );
  const levelClasses = [
    "bg-surface-solid/50 ring-1 ring-inset ring-ink-muted/25",
    "bg-accent-fill/25",
    "bg-accent-fill/50",
    "bg-accent-fill/75",
    "bg-accent-fill",
  ];
  const cellLabel = (date: string, minutes: number) =>
    `${formatCalendarDay(date, locale.current)}: ${withUnit(minutes, "minute")}`;

  const withUnit = (value: number, unit: string) =>
    formatNumber(value, locale.current, {
      style: "unit",
      unit,
      unitDisplay: "long",
    });

  async function load(selected: StatisticsRange) {
    const id = ++request;
    statistics = null;
    failure = null;
    try {
      const result = await fetchStatistics(selected);
      if (id === request) statistics = result;
    } catch (error) {
      if (id === request) failure = reasonOf(error);
    }
  }

  $effect(() => {
    void load(range);
  });
</script>

<div class="flex flex-col gap-6">
  <RangeChips
    options={ranges}
    bind:value={range}
    name="stats-range"
    legend={t(m.rangeLegend)}
  />

  <RegionState
    {status}
    minHeight="20rem"
    {errorMessage}
    onRetry={() => void load(range)}
  >
    {#if statistics}
      <div class="flex flex-col gap-6">
        <Panel aria-labelledby="stats-summary-heading">
          {#snippet heading()}
            <h2 id="stats-summary-heading" class="text-heading text-ink-strong">
              {t(m.summaryHeading)}
            </h2>
          {/snippet}
          <dl class="grid grid-cols-2 gap-4 lg:grid-cols-4">
            {#each figures as figure (figure.label)}
              <div
                class="flex flex-col gap-1 rounded-tile bg-surface-solid/50 p-4"
              >
                <dt class="text-label text-ink-muted">{t(figure.label)}</dt>
                <dd class="text-title-lg font-semibold text-ink-strong">
                  {withUnit(figure.value, figure.unit)}
                </dd>
              </div>
            {/each}
          </dl>
        </Panel>

        <Panel aria-labelledby="stats-days-heading">
          {#snippet heading()}
            <h2 id="stats-days-heading" class="text-heading text-ink-strong">
              {t(m.daysHeading)}
            </h2>
          {/snippet}
          <RegionState
            status={statistics.calendar.length ? "ready" : "empty"}
            minHeight="8rem"
            emptyTitle={t(m.noPracticeTitle)}
            emptyText={t(m.noPracticeText)}
          >
            {#snippet emptyAction()}
              <Button variant="secondary" href={ROUTES.library.path}>
                {t(m.openLibrary)}
              </Button>
            {/snippet}
            <div
              role="region"
              aria-label={t(m.heatmapLabel)}
              tabindex="0"
              class="overflow-x-auto rounded-tile p-1 focus-visible:outline-2 focus-visible:outline-accent-strong"
            >
              <div class="flex w-max flex-col gap-3">
                <div class="grid grid-flow-col grid-rows-7 gap-1">
                  {#each weeks as week, weekIndex (weekIndex)}
                    {#each week as cell, dayIndex (dayIndex)}
                      {#if cell}
                        <div
                          role="img"
                          aria-label={cellLabel(cell.date, cell.minutes)}
                          title={cellLabel(cell.date, cell.minutes)}
                          class="size-4 rounded-[4px] {levelClasses[
                            cell.level
                          ]}"
                        ></div>
                      {:else}
                        <div class="size-4" aria-hidden="true"></div>
                      {/if}
                    {/each}
                  {/each}
                </div>
                <div
                  class="flex items-center gap-2 text-xs text-ink-body"
                  aria-hidden="true"
                >
                  <span>{t(m.heatmapLess)}</span>
                  {#each levelClasses as levelClass, level (level)}
                    <span
                      class="size-4 rounded-[4px] {levelClass}"
                      data-level={level}
                    ></span>
                  {/each}
                  <span>{t(m.heatmapMore)}</span>
                </div>
              </div>
            </div>
          </RegionState>
        </Panel>
      </div>
    {/if}
  </RegionState>
</div>
