<!--
  @component
  The sound library (FR-049, FR-054, PB-16): category chips ("All" by
  default) and sound tiles grouped by category. Each tile shows the name, the
  duration, an add/remove action whose label follows the mix, and download.
  The grid wraps, so many built-in sounds never scroll the page sideways.
-->
<script lang="ts">
  import OverflowMenu from "$lib/components/ui/menu/overflow-menu.svelte";
  import OverflowItem from "$lib/components/ui/menu/overflow-item.svelte";
  import { downloadFile } from "$lib/download-file";
  import {
    soundDownloadUrl,
    type SoundscapeCategory,
    type SoundscapeSound,
  } from "$lib/api/soundscape";
  import RegionState from "$lib/components/feedback/region-state.svelte";
  import SoundIcon from "$lib/components/icons/sound-icon.svelte";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import FilterSelect from "$lib/components/filters/filter-select.svelte";
  import { IconButton } from "$lib/components/ui/icon-button/index.js";
  import { Panel } from "$lib/components/ui/panel/index.js";
  import { actionMessages } from "$lib/i18n/common.messages";
  import { formatDuration } from "$lib/i18n/formatting/duration-format";
  import { locale } from "$lib/i18n/locale.svelte";
  import { t } from "$lib/i18n/translate";
  import { ROUTES } from "$lib/navigation/routes";
  import { session } from "$lib/session/session.svelte";
  import type { MixState } from "./mix-state.svelte";
  import { soundscapeMessages as m } from "./soundscape.messages";

  type Props = {
    mix: MixState;
    library: {
      categories: SoundscapeCategory[];
      sounds: SoundscapeSound[];
    } | null;
    status: "loading" | "error" | "empty" | "ready";
    errorMessage?: string;
    onRetry: () => void;
  };

  let { mix, library, status, errorMessage, onRetry }: Props = $props();

  const ALL = "all";
  let selected = $state<string>(ALL);

  const groups = $derived(
    (library?.categories ?? [])
      .map((category) => ({
        category,
        sounds: library!.sounds.filter(
          (sound) => sound.category_id === category.id,
        ),
      }))
      .filter((group) => group.sounds.length > 0),
  );

  const options = $derived([
    { value: ALL, label: t(m.allCategories) },
    ...groups.map((group) => ({
      value: String(group.category.id),
      label: group.category.name,
    })),
  ]);

  const visible = $derived.by(() => {
    const chosen = groups.filter(
      (group) => String(group.category.id) === selected,
    );
    return chosen.length ? chosen : groups;
  });
</script>

{#snippet adminLink()}
  <Button variant="secondary" href={`/admin/soundscape/`}>
    {t(m.openAdmin)}
  </Button>
{/snippet}

<Panel aria-labelledby="soundscape-sounds-heading">
  {#snippet heading()}
    <h2 id="soundscape-sounds-heading" class="text-heading text-ink-strong">
      {t(m.soundsHeading)}
    </h2>
  {/snippet}
  <RegionState
    {status}
    minHeight="16rem"
    {errorMessage}
    {onRetry}
    emptyTitle={t(m.noSoundsTitle)}
    emptyText={t(m.noSoundsText)}
    emptyAction={session.isAdmin ? adminLink : undefined}
  >
    <div class="flex flex-col gap-6">
      <FilterSelect
        {options}
        bind:value={selected}
        legend={t(m.categoryLegend)}
      />
      {#each visible as group (group.category.id)}
        <section
          aria-labelledby="soundscape-category-{group.category.id}"
          class="flex flex-col gap-3"
        >
          <h3
            id="soundscape-category-{group.category.id}"
            class="text-lg font-semibold break-words text-ink-strong"
          >
            {group.category.name}
          </h3>
          <ul
            class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,14rem),1fr))] gap-3"
          >
            {#each group.sounds as sound (sound.id)}
              {@const inMix = mix.has(sound.id)}
              <li
                class="flex min-w-0 flex-col justify-between gap-3 rounded-tile border border-rim bg-surface-solid/70 p-4"
                data-in-mix={inMix || undefined}
              >
                <div class="flex items-start justify-between gap-2">
                  <SoundIcon {sound} categorySlug={group.category.slug} />
                  <div class="min-w-0 flex-1">
                    <p class="font-medium break-words text-ink-strong">
                      {sound.name}
                    </p>
                    <p class="text-sm text-ink-muted">
                      {formatDuration(sound.duration_ms, locale.current)}
                    </p>
                  </div>
                  {#if inMix}
                    <span
                      class="inline-flex shrink-0 items-center gap-1 text-xs font-semibold text-ink-strong"
                    >
                      <AppIcon name="check-circle" size={16} />
                      {t(m.inMix)}
                    </span>
                  {/if}
                </div>
                <div class="flex items-center justify-between gap-2">
                  <Button
                    variant={inMix ? "secondary" : "ghost"}
                    size="sm"
                    aria-label={t(inMix ? m.removeFromMix : m.addToMix, {
                      name: sound.name,
                    })}
                    onclick={() => mix.toggle(sound)}
                  >
                    <AppIcon name={inMix ? "x" : "plus"} size={16} />
                    {t(inMix ? actionMessages.remove : actionMessages.add)}
                  </Button>
                  <OverflowMenu
                    label={t(m.moreActionsFor, { name: sound.name })}
                  >
                    <OverflowItem
                      icon="download-simple"
                      label={t(m.downloadSound, { name: sound.name })}
                      onclick={() => downloadFile(soundDownloadUrl(sound.id))}
                    />
                  </OverflowMenu>
                </div>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  </RegionState>
</Panel>
