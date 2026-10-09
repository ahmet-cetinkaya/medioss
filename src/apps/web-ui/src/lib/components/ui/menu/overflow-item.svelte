<!--
  One item inside an <OverflowMenu>: a bits-ui dropdown item. Destructive
  styling is opt-in; the icon is decorative (the label carries the name).
-->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import AppIcon from "$lib/components/icons/app-icon.svelte";

  let {
    label,
    icon,
    destructive = false,
    disabled = false,
    onclick,
  }: {
    label: string;
    icon?: import("$lib/components/icons/icon-name").IconName;
    destructive?: boolean;
    disabled?: boolean;
    onclick: () => void;
  } = $props();
</script>

<DropdownMenu.Item
  {disabled}
  onSelect={() => onclick()}
  class="focus-ring flex min-h-11 cursor-pointer select-none items-center gap-2 rounded-full px-3 text-sm text-ink-strong outline-none data-highlighted:bg-panel-tint data-disabled:pointer-events-none data-disabled:opacity-50 {destructive
    ? 'text-destructive'
    : ''}"
>
  {#if icon}
    <span class="text-ink-muted" aria-hidden="true">
      <AppIcon name={icon} size={16} />
    </span>
  {/if}
  {label}
</DropdownMenu.Item>
