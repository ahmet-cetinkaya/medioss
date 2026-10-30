<!--
  @component
  A meditation as a full-width list row (Program screen, Library favorites):
  title, one meta line, an optional detail line and description, and a play
  glyph on the right. The whole row opens the Player: the title is the one
  link and stretches over the row, so the glyph is decorative.

  Props: `title` (user content, shown as-is), `meta` (formatted line),
  `detail?` (a second muted line, e.g. the programs it belongs to),
  `description?` (clamped to three lines), `playLabel` (the link's
  accessible name), `playHref`.
-->
<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";

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
  class="relative flex w-full min-w-0 cursor-pointer items-center gap-2 rounded-thumb border border-rim bg-surface-solid/70 p-3 motion-safe:transition-colors motion-safe:duration-(--duration-fast) hover:text-accent-strong"
>
  <div class="flex min-w-0 flex-1 flex-col gap-1">
    <h3 class="text-body-md break-words text-ink-strong">
      <a
        href={playHref}
        aria-label={playLabel}
        class="focus-ring rounded-thumb after:absolute after:inset-0 after:content-['']"
        >{title}</a
      >
    </h3>
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
  <span
    aria-hidden="true"
    class="flex size-12 shrink-0 items-center justify-center rounded-full text-accent-strong"
  >
    <AppIcon name="play" filled />
  </span>
</div>
