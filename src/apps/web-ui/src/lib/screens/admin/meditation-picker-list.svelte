<!-- @component Searchable card list of meditations for the pick-a-meditation
  dialog: each card names the meditation, its duration and the programs it
  belongs to; clicking a card picks it. -->
<script lang="ts">
  import type { Meditation } from "$lib/api/meditation";
  import { formatDuration } from "$lib/i18n/formatting/duration-format";
  import { locale, t } from "$lib/i18n/locale.svelte";
  import { meditationAdminMessages as m } from "./admin.messages";
  import { meditationMatchesSearch } from "./meditations-logic";

  let {
    listId,
    meditations,
    search,
    onPick,
  }: {
    listId: string;
    meditations: Meditation[];
    search: string;
    onPick: (meditation: Meditation) => void;
  } = $props();

  const matches = $derived(
    meditations.filter((meditation) =>
      meditationMatchesSearch(meditation, search),
    ),
  );
</script>

<ul
  id={listId}
  role="listbox"
  aria-label={t(m.searchMeditationsLabel)}
  class="grid max-h-80 grid-cols-1 gap-2 overflow-y-auto pe-1"
>
  {#each matches as meditation (meditation.id)}
    <li>
      <button
        type="button"
        role="option"
        aria-selected="false"
        class="flex w-full flex-wrap items-center gap-2 rounded-tile border border-rim bg-surface-solid/50 p-3 text-start hover:bg-surface-solid/80"
        onclick={() => onPick(meditation)}
      >
        <span class="grid min-w-0 flex-[1_1_12rem] grid-cols-1 gap-0.5">
          <span class="font-medium break-words">{meditation.name}</span>
          <span class="text-sm text-ink-muted">
            {formatDuration(meditation.duration_ms, locale.current, "short")}
            <span aria-hidden="true">·</span>
            {#each meditation.programs ?? [] as program, i (program.id)}
              {#if i > 0},{/if}
              {program.title}
            {:else}
              {t(m.pickerNoProgram)}
            {/each}
          </span>
        </span>
      </button>
    </li>
  {:else}
    <li class="text-sm text-ink-muted">{t(m.pickerNoMatches)}</li>
  {/each}
</ul>
