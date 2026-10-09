<!--
  @component
  Daily mood trend (FR-057): an aria-hidden SVG line with points in
  `--chart-mark` (3:1 on every surface), score 5 at the top and 1 at the
  bottom, plus a text list of each day's average as its alternative. Days are
  calendar days as the server reports them (formatCalendarDay, never shifted).

  Props: `trend` (the statistics trend, oldest first, at least one day).
-->
<script lang="ts">
  import type { MoodStatistics } from "$lib/api/mood";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { formatCalendarDay } from "$lib/i18n/formatting/date-time-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { formatMean, trendPoints } from "./mood-insights";
  import { moodMessages as m } from "./mood.messages";

  let { trend }: { trend: MoodStatistics["trend"] } = $props();

  const WIDTH = 320;
  const HEIGHT = 120;
  const PAD = 10;
  const LEVELS = [1, 3, 5];

  const points = $derived(
    trendPoints(
      trend.map((day) => day.average_score),
      WIDTH,
      HEIGHT,
      PAD,
    ),
  );
  const levelY = (level: number) =>
    PAD + ((5 - level) / 4) * (HEIGHT - 2 * PAD);
</script>

<div class="flex flex-col gap-3">
  <svg
    viewBox="0 0 {WIDTH} {HEIGHT}"
    class="h-auto w-full"
    aria-hidden="true"
    focusable="false"
  >
    {#each LEVELS as level (level)}
      <line
        x1={PAD}
        x2={WIDTH - PAD}
        y1={levelY(level)}
        y2={levelY(level)}
        class="stroke-divider"
        stroke-width="1"
      />
    {/each}
    {#if points.length > 1}
      <polyline
        points={points.map((point) => `${point.x},${point.y}`).join(" ")}
        fill="none"
        class="stroke-chart-mark"
        stroke-width="2.5"
        stroke-linejoin="round"
        stroke-linecap="round"
      />
    {/if}
    {#each points as point, index (index)}
      <circle cx={point.x} cy={point.y} r="4" class="fill-chart-mark" />
    {/each}
  </svg>
  <details class="group">
    <summary
      class="focus-ring inline-flex min-h-11 cursor-pointer items-center gap-1 rounded-full px-1 text-sm font-medium text-ink-body"
    >
      <AppIcon
        name="caret-down"
        size={16}
        class="-rotate-90 group-open:rotate-0 motion-safe:transition-transform"
      />
      {t(m.dailyAverages)}
    </summary>
    <ul
      class="mt-2 grid grid-cols-1 gap-1 text-sm text-ink-body sm:grid-cols-2"
    >
      {#each trend as day (day.date)}
        <li>
          {t(m.dailyAverage, {
            day: formatCalendarDay(day.date, locale.current),
            mean: formatMean(day.average_score, locale.current),
          })}
        </li>
      {/each}
    </ul>
  </details>
</div>
