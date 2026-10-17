<!--
  @component
  Inline: a wide cover with the name labelled at its bottom-left. Floating
  (while playing): a static centered circular cover with the name below.
-->
<script lang="ts">
  import type { Meditation, ProgramSummary } from "$lib/api/meditation";
  import { t } from "$lib/i18n/locale.svelte";
  import { ROUTES } from "$lib/navigation/routes";
  import PlayerArtwork from "./player-artwork.svelte";
  import { playerMessages as m } from "./player.messages";

  type Props = {
    meditation: Meditation;
    /** undefined while the program is looked up; null for none. */
    program: ProgramSummary | null | undefined;
    playing: boolean;
    showCover: boolean;
    onCoverError: () => void;
  };

  let { meditation, program, playing, showCover, onCoverError }: Props =
    $props();

  const inlineCoverClass =
    "relative block aspect-video w-full overflow-hidden lg:aspect-auto lg:h-72 lg:max-h-72 rounded-tile bg-surface-solid/50";
</script>

{#snippet artwork()}
  <PlayerArtwork
    {program}
    meditationId={meditation.id}
    {showCover}
    {onCoverError}
  />
{/snippet}

{#snippet coverTitle()}
  <div
    class="absolute inset-x-0 bottom-0 z-10 flex flex-col items-start gap-1 p-5 sm:p-6"
  >
    <h1
      class="max-w-full rounded-full bg-surface-solid/85 px-4 py-2 text-title-lg break-words text-ink-strong backdrop-blur-sm"
    >
      {meditation.name}
    </h1>
  </div>
{/snippet}

{#if playing}
  <div class="flex flex-col items-center gap-4">
    {#if program}
      <a
        href={`${ROUTES.program.path}?id=${program.id}`}
        aria-label={t(m.programLink, { name: program.title })}
        class="focus-ring relative block size-40 overflow-hidden rounded-full bg-surface-solid/50"
      >
        {@render artwork()}
      </a>
    {:else}
      <div
        class="relative block size-40 overflow-hidden rounded-full bg-surface-solid/50"
      >
        {@render artwork()}
      </div>
    {/if}
    <div class="flex min-w-0 flex-col items-center gap-2">
      <h1 class="min-w-0 text-title-lg text-center break-words text-ink-strong">
        {meditation.name}
      </h1>
      {#if meditation.description}
        <p
          class="max-w-prose text-body-md text-center break-words text-ink-body"
        >
          {meditation.description}
        </p>
      {/if}
    </div>
  </div>
{:else}
  <div class="flex min-w-0 flex-col gap-3">
    {#if program}
      <a
        href={`${ROUTES.program.path}?id=${program.id}`}
        aria-label={t(m.programLink, { name: program.title })}
        class="focus-ring {inlineCoverClass}"
      >
        {@render artwork()}
        {@render coverTitle()}
      </a>
    {:else}
      <div class={inlineCoverClass}>
        {@render artwork()}
        {@render coverTitle()}
      </div>
    {/if}
    {#if meditation.description}
      <p class="text-body-md break-words text-ink-body">
        {meditation.description}
      </p>
    {/if}
  </div>
{/if}
