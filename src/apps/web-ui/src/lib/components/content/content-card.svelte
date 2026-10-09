<!--
  @component
  Content card (DESIGN_SYSTEM §5.7). The thumbnail, title and meta line are
  one link. The thumbnail shows the cover image (decorative, `alt=""`) or
  generated artwork when there is no cover or it fails to load, under the
  white wash. `completed` adds a decorative check badge; put the completed
  state in `meta` as text too (FR-092). `actions` (e.g. a download button)
  render below the link, never inside it.

  Props: `href`, `title` (user content, shown as-is), `meta` (formatted
  parts, joined by an aria-hidden bullet), `coverUrl`, `artworkSeed`,
  `artworkCategory`, `completed`, `headingLevel` (2 | 3 | 4, default 3),
  `actions` (Snippet), `compact` (smaller artwork, text and
  spacing for horizontal rows; default false), `class`.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ArtworkCategory } from "$lib/artwork/artwork-palette";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { cn } from "$lib/utils.js";
  import GeneratedArtwork from "./generated-artwork.svelte";

  type Props = {
    href: string;
    title: string;
    meta?: string[];
    coverUrl?: string | null;
    artworkSeed: string | number;
    artworkCategory: ArtworkCategory;
    completed?: boolean;
    headingLevel?: 2 | 3 | 4;
    actions?: Snippet;
    compact?: boolean;
    class?: string;
  };

  let {
    href,
    title,
    meta = [],
    coverUrl = null,
    artworkSeed,
    artworkCategory,
    completed = false,
    headingLevel = 3,
    actions,
    compact = false,
    class: className,
  }: Props = $props();

  // A broken cover falls back to artwork; a new URL gets a fresh try.
  let failedUrl = $state<string | null>(null);
  const showCover = $derived(Boolean(coverUrl) && coverUrl !== failedUrl);
</script>

<div
  class={cn("flex min-w-0 flex-col", compact ? "gap-1" : "gap-2", className)}
>
  <a
    {href}
    class={cn(
      "group/card focus-ring glass-panel elevation-1 flex min-w-0 flex-col rounded-panel border border-rim p-3 motion-safe:transition-colors motion-safe:duration-(--duration-fast) hover:text-accent-strong md:rounded-thumb md:border-0 md:bg-transparent md:p-0 md:shadow-none md:backdrop-blur-none",
      compact ? "min-h-11 gap-2" : "gap-3",
    )}
  >
    <span
      class={cn(
        "relative block overflow-hidden rounded-thumb md:elevation-1",
        compact ? "aspect-3/2" : "aspect-4/3",
      )}
    >
      {#if showCover}
        <img
          src={coverUrl}
          alt=""
          loading="lazy"
          decoding="async"
          class="absolute inset-0 size-full object-cover"
          onerror={() => (failedUrl = coverUrl)}
        />
      {:else}
        <GeneratedArtwork seed={artworkSeed} category={artworkCategory} />
      {/if}
      <span
        aria-hidden="true"
        class="absolute inset-0 bg-(image:--gradient-image-wash)"
      ></span>
      {#if completed}
        <span
          aria-hidden="true"
          class={cn(
            "absolute flex items-center justify-center rounded-full bg-surface-solid text-success",
            compact ? "top-2 right-2 size-6" : "top-3 right-3 size-7",
          )}
        >
          <AppIcon name="check" size={14} />
        </span>
      {/if}
    </span>
    <div
      class={cn("flex min-w-0 flex-col px-1", compact ? "gap-0.5" : "gap-1")}
    >
      <svelte:element
        this={`h${headingLevel}`}
        class={cn(
          "font-medium break-words text-ink-strong",
          compact ? "line-clamp-2 text-sm" : "text-body-lg",
        )}
      >
        {title}
      </svelte:element>
      {#if meta.length > 0}
        <span
          class="flex flex-wrap items-center gap-x-1.5 text-meta break-words text-ink-muted"
        >
          {#each meta as part, i (i)}
            {#if i > 0}
              <span aria-hidden="true" class="text-ink-faint">•</span>
            {/if}
            <span class="min-w-0">{part}</span>
          {/each}
        </span>
      {/if}
    </div>
  </a>
  {#if actions}
    <div class="flex flex-wrap items-center gap-2 px-1">
      {@render actions()}
    </div>
  {/if}
</div>
