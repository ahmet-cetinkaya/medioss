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
  `actions` (Snippet), `overlay` (image-first variant: the title and meta sit
  on the cover over a dark scrim and nothing renders below it; default false),
  `progress` (0..1, overlay only: a thin fill bar at the cover's bottom edge,
  drawn only above 0) with `progressLabel` (the localized state, e.g.
  "40% complete" or "Completed", read by screen readers since the bar and
  the check badge are decorative), `compact` (smaller artwork, text and
  spacing for horizontal rows; default false), `aspect` ("4/3" | "3/4" |
  "3/2" thumbnail ratio; default "4/3", or "3/2" when `compact`), `class`.
  Without `href` the card is a button: `onselect` runs on click, `ariaLabel`
  names it and `current` marks it as the selected card.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ArtworkCategory } from "$lib/artwork/artwork-palette";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { cn } from "$lib/utils.js";
  import GeneratedArtwork from "./generated-artwork.svelte";

  type Aspect = "4/3" | "3/4" | "3/2";

  const ASPECT_CLASS: Record<Aspect, string> = {
    "4/3": "aspect-4/3",
    "3/4": "aspect-3/4",
    "3/2": "aspect-3/2",
  };

  type Props = {
    href?: string;
    onselect?: () => void;
    ariaLabel?: string;
    current?: boolean;
    aspect?: Aspect;
    title: string;
    meta?: string[];
    coverUrl?: string | null;
    artworkSeed: string | number;
    artworkCategory: ArtworkCategory;
    completed?: boolean;
    headingLevel?: 2 | 3 | 4;
    actions?: Snippet;
    compact?: boolean;
    overlay?: boolean;
    progress?: number;
    progressLabel?: string;
    class?: string;
  };

  let {
    href,
    onselect,
    ariaLabel,
    current = false,
    aspect,
    title,
    meta = [],
    coverUrl = null,
    artworkSeed,
    artworkCategory,
    completed = false,
    headingLevel = 3,
    actions,
    compact = false,
    overlay = false,
    progress,
    progressLabel,
    class: className,
  }: Props = $props();

  // A broken cover falls back to artwork; a new URL gets a fresh try.
  let failedUrl = $state<string | null>(null);
  const showCover = $derived(Boolean(coverUrl) && coverUrl !== failedUrl);
  const progressWidth = $derived(
    progress === undefined ? 0 : Math.min(Math.max(progress, 0), 1) * 100,
  );
  const thumbnailAspect = $derived(aspect ?? (compact ? "3/2" : "4/3"));
  const triggerProps = $derived(
    href === undefined
      ? {
          type: "button",
          "aria-label": ariaLabel,
          "aria-current": current ? ("true" as const) : undefined,
          onclick: onselect,
        }
      : { href },
  );
</script>

<div
  class={cn("flex min-w-0 flex-col", compact ? "gap-1" : "gap-2", className)}
>
  <svelte:element
    this={href === undefined ? "button" : "a"}
    {...triggerProps}
    class={cn(
      "group/card focus-ring flex min-w-0 flex-col text-start motion-safe:transition-colors motion-safe:duration-(--duration-fast)",
      overlay
        ? "rounded-thumb elevation-1"
        : "glass-panel elevation-1 rounded-panel border border-rim p-3 hover:text-accent-strong md:rounded-thumb md:border-0 md:bg-transparent md:p-0 md:shadow-none md:backdrop-blur-none",
      compact ? "min-h-11 gap-2" : "gap-3",
    )}
  >
    <span
      class={cn(
        "relative block overflow-hidden rounded-thumb",
        !overlay && "md:elevation-1",
        ASPECT_CLASS[thumbnailAspect],
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
        class={cn(
          "absolute inset-0",
          overlay
            ? "bg-(image:--gradient-image-scrim)"
            : "bg-(image:--gradient-image-wash)",
        )}
      ></span>
      {#if overlay}
        <span class="absolute inset-x-0 bottom-0 flex flex-col gap-0.5 p-3">
          {@render heading("text-(--ink-on-image)")}
          {@render metaLine("text-(--ink-on-image-muted)")}
          {#if progressWidth > 0}
            <span
              aria-hidden="true"
              class="mt-1 block h-1 overflow-hidden rounded-full bg-(--ink-on-image-track)"
            >
              <span
                class="block h-full rounded-full bg-(--ink-on-image-fill)"
                style:width="{progressWidth}%"
              ></span>
            </span>
          {/if}
          {#if progressLabel}
            <span class="sr-only">{progressLabel}</span>
          {/if}
        </span>
      {/if}
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
    {#if !overlay}
      <span
        class={cn("flex min-w-0 flex-col px-1", compact ? "gap-0.5" : "gap-1")}
      >
        {@render heading("text-ink-strong")}
        {@render metaLine("text-ink-muted")}
      </span>
    {/if}
  </svelte:element>
  {#if actions}
    <div class="flex flex-wrap items-center gap-2 px-1">
      {@render actions()}
    </div>
  {/if}
</div>

{#snippet heading(colorClass: string)}
  <!-- A button must not contain a heading, so the button variant flattens
       the title to a span (24-F3). -->
  <svelte:element
    this={href === undefined ? "span" : `h${headingLevel}`}
    class={[
      "font-medium break-words",
      compact ? "line-clamp-2 text-sm" : "text-body-lg",
      colorClass,
    ]}
  >
    {title}
  </svelte:element>
{/snippet}

{#snippet metaLine(colorClass: string)}
  {#if meta.length > 0}
    <span
      class={cn(
        "flex flex-wrap items-center gap-x-1.5 text-meta break-words",
        colorClass,
      )}
    >
      {#each meta as part, i (i)}
        {#if i > 0}
          <span aria-hidden="true" class="opacity-70">•</span>
        {/if}
        <span class="line-clamp-2 min-w-0">{part}</span>
      {/each}
    </span>
  {/if}
{/snippet}
