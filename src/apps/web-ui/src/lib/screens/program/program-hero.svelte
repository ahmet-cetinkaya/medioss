<!--
  @component
  Program detail hero: a 16:9 cover with the program title on the shared
  image-first scrim (CoverOverlay, the Library card treatment) and the
  progress as a fill bar at its bottom edge. The description and the primary
  action live below the hero, not on the image. Falls back to generated
  artwork when there is no cover or it fails to load.

  Props: `program` (id, title, has_cover), `progress` (0..1), `progressLabel`
  (localized state, read by screen readers; the bar is decorative).
-->
<script lang="ts">
  import { programCoverUrl, type Program } from "$lib/api/meditation";
  import CoverOverlay from "$lib/components/content/cover-overlay.svelte";
  import GeneratedArtwork from "$lib/components/content/generated-artwork.svelte";

  type Props = {
    program: Pick<Program, "id" | "title" | "has_cover">;
    progress: number;
    progressLabel: string;
  };

  let { program, progress, progressLabel }: Props = $props();

  // A broken cover falls back to artwork; a new program gets a fresh try.
  let failedId = $state<number | null>(null);
  const showCover = $derived(program.has_cover && failedId !== program.id);
  const progressWidth = $derived(Math.min(Math.max(progress, 0), 1) * 100);
</script>

<div
  data-elevation="featured"
  class="elevation-3 relative block aspect-video w-full overflow-hidden rounded-tile bg-surface-solid/50"
>
  {#if showCover}
    <img
      src={programCoverUrl(program.id)}
      alt=""
      decoding="async"
      class="absolute inset-0 size-full object-cover"
      onerror={() => (failedId = program.id)}
    />
  {:else}
    <GeneratedArtwork seed={program.id} category="meditation" />
  {/if}
  <CoverOverlay class="gap-2 p-5 sm:p-6">
    <h2
      id="program-title"
      class="max-w-full text-title-lg break-words text-(--ink-on-image)"
    >
      {program.title}
    </h2>
    {#if progressWidth > 0}
      <span
        aria-hidden="true"
        class="block h-1 overflow-hidden rounded-full bg-(--ink-on-image-track)"
      >
        <span
          class="block h-full rounded-full bg-(--ink-on-image-fill)"
          style:width="{progressWidth}%"
        ></span>
      </span>
    {/if}
    <span class="sr-only">{progressLabel}</span>
  </CoverOverlay>
</div>
