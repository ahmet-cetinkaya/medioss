<!--
  @component
  Range filter chips (e.g. Statistics "Today / Last 30 days / All time"):
  a ChipGroup (single-select radios, DESIGN_SYSTEM §5.6), generic over the
  option values.

  Props: `options` ({ value, label }[] with translated labels), `bind:value`,
  `name` (radio group name, unique per page), `legend` (translated),
  `legendHidden` (default true; the legend stays the group's name), `class`.
-->
<script lang="ts" generics="T extends string">
  import { ChipGroup } from "$lib/components/ui/chip-group/index.js";

  type Props = {
    options: readonly { value: T; label: string }[];
    value?: T;
    name: string;
    legend: string;
    legendHidden?: boolean;
    class?: string;
  };

  let {
    options,
    value = $bindable(),
    name,
    legend,
    legendHidden = true,
    class: className,
  }: Props = $props();
</script>

<ChipGroup
  {options}
  bind:value
  {name}
  {legend}
  {legendHidden}
  class={className}
/>
