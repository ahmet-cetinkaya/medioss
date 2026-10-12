<!--
  @component
  Category tile (DESIGN_SYSTEM §5.5): one link, a rimmed frame around
  washed generated artwork, title at the bottom-left over the wash.
  `featured` uses `elevation-3`, sets `data-elevation="featured"` (FR-101;
  one per screen) and shows a decorative 44 px arrow circle (`aria-hidden`,
  not a second tab stop).

  Props: `href`, `title` (translated), `variant` ("standard" | "featured",
  default "standard"), `artworkSeed`, `artworkCategory`, `description`
  (optional, translated), `class`.
-->
<script lang="ts">
  import type { ArtworkCategory } from "$lib/artwork/artwork-palette";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { cn } from "$lib/utils.js";
  import GeneratedArtwork from "./generated-artwork.svelte";

  type Props = {
    href: string;
    title: string;
    variant?: "standard" | "featured";
    artworkSeed: string | number;
    artworkCategory: ArtworkCategory;
    description?: string;
    class?: string;
  };

  let {
    href,
    title,
    variant = "standard",
    artworkSeed,
    artworkCategory,
    description,
    class: className,
  }: Props = $props();

  const featured = $derived(variant === "featured");
</script>

<a
  {href}
  data-elevation={featured ? "featured" : undefined}
  class={cn(
    "focus-ring block min-w-0 rounded-tile border border-rim bg-surface-solid/70 p-2 motion-safe:transition-colors motion-safe:duration-(--duration-fast) hover:bg-surface-solid/90",
    featured ? "elevation-3" : "elevation-1",
    className,
  )}
>
  <span
    class={cn(
      "relative flex h-full flex-col justify-end gap-1 overflow-hidden rounded-thumb p-6",
      featured ? "min-h-56 pr-18" : "min-h-36",
    )}
  >
    <GeneratedArtwork seed={artworkSeed} category={artworkCategory} wash />
    <span class="relative text-title-lg break-words text-ink-strong">
      {title}
    </span>
    {#if description}
      <span class="relative text-sm break-words text-ink-body">
        {description}
      </span>
    {/if}
    {#if featured}
      <span
        aria-hidden="true"
        class="absolute right-4 bottom-4 flex size-11 items-center justify-center rounded-full bg-surface-solid text-accent-strong"
      >
        <AppIcon name="arrow-up-right" size={20} />
      </span>
    {/if}
  </span>
</a>
