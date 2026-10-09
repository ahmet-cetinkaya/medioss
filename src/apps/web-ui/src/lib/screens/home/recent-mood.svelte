<!--
  @component
  Recent mood entries (FR-110, PB-17): the signed-in user's three latest
  check-ins with a link to Mood. Loads on its own with its own error and
  Retry. Exposes `reload()` for the check-in dialog; the empty state offers
  `onCheckIn`.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { ApiError } from "$lib/api/api-error";
  import { listMood, type MoodEntry } from "$lib/api/mood";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import FeedItem from "$lib/components/content/feed-item.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { moodScoreMessages } from "$lib/components/mood/mood-score.messages";
  import type { MoodScore } from "$lib/components/mood/score-selector.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatDateTime } from "$lib/i18n/formatting/date-time-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { ROUTES } from "$lib/navigation/routes";
  import { homeMessages as m } from "./home.messages";
  import { reasonMessage } from "./reason-message";

  type Props = { onCheckIn: () => void };

  let { onCheckIn }: Props = $props();

  const RECENT_COUNT = 3;

  let entries = $state<MoodEntry[] | null>(null);
  let loadFailure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(
    loadFailure
      ? "error"
      : entries === null
        ? "loading"
        : entries.length
          ? "ready"
          : "empty",
  );

  export async function reload() {
    const id = ++request;
    entries = null;
    loadFailure = null;
    try {
      const page = await listMood("all");
      if (id === request) entries = page.items.slice(0, RECENT_COUNT);
    } catch (error) {
      if (id === request)
        loadFailure =
          error instanceof ApiError ? serverReason(error).reason : "generic";
    }
  }

  onMount(() => {
    void reload();
  });
</script>

{#snippet checkInAction()}
  <Button variant="secondary" onclick={onCheckIn}>{t(m.checkInNow)}</Button>
{/snippet}

<Panel aria-labelledby="home-mood-heading">
  {#snippet heading()}
    <div class="flex flex-wrap items-center justify-between gap-2">
      <h2 id="home-mood-heading" class="text-heading text-ink-strong">
        {t(m.moodHeading)}
      </h2>
      <a
        href={ROUTES.mood.path}
        class="focus-ring inline-flex min-h-11 items-center rounded-full px-2 text-sm font-medium text-accent-strong hover:underline"
      >
        {t(m.openMood)}
      </a>
    </div>
  {/snippet}
  <RegionState
    {status}
    minHeight="8rem"
    errorMessage={loadFailure
      ? t(reasonMessage(loadFailure, m.moodFailed))
      : undefined}
    onRetry={() => void reload()}
    emptyTitle={t(m.moodEmptyTitle)}
    emptyText={t(m.moodEmptyText)}
    emptyAction={checkInAction}
  >
    {#if entries}
      <ul>
        {#each entries as entry (entry.id)}
          <FeedItem
            as="li"
            source={t(m.moodScore, {
              score: entry.score,
              label: t(moodScoreMessages[entry.score as MoodScore]),
            })}
            meta={formatDateTime(entry.recorded_at, locale.current)}
            body={entry.note ?? undefined}
          />
        {/each}
      </ul>
    {/if}
  </RegionState>
</Panel>
