<!--
  @component
  A meditation as a full-width list row (Program screen, Library favorites):
  title, one meta line, an optional detail line and description, and a play
  button on the right.

  Props: `title` (user content, shown as-is), `meta` (formatted line),
  `detail?` (a second muted line, e.g. the programs it belongs to),
  `description?` (clamped to three lines), `playLabel` (the play button's
  accessible name), `playHref`.
-->
<script lang="ts">
  import { IconButton } from "$lib/components/ui/icon-button/index.js";

  type Props = {
    title: string;
    meta: string;
    detail?: string | null;
    description?: string | null;
    playLabel: string;
    playHref: string;
  };

  let { title, meta, detail, description, playLabel, playHref }: Props =
    $props();
</script>

<div
  class="flex w-full min-w-0 items-center gap-2 rounded-thumb border border-rim bg-surface-solid/70 p-3"
>
  <div class="flex min-w-0 flex-1 flex-col gap-1">
    <h3 class="text-body-md break-words text-ink-strong">{title}</h3>
    <p class="text-meta text-ink-muted">{meta}</p>
    {#if detail}
      <p class="text-meta break-words text-ink-muted">{detail}</p>
    {/if}
    {#if description}
      <p
        class="line-clamp-3 text-sm break-words whitespace-pre-line text-ink-body"
      >
        {description}
      </p>
    {/if}
  </div>
  <IconButton
    label={playLabel}
    icon="play"
    filled
    href={playHref}
    class="shrink-0 text-accent-strong"
  />
</div>
