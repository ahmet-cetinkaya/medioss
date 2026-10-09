<!--
  @component
  Native range slider: arrow, Page and Home/End keys work natively (FR-098).
  44 px tall hit area; the track (`--control-border`) and the thumb rim
  (`--accent-fill`) meet 3:1 (FR-090).

  Props: `bind:value` (number), `min` (0), `max` (100), `step` (1), `label`
  (translated accessible name), `valueText` (translated `aria-valuetext`,
  e.g. "40 percent"), `class`, and every native input attribute.
-->
<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { cn } from "$lib/utils.js";

  type Props = Omit<HTMLInputAttributes, "type" | "value"> & {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    label: string;
    valueText?: string;
  };

  let {
    value = $bindable(0),
    min = 0,
    max = 100,
    step = 1,
    label,
    valueText,
    class: className,
    ...restProps
  }: Props = $props();

  const fill = $derived(
    max > min ? `${((value - min) / (max - min)) * 100}%` : "0%",
  );
</script>

<input
  type="range"
  class={cn(
    "focus-ring h-11 w-full min-w-0 cursor-pointer appearance-none rounded-full bg-transparent disabled:cursor-not-allowed disabled:opacity-60",
    "[&::-webkit-slider-runnable-track]:h-1.5 [&::-webkit-slider-runnable-track]:rounded-full [&::-webkit-slider-runnable-track]:bg-[linear-gradient(to_right,var(--accent-fill)_var(--fill),var(--control-border)_var(--fill))]",
    "[&::-webkit-slider-thumb]:-mt-[7px] [&::-webkit-slider-thumb]:size-5 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-accent-fill [&::-webkit-slider-thumb]:bg-surface-solid [&::-webkit-slider-thumb]:shadow-none",
    "[&::-moz-range-track]:h-1.5 [&::-moz-range-track]:rounded-full [&::-moz-range-track]:bg-control-border [&::-moz-range-progress]:h-1.5 [&::-moz-range-progress]:rounded-full [&::-moz-range-progress]:bg-accent-fill",
    "[&::-moz-range-thumb]:size-5 [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-accent-fill [&::-moz-range-thumb]:bg-surface-solid [&::-moz-range-thumb]:shadow-none",
    className,
  )}
  style:--fill={fill}
  {min}
  {max}
  {step}
  aria-label={label}
  aria-valuetext={valueText}
  bind:value
  {...restProps}
/>
