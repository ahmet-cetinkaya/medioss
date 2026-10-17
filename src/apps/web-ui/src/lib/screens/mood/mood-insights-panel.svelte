<!-- @component Mood insights for the range: entry count, mean, distribution bars and the trend chart. -->
<script lang="ts">
  import type { MoodStatistics } from "$lib/api/mood";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { moodScoreMessages } from "$lib/components/mood/mood-score.messages";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatNumber } from "$lib/i18n/formatting/number-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { distributionRows, formatMean } from "./mood-insights";
  import type { RegionStatus } from "./mood-status";
  import { moodMessages as m } from "./mood.messages";
  import TrendChart from "./trend-chart.svelte";

  let {
    statistics,
    status,
    errorMessage,
    onRetry,
    onChooseScore,
  }: {
    statistics: MoodStatistics | null;
    status: RegionStatus;
    errorMessage: string;
    onRetry: () => void;
    /** The no-data action: move focus to the score selector. */
    onChooseScore: () => void;
  } = $props();
</script>

<Panel aria-labelledby="mood-insights-heading">
  {#snippet heading()}
    <h2 id="mood-insights-heading" class="text-heading text-ink-strong">
      {t(m.insightsHeading)}
    </h2>
  {/snippet}
  <RegionState
    {status}
    minHeight="16rem"
    {errorMessage}
    {onRetry}
    emptyTitle={t(m.noDataTitle)}
    emptyText={t(m.noDataText)}
  >
    {#snippet emptyAction()}
      <Button variant="secondary" onclick={onChooseScore}>
        {t(m.chooseScore)}
      </Button>
    {/snippet}
    {#if statistics && statistics.entry_count > 0}
      <div class="flex flex-col gap-6">
        <dl class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-1 rounded-tile bg-surface-solid/50 p-4">
            <dt class="text-label text-ink-muted">{t(m.entryCount)}</dt>
            <dd class="text-title-lg font-semibold text-ink-strong">
              {formatNumber(statistics.entry_count, locale.current)}
            </dd>
          </div>
          <div class="flex flex-col gap-1 rounded-tile bg-surface-solid/50 p-4">
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
