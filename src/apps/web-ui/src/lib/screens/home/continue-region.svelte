<!--
  @component
  Continue-program card (FR-025): a prominent bento card; `featured` lifts it
  to elevation-3 and marks it the screen's single featured element (FR-101).
  The whole pill is clickable and opens the Player on the next meditation.
  The next meditation's name comes from the program's members; if they fail
  to load, the card shows the program title only and stays playable. The
  parent learns the region's state through `onStatus` so it can hand
  `featured` to the suggestion card when nothing is in progress.

  Props: `featured` (default false), `onStatus` (called on every state
  change; "empty" means nothing is in progress).
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { ApiError } from "$lib/api/api-error";
  import {
    fetchContinueProgram,
    fetchProgramMembers,
    programCoverUrl,
    type ContinueProgram,
  } from "$lib/api/meditation";
  import { serverReason, type ServerReason } from "$lib/api/server-reason";
  import NowPlayingPill from "$lib/components/content/now-playing-pill.svelte";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { programProgressLabel } from "$lib/meditation/progress-label";
  import { playerHref } from "$lib/navigation/routes";
  import { cn } from "$lib/utils.js";
  import { homeLibrary } from "./home-library.svelte";
  import HomeStartAction, { hasStartAction } from "./home-start-action.svelte";
  import { homeMessages as m } from "./home.messages";
  import { reasonMessage } from "./reason-message";

  type RegionStatus = "loading" | "error" | "empty" | "ready";

  type Props = {
    featured?: boolean;
    onStatus?: (status: RegionStatus) => void;
  };

  let { featured = false, onStatus }: Props = $props();

  // `undefined` while loading; `null` when nothing is in progress.
  let current = $state<ContinueProgram | null | undefined>(undefined);
  let nextName = $state<string | null>(null);
  let failure = $state<ServerReason | null>(null);
  let request = 0;

  const status = $derived(
    failure
      ? "error"
      : current === undefined
        ? "loading"
        : current
          ? "ready"
          : "empty",
  );
  const noContent = $derived(homeLibrary.empty === true);

  $effect(() => onStatus?.(status));

  async function loadNextName(next: ContinueProgram, id: number) {
    try {
      const members = await fetchProgramMembers(next.program.id);
      const match = members.find((item) => item.id === next.meditation_id);
      if (id === request) nextName = match?.name ?? null;
    } catch {
      // The card still works with the program title only (FR-025).
    }
  }

  async function load() {
    const id = ++request;
    current = undefined;
    nextName = null;
    failure = null;
    try {
      const result = await fetchContinueProgram();
      if (id !== request) return;
      current = result;
      if (result) void loadNextName(result, id);
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
  aria-labelledby="home-continue-heading"
  data-elevation={featured ? "featured" : undefined}
  class={cn(
    "flex min-h-44 flex-col gap-3",
    featured ? "elevation-3" : "elevation-1",
  )}
>
  {#snippet heading()}
    <h2 id="home-continue-heading" class="text-heading text-ink-strong">
      {t(m.continueHeading)}
    </h2>
  {/snippet}
  <RegionState
    {status}
    minHeight="8rem"
    errorMessage={failure
      ? t(reasonMessage(failure, m.continueFailed))
      : undefined}
    onRetry={() => void load()}
    emptyTitle={t(noContent ? m.noContentTitle : m.continueEmptyTitle)}
    emptyText={t(noContent ? m.noContentText : m.continueEmptyText)}
    emptyAction={hasStartAction() ? startAction : undefined}
  >
    {#if current}
      {@const label = nextName ?? current.program.title}
      <NowPlayingPill
        title={current.program.title}
        meta={[
          ...(nextName ? [nextName] : []),
          programProgressLabel(current.program.progress, locale.current) ?? "",
        ]}
        thumbnailUrl={current.program.has_cover
          ? programCoverUrl(current.program.id)
          : null}
        artworkSeed={current.program.id}
        href={playerHref(current.meditation_id)}
        playLabel={t(m.playNext, { title: label })}
      />
    {/if}
  </RegionState>
</Panel>
