<!--
  @component
  Now-playing / suggestion pill (DESIGN_SYSTEM §5.10). The whole pill is one
  clickable target: the image, title and meta sit inside the same link as
  the play control, so clicking anywhere opens the Player. `playLabel` is the
  link's accessible name (default "Play {title}"). `state="loading"` swaps
  the glyph for the busy ring.

  Props: `title` (user content), `href` (the Player address), `meta`
  (formatted parts, joined by an aria-hidden vertical divider),
  `thumbnailUrl` (decorative image), `artworkSeed` + `artworkCategory`
  (artwork when there is no image), `playLabel` (translated), `state`
  ("idle" | "loading", default "idle"), `class`.
-->
<script lang="ts">
  import type { ArtworkCategory } from "$lib/artwork/artwork-palette";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { t } from "$lib/i18n/translate";
  import { cn } from "$lib/utils.js";
  import { contentMessages } from "./content.messages";
  import GeneratedArtwork from "./generated-artwork.svelte";

  type Props = {
    title: string;
    href: string;
    meta?: string[];
    thumbnailUrl?: string | null;
    artworkSeed?: string | number;
    artworkCategory?: ArtworkCategory;
    playLabel?: string;
    state?: "idle" | "loading";
    class?: string;
  };

  let {
    title,
    href,
    meta = [],
    thumbnailUrl = null,
    artworkSeed,
    artworkCategory = "meditation",
    playLabel,
    state: pillState = "idle",
    class: className,
  }: Props = $props();

  let failedUrl = $state<string | null>(null);
  const showImage = $derived(
    Boolean(thumbnailUrl) && thumbnailUrl !== failedUrl,
  );
</script>

<a
  {href}
  aria-label={playLabel ?? t(contentMessages.play, { title })}
  class={cn(
    "glass-solid focus-ring flex min-h-20 min-w-0 cursor-pointer items-center gap-3 rounded-full p-3 motion-safe:transition-colors motion-safe:duration-(--duration-fast) hover:text-accent-strong",
    className,
  )}
>
  <span
    aria-hidden="true"
    class="relative block size-14 shrink-0 overflow-hidden rounded-full"
  >
    {#if showImage}
      <img
        src={thumbnailUrl}
        alt=""
        decoding="async"
        class="absolute inset-0 size-full object-cover"
        onerror={() => (failedUrl = thumbnailUrl)}
      />
    {:else}
      <GeneratedArtwork
        seed={artworkSeed ?? title}
        category={artworkCategory}
      />
    {/if}
  </span>
  <span class="flex min-w-0 flex-1 flex-col gap-0.5">
    <span
      class="text-body-lg flex min-h-11 items-center font-semibold leading-tight break-words text-ink-strong"
    >
      {title}
    </span>
    {#if meta.length > 0}
      <span
        class="flex flex-wrap items-center gap-x-2 text-meta break-words text-ink-muted"
      >
        {#each meta as part, i (i)}
          {#if i > 0}
            <span aria-hidden="true" class="h-3 w-px bg-ink-faint"></span>
          {/if}
          <span class="min-w-0">{part}</span>
        {/each}
      </span>
    {/if}
  </span>
  <span
    aria-hidden="true"
    class={cn(
      "flex size-12 shrink-0 items-center justify-center rounded-full",
      pillState === "loading"
        ? "text-accent-strong"
        : "bg-accent-fill text-primary-foreground",
    )}
  >
    {#if pillState === "loading"}
      <span
        class="size-4 rounded-full border-2 border-current border-r-transparent motion-safe:animate-spin"
      ></span>
    {:else}
      <AppIcon name="play" filled />
    {/if}
  </span>
</a>
