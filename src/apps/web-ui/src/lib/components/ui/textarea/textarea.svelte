<!--
  @component
  Pill textarea (DESIGN_SYSTEM §5.11): one line renders as a pill; once the
  text needs more than one line the field grows with it and the radius drops
  to `rounded-panel`. Props: `bind:value`, `bind:ref`, `class` and every
  native textarea attribute (`maxlength`, `rows`, `placeholder`, ...).
-->
<script lang="ts">
  import { cn, type WithElementRef } from "$lib/utils.js";
  import type { HTMLTextareaAttributes } from "svelte/elements";

  type Props = WithElementRef<HTMLTextareaAttributes, HTMLTextAreaElement>;

  let {
    ref = $bindable(null),
    value = $bindable(),
    rows = 1,
    class: className,
    ...restProps
  }: Props = $props();

  // One line: 24 px line height + 2 x 12 px padding (border excluded).
  const ONE_LINE_SCROLL_HEIGHT = 48;

  let multiline = $state(false);

  // Auto-grow: re-measure after every value change (field-sizing is not
  // available in every browser).
  $effect(() => {
    void value;
    if (!ref) return;
    ref.style.height = "auto";
    const border = ref.offsetHeight - ref.clientHeight;
    ref.style.height = `${ref.scrollHeight + border}px`;
    multiline = ref.scrollHeight > ONE_LINE_SCROLL_HEIGHT;
  });
</script>

<textarea
  bind:this={ref}
  data-slot="textarea"
  class={cn(
    "glass-control focus-ring block min-h-12 w-full min-w-0 resize-none overflow-hidden rounded-panel px-5 py-3 text-base leading-6 text-ink-strong placeholder:text-ink-muted aria-invalid:border-destructive disabled:cursor-not-allowed disabled:opacity-60",
    multiline ? "rounded-panel" : "rounded-full",
    className,
  )}
  {rows}
  bind:value
  {...restProps}
></textarea>
