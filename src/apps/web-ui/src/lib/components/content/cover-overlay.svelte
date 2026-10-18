<!--
  @component
  The image-first cover treatment (DESIGN_SYSTEM §5.7): a dark scrim over the
  cover, with the caption (title, meta, progress) pinned to its bottom edge in
  the on-image ink. One owner for the scrim and ink, used by ContentCard's
  `overlay` variant, the Player cover and the Program hero, so they cannot
  drift apart. The scrim reaches full strength before the caption begins, so
  the ink holds 4.5:1 over any artwork (global-css-contrast.test.ts).

  Render it inside a `relative overflow-hidden` box that holds the cover
  image or artwork; it only adds the scrim and the caption.

  Props: `children` (the caption: heading, meta, ...), `class` (caption
  padding, default `p-3`).
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils.js";

  type Props = {
    children: Snippet;
    class?: string;
  };

  let { children, class: className }: Props = $props();
</script>

<span
  aria-hidden="true"
  class="absolute inset-0 bg-(image:--gradient-image-scrim)"
></span>
<span
  class={cn(
    "absolute inset-x-0 bottom-0 flex flex-col gap-0.5 p-3 text-(--ink-on-image)",
    className,
  )}
>
  {@render children()}
</span>
