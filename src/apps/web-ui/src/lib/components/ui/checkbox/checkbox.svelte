<!--
  @component
  Labelled checkbox: a native `type="checkbox"` (Space toggles it, assistive
  technology reports checked/unchecked) with a visible box and label. The
  whole row is a 44 px label target and the checked state shows through the
  check mark and the filled box, not color alone (FR-092).

  Props: `bind:checked`, `label` (translated accessible name, shown), `class`,
  and every native input attribute (`disabled`, `name`, `id`, `onchange`, ...).
-->
<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { cn } from "$lib/utils.js";

  type Props = Omit<HTMLInputAttributes, "type" | "checked"> & {
    checked?: boolean;
    label: string;
  };

  let {
    checked = $bindable(false),
    label,
    class: className,
    ...restProps
  }: Props = $props();
</script>

<label
  class={cn(
    "inline-flex min-h-11 max-w-full cursor-pointer items-center gap-3 text-sm text-ink-strong select-none has-disabled:cursor-not-allowed has-disabled:opacity-60",
    className,
  )}
>
  <input type="checkbox" class="peer sr-only" bind:checked {...restProps} />
  <span
    aria-hidden="true"
    class={cn(
      "inline-flex size-6 shrink-0 items-center justify-center rounded-md border text-primary-foreground peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-accent-strong motion-safe:transition-colors motion-safe:duration-(--duration-fast)",
      checked ? "border-accent-fill bg-accent-fill" : "glass-control",
    )}
  >
    {#if checked}
      <AppIcon name="check" size={16} />
    {/if}
  </span>
  <span class="min-w-0 break-words">{label}</span>
</label>
