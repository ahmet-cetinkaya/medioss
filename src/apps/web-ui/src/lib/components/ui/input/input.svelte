<!--
  @component
  Pill input (DESIGN_SYSTEM §5.11): solid white, 48 px tall, a
  `--control-border` boundary (>= 3:1), `--text-muted` placeholder and the
  `focus-ring` recipe. Same props as the shadcn input: `bind:value`,
  `bind:ref`, `bind:files` (type "file") and every native input attribute.
-->
<script lang="ts">
  import { cn, type WithElementRef } from "$lib/utils.js";
  import type {
    HTMLInputAttributes,
    HTMLInputTypeAttribute,
  } from "svelte/elements";

  type InputType = Exclude<HTMLInputTypeAttribute, "file">;

  type Props = WithElementRef<
    Omit<HTMLInputAttributes, "type"> &
      (
        | { type: "file"; files?: FileList }
        | { type?: InputType; files?: undefined }
      )
  >;

  let {
    ref = $bindable(null),
    value = $bindable(),
    type,
    files = $bindable(),
    class: className,
    "data-slot": dataSlot = "input",
    ...restProps
  }: Props = $props();

  const classes = $derived(
    cn(
      "glass-control focus-ring h-12 w-full min-w-0 rounded-full px-5 text-base text-ink-strong placeholder:text-ink-muted aria-invalid:border-destructive disabled:cursor-not-allowed disabled:opacity-60 file:mr-3 file:border-0 file:bg-transparent file:py-3 file:text-sm file:font-medium file:text-ink-strong",
      className,
    ),
  );
</script>

{#if type === "file"}
  <input
    bind:this={ref}
    data-slot={dataSlot}
    class={classes}
    type="file"
    bind:files
    bind:value
    {...restProps}
  />
{:else}
  <input
    bind:this={ref}
    data-slot={dataSlot}
    class={classes}
    {type}
    bind:value
    {...restProps}
  />
{/if}
