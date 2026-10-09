<!--
  @component
  Glass panel (DESIGN_SYSTEM §5.12): `glass-panel`, `rounded-panel`, 1 px rim,
  24 px padding. Panels never nest panels.

  Props: `as` ("section" | "div", default "section"), `tint` (adds
  `--gradient-panel-tint`), `emphasis` ("default" | "destructive"; the
  destructive rim marks danger zones such as Delete account), `heading`
  (snippet rendered above the content; give its h2 an id and pass
  `aria-labelledby`), `class`, `children`, and any other attribute
  (`aria-labelledby`, `id`, ...).
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { cn } from "$lib/utils.js";

  type Props = HTMLAttributes<HTMLElement> & {
    as?: "section" | "div";
    tint?: boolean;
    emphasis?: "default" | "destructive";
    heading?: Snippet;
    children: Snippet;
  };

  let {
    as = "section",
    tint = false,
    emphasis = "default",
    heading,
    class: className,
    children,
    ...restProps
  }: Props = $props();
</script>

<svelte:element
  this={as}
  data-slot="panel"
  class={cn(
    "glass-panel rounded-panel p-6",
    tint && "bg-(image:--gradient-panel-tint)",
    emphasis === "destructive" && "border-destructive!",
    className,
  )}
  {...restProps}
>
  {#if heading}
    <div class="mb-4">{@render heading()}</div>
  {/if}
  {@render children()}
</svelte:element>
