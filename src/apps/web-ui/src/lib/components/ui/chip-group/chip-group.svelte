<!--
  @component
  Single-select filter chips (DESIGN_SYSTEM §5.6): native radio inputs in a
  fieldset, so arrow keys move the selection and Tab enters and leaves the
  group (FR-098). Each label is a 44 px pill. The selected chip is solid
  white with weight 600 to distinguish its state (FR-092).

  Props: `options` ({ value, label }[] with translated labels), `bind:value`,
  `name` (radio group name, unique per page), `legend` (translated),
  `legendHidden` (visually hide the legend; it stays the group's name),
  `class`.
-->
<script lang="ts" generics="T extends string">
  import { cn } from "$lib/utils.js";

  type Props = {
    options: readonly { value: T; label: string }[];
    value?: T;
    name: string;
    legend: string;
    legendHidden?: boolean;
    showCheck?: boolean;
    class?: string;
  };

  let {
    options,
    value = $bindable(),
    name,
    legend,
    legendHidden = false,
    showCheck = true,
    class: className,
  }: Props = $props();
</script>

<fieldset class={cn("m-0 min-w-0 border-0 p-0", className)}>
  <legend
    class={legendHidden ? "sr-only" : "mb-2 text-sm font-medium text-ink-body"}
  >
    {legend}
  </legend>
  <div class="flex flex-wrap gap-2">
    {#each options as option (option.value)}
      {@const selected = value === option.value}
      <label
        class={cn(
          "relative inline-flex min-h-11 min-w-11 max-w-full cursor-pointer items-center justify-center gap-1.5 rounded-full border border-control-border px-4.5 py-1 text-center text-sm break-words [&_svg]:shrink-0 leading-tight select-none has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-accent-strong motion-safe:transition-colors motion-safe:duration-(--duration-fast)",
          selected
            ? "bg-surface-solid font-semibold text-ink-strong"
            : "bg-transparent font-normal text-ink-body hover:bg-surface-solid/50",
        )}
      >
        <input
          type="radio"
          class="sr-only"
          {name}
          value={option.value}
          bind:group={value}
        />
        {option.label}
        {#if selected && showCheck}
          <span aria-hidden="true" class="text-ink-muted">✓</span>
        {/if}
      </label>
    {/each}
  </div>
</fieldset>
