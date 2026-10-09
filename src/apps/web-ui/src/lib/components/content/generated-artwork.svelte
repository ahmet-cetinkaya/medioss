<!--
  @component
  Generated "aurora" artwork (DESIGN_SYSTEM §7) from
  `artworkPalette(seed, category)`. Decorative (`aria-hidden`). It is
  absolutely positioned and fills its container, which must be `relative`
  (and usually `overflow-hidden` with the container's radius).

  Props: `seed` (string | number, e.g. the item id), `category`
  (ArtworkCategory), `wash` (adds the white bottom wash for text on top,
  default false), `class`.
-->
<script lang="ts">
  import {
    artworkPalette,
    type ArtworkCategory,
  } from "$lib/artwork/artwork-palette";
  import { cn } from "$lib/utils.js";

  type Props = {
    seed: string | number;
    category: ArtworkCategory;
    wash?: boolean;
    class?: string;
  };

  let { seed, category, wash = false, class: className }: Props = $props();

  const palette = $derived(artworkPalette(seed, category));
</script>

<span
  aria-hidden="true"
  class={cn("pointer-events-none absolute inset-0 block", className)}
>
  <span
    class={cn(
      "aurora absolute inset-0 block",
      palette.mirrored && "-scale-x-100",
    )}
    style:--aurora-base={palette.base}
    style:--aurora-1={palette.colors[0]}
    style:--aurora-2={palette.colors[1]}
    style:--aurora-3={palette.colors[2]}
  ></span>
  {#if wash}
    <span class="absolute inset-0 block bg-(image:--gradient-image-wash)"
    ></span>
  {/if}
</span>
