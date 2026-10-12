<!--
  @component
  Native `<select>` styled like the pill Input (DESIGN_SYSTEM §5.11), so
  options are chosen by name with the platform picker. Props: `bind:value`,
  `class`, `children` (the options), and every native select attribute.
-->
<script lang="ts">
  import type { HTMLSelectAttributes } from "svelte/elements";
  import { cn } from "$lib/utils.js";

  let {
    value = $bindable(),
    class: className,
    children,
    ...restProps
  }: HTMLSelectAttributes = $props();
</script>

<select
  bind:value
  class={cn(
    "focus-ring h-12 w-full min-w-0 rounded-full border border-control-border bg-surface-solid px-5 text-base text-ink-strong disabled:cursor-not-allowed disabled:opacity-60",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</select>
