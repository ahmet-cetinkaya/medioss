<!--
  @component
  One-row filter dropdown (DESIGN_SYSTEM §5.11): a native `<select>` styled
  like the pill Input, so the platform picker shows the options. It replaces
  the wrapping chip rows for a screen with a single filter field.

  Props: `options` ({ value, label }[] with translated labels), `bind:value`,
  `legend` (translated name; the select's accessible label, visually hidden),
  `class`. The label id is generated per instance, so a control rendered twice
  (header slot) never shares an id.
-->
<script lang="ts" generics="T extends string">
  import AppIcon from "$lib/components/icons/app-icon.svelte";
  import { cn } from "$lib/utils.js";

  const id = $props.id();

  type Props = {
    options: readonly { value: T; label: string }[];
    value?: T;
    legend: string;
    class?: string;
  };

  let {
    options,
    value = $bindable(),
    legend,
    class: className,
  }: Props = $props();
</script>

<div class={cn("relative w-full sm:max-w-xs", className)}>
  <label for="{id}-filter" class="sr-only">{legend}</label>
  <select
    id="{id}-filter"
    bind:value
    class="glass-control focus-ring h-12 w-full min-w-0 appearance-none rounded-full ps-5 pe-11 text-base text-ink-strong"
  >
    {#each options as option (option.value)}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
  <span
    aria-hidden="true"
    class="pointer-events-none absolute inset-y-0 end-4 flex items-center text-ink-muted"
  >
    <AppIcon name="caret-down" size={18} />
  </span>
</div>
