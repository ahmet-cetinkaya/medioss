<!--
  @component
  Statistics content (FR-063–FR-065, PB-11). Range chips (default All time)
  drive one load; the summary always shows the four figures (zeros included,
  FR-065) and the practiced days list each day's calendar date exactly as the
  server reports it (formatCalendarDay, never shifted) with its minutes.
-->
<script lang="ts">
  import { ApiError } from "$lib/api/api-error";
  import {
    fetchStatistics,
    type StatisticsRange,
    type UserStatistics,
  } from "$lib/api/meditation";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import RangeChips from "$lib/components/filters/range-chips.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatCalendarDay } from "$lib/i18n/formatting/date-time-format";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { serverReasonMessages } from "$lib/i18n/server-reason.messages";
  import { ROUTES } from "$lib/navigation/routes";
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
    failure === "network" || failure === "server"
      ? t(serverReasonMessages[failure])
      : t(m.loadFailed),
  );

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
      if (id === request)
        failure =
          error instanceof ApiError ? serverReason(error).reason : "generic";
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
            <ul class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3">
              {#each statistics.calendar as day (day.date)}
                <li
                  class="flex flex-wrap items-baseline justify-between gap-3 rounded-tile bg-surface-solid/50 px-4 py-3"
                >
                  <span class="font-medium text-ink-strong">
                    {formatCalendarDay(day.date, locale.current)}
                  </span>
                  <span class="text-sm text-ink-body">
                    {withUnit(day.minutes, "minute")}
                  </span>
                </li>
              {/each}
            </ul>
          </RegionState>
        </Panel>
      </div>
    {/if}
  </RegionState>
</div>
