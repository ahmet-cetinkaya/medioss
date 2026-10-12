<!--
  @component
  Compact quick check-in card (FR-108): one button that opens the check-in
  dialog. Props: `onOpen`, `class`.
-->
<script lang="ts">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { t } from "$lib/i18n/locale.svelte";
  import { cn } from "$lib/utils.js";
  import { homeMessages as m } from "./home.messages";

  type Props = { onOpen: () => void; class?: string };

  let { onOpen, class: className }: Props = $props();
</script>

<button
  type="button"
  onclick={onOpen}
  class={cn(
    "focus-ring glass-panel elevation-1 flex cursor-pointer min-h-20 w-full min-w-0 items-center gap-4 rounded-panel border border-rim p-4 text-start motion-safe:transition-colors motion-safe:duration-(--duration-fast) hover:bg-surface-solid/50",
    className,
  )}
>
  <span
    aria-hidden="true"
    class="grid size-12 shrink-0 place-items-center rounded-full bg-surface-solid text-ink-strong"
  >
    <AppIcon name="smiley" />
  </span>
  <span class="flex min-w-0 flex-col">
    <span class="text-heading break-words text-ink-strong">
      {t(m.checkInHeading)}
    </span>
    <span class="text-meta break-words text-ink-muted">
      {t(m.checkInHint)}
    </span>
  </span>
</button>
