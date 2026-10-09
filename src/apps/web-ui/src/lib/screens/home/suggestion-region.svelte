<!--
  @component
  Daily-suggestion card (FR-026): a prominent bento card; `featured` lifts it
  to elevation-3 and marks it the screen's single featured element (FR-101).
  The whole pill is clickable and opens the Player. The suggestion is null
  exactly when the library is empty, so this region also tells the other
  regions (FR-028).

  Props: `featured` (default false).
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { ApiError } from "$lib/api/api-error";
  import {
    fetchDailySuggestion,
    programCoverUrl,
    type Meditation,
  } from "$lib/api/meditation";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import NowPlayingPill from "$lib/components/content/now-playing-pill.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { formatDuration } from "$lib/i18n/formatting/duration-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { playerHref } from "$lib/navigation/routes";
  import { cn } from "$lib/utils.js";
  import { homeLibrary } from "./home-library.svelte";
  import HomeStartAction, { hasStartAction } from "./home-start-action.svelte";
  import { homeMessages as m } from "./home.messages";
  import { reasonMessage } from "./reason-message";

  type Props = { featured?: boolean };

  let { featured = false }: Props = $props();

  // `undefined` while loading; `null` when the library is empty.
  let suggestion = $state<Meditation | null | undefined>(undefined);
  let failure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(
    failure
      ? "error"
      : suggestion === undefined
        ? "loading"
        : suggestion
          ? "ready"
          : "empty",
  );

  async function load() {
    const id = ++request;
    suggestion = undefined;
    failure = null;
    try {
      const result = await fetchDailySuggestion();
      if (id !== request) return;
      suggestion = result;
      homeLibrary.empty = result === null;
    } catch (error) {
      if (id === request)
        failure =
          error instanceof ApiError ? serverReason(error).reason : "generic";
    }
  }

  onMount(() => {
    void load();
  });
</script>

{#snippet startAction()}<HomeStartAction />{/snippet}

<Panel
  aria-labelledby="home-suggestion-heading"
  data-elevation={featured ? "featured" : undefined}
  class={cn(
    "flex min-h-44 flex-col gap-3",
    featured ? "elevation-3" : "elevation-1",
  )}
>
  {#snippet heading()}
    <h2 id="home-suggestion-heading" class="text-heading text-ink-strong">
      {t(m.suggestionHeading)}
    </h2>
  {/snippet}
  <RegionState
    {status}
    minHeight="8rem"
    errorMessage={failure
      ? t(reasonMessage(failure, m.suggestionFailed))
      : undefined}
    onRetry={() => void load()}
    emptyTitle={t(m.noContentTitle)}
    emptyText={t(m.noContentText)}
    emptyAction={hasStartAction() ? startAction : undefined}
  >
    {#if suggestion}
      <NowPlayingPill
        title={suggestion.name}
        meta={[formatDuration(suggestion.duration_ms, locale.current)]}
        thumbnailUrl={suggestion.program_has_cover && suggestion.program_id
          ? programCoverUrl(suggestion.program_id)
          : null}
        artworkSeed={suggestion.id}
        href={playerHref(suggestion.id)}
      />
    {/if}
  </RegionState>
</Panel>
