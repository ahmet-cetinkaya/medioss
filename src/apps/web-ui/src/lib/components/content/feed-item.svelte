<!--
  @component
  Feed item (DESIGN_SYSTEM §5.9): header (16 px orb or `avatar` + source
  name in weight 600, then `meta` such as a time), body text, optional action
  row. No bubbles; consecutive items are separated by a hairline. `align="end"`
  right-aligns items authored by the current user (default orb: "user").

  Props: `source` (speaker or source name), `body` (user content, shown
  as-is with its line breaks), `meta` (formatted text), `align` ("start" |
  "end", default "start"), `as` ("article" | "li", default "article"),
  `avatar` (Snippet), `actions` (Snippet), `class`.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils.js";
  import Orb from "./orb.svelte";

  type Props = {
    source: string;
    body?: string;
    meta?: string;
    align?: "start" | "end";
    as?: "article" | "li";
    avatar?: Snippet;
    actions?: Snippet;
    class?: string;
  };

  let {
    source,
    body,
    meta,
    align = "start",
    as = "article",
    avatar,
    actions,
    class: className,
  }: Props = $props();

  const end = $derived(align === "end");
</script>

<svelte:element
  this={as}
  class={cn(
    "flex min-w-0 flex-col gap-2 border-t border-divider py-5 first:border-t-0",
    end ? "items-end text-right" : "items-start text-left",
    className,
  )}
>
  <div class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
    {#if avatar}
      {@render avatar()}
    {:else}
      <Orb size="inline" variant={end ? "user" : "brand"} />
    {/if}
    <span class="min-w-0 font-semibold break-words text-ink-strong">
      {source}
    </span>
    {#if meta}
      <span class="text-meta text-ink-muted">{meta}</span>
    {/if}
  </div>
  {#if body}
    <p class="text-body-md break-words whitespace-pre-line text-ink-body">
      {body}
    </p>
  {/if}
  {#if actions}
    <div class="flex flex-wrap items-center gap-3 text-ink-faint">
      {@render actions()}
    </div>
  {/if}
</svelte:element>
